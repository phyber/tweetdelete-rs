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
    Response,
    Token,
};

use crate::config::Config;
use crate::errors::Error;

// Maximum number of tweets we can request per API call.
// https://developer.twitter.com/en/docs/twitter-api/v1/tweets/timelines/api-reference/get-statuses-user_timeline
const MAX_TIMELINE_COUNT: i32 = 200;

#[derive(Debug)]
enum TweetAction {
    Delete(Duration),
    Keep(Duration),
}

#[derive(Debug)]
pub struct Client {
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
