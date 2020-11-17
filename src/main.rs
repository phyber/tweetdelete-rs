// Deletes tweets older than the age specified in the config.
use chrono::{
    DateTime,
    Duration,
    Utc,
};
use egg_mode::{
    auth::verify_tokens,
    tweet::delete,
    tweet::user_timeline,
    tweet::Tweet,
    KeyPair,
    Response,
    Token,
};
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_yaml;
use std::fs::File;
use std::io::BufReader;
use std::path::{
    Path,
    PathBuf,
};

mod errors;
use errors::Error;

// Path to config file.
const USER_CONFIG_PATH: &str = "~/.tweetdelete.yaml";

// Maximum number of tweets we can request per API call.
// https://developer.twitter.com/en/docs/twitter-api/v1/tweets/timelines/api-reference/get-statuses-user_timeline
const MAX_TIMELINE_COUNT: i32 = 200;

#[derive(Debug)]
enum TweetAction {
    Delete(Duration),
    Keep(Duration),
}

// ApiConfig is always required in the config file
#[derive(Debug, Serialize, Deserialize)]
struct ApiConfig {
    access_token_key: String,
    access_token_secret: String,
    consumer_key: String,
    consumer_secret: String,
}

// General config is optional, and comes with some sensible defaults in the
// Default impl
#[derive(Debug, Serialize, Deserialize)]
struct GeneralConfig {
    log_file: Option<String>,
    max_tweet_age: i64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_file: None,
            max_tweet_age: 180,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    api: ApiConfig,
    general: GeneralConfig,
}

impl Config {
    // Gets a new config from the path
    fn new(file_path: &str) -> Result<Self, Error> {
        // Work out the real path
        let path = shellexpand::tilde(file_path);

        // We should work out how to use the Cow<str> that shellexpand::tilde
        // returns here, but for now, just doing to_string is quicker
        let path = path.to_string();
        let path = Path::new(&path);

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_yaml::from_reader(reader)?;

        Ok(config)
    }

    // Return an access token based on the config values.
    fn access_token(&self) -> Token {
        let consumer_token = KeyPair::new(
            self.api.consumer_key.to_string(),
            self.api.consumer_secret.to_string(),
        );

        let access_token = KeyPair::new(
            self.api.access_token_key.to_string(),
            self.api.access_token_secret.to_string(),
        );

        Token::Access {
            access: access_token,
            consumer: consumer_token,
        }
    }

    // Return the path to the logfile
    // TODO: This should actually return an Option<Path>
    fn log_file(&self) -> &Option<String> {
        &self.general.log_file
    }

    // Return the maximum age of Tweets we want to keep
    fn max_tweet_age(&self) -> i64 {
        self.general.max_tweet_age
    }
}

#[derive(Debug)]
struct Client {
    token: Token,
    user_id: u64,
}

impl Client {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let token = config.access_token();
        let verified = verify_tokens(&token).await?;

        let client = Self {
            token: token,
            user_id: verified.id,
        };

        Ok(client)
    }

    async fn delete(&self, id: u64)
    -> Result<Response<Tweet>, Error> {
        let deleted: Response<Tweet> = delete(id, &self.token).await?;

        Ok(deleted)
    }

    // Process the timeline, deleting tweets older than max_age
    pub async fn process_timeline(&self, max_age: i64) -> Result<usize, Error> {
        // Get our own timeline, including replies and RTs.
        let timeline = user_timeline(self.user_id, true, true, &self.token)
            .with_page_size(MAX_TIMELINE_COUNT);

        // Keep track of how many tweets were deleted
        let mut num_deleted: usize = 0;

        // Start processing the timeline
        let (mut timeline, mut feed) = timeline.start().await?;

        loop {
            // If there's nothing in the feed, we're done
            if feed.is_empty() {
                break;
            }

            // Loop over statuses in the feed
            for status in &*feed {
                let created_at = status.created_at;
                let tweet_id   = status.id;
                let tweet      = &status.text;

                match tweet_action(created_at, max_age) {
                    TweetAction::Delete(_) => {
                        println!("DELORTED");
                        num_deleted += 1;
                    },
                    _ => {},
                }

                println!(
                    "{time}/{id}: {tweet}",
                    time=created_at,
                    id=tweet_id,
                    tweet=tweet,
                );
            }

            // Get the next batch of Tweets
            // We can't directly do this assignment, so we use some temporary
            // values.
            let (inner_timeline, inner_feed) = timeline.older(None).await?;
            timeline = inner_timeline;
            feed     = inner_feed;
        }

        Ok(num_deleted)
    }
}

// Checks the tweet age against our max age.
fn tweet_action(created_at: DateTime<Utc>, max_age: i64) -> TweetAction {
    let now: DateTime<Utc> = Utc::now();
    let diff = now - created_at;

    if diff > Duration::days(max_age) {
        TweetAction::Delete(diff)
    }
    else {
        TweetAction::Keep(diff)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::new(USER_CONFIG_PATH)?;
    let max_age = config.max_tweet_age();

    let client = Client::new(config).await?;
    let num_deleted = client.process_timeline(max_age).await?;

    println!("Finished. {count} tweets deleted.", count=num_deleted);

    Ok(())
}
