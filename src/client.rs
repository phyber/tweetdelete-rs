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
use std::fmt;

use crate::config::Config;
use crate::errors::Error;

// Maximum number of tweets we can request per API call.
// https://developer.twitter.com/en/docs/twitter-api/v1/tweets/timelines/api-reference/get-statuses-user_timeline
const MAX_TIMELINE_COUNT: i32 = 200;

// The stored Duration here isn't used for now, but in the future the delete
// output should mention the age of the deleted Tweet.
#[derive(Debug)]
enum StatusAction {
    Delete(Duration),
    Keep(Duration),
}

#[derive(Debug)]
struct Status<'a> {
    tweet: &'a Tweet,
}

impl<'a> Status<'a> {
    fn new(tweet: &'a Tweet) -> Self {
        Self {
            tweet: tweet,
        }
    }

    // Checks the tweet age against a given max_age
    fn action(&self, max_age: i64) -> StatusAction {
        let now: DateTime<Utc> = Utc::now();
        let diff = now - self.tweet.created_at;

        if diff > Duration::days(max_age) {
            StatusAction::Delete(diff)
        }
        else {
            StatusAction::Keep(diff)
        }
    }

    // Delete the tweet and give some output to log
    async fn delete(&self, token: &Token) -> Result<Response<Tweet>, Error> {
        let id = self.tweet.id;
        let deleted: Response<Tweet> = delete(id, token).await?;

        println!("{}", self);

        Ok(deleted)
    }
}

impl<'a> fmt::Display for Status<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{time}/{id}: {tweet}",
            time=self.tweet.created_at,
            id=self.tweet.id,
            tweet=self.tweet.text,
        )
    }
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
                let status = Status::new(status);

                if let StatusAction::Delete(_) = status.action(max_age) {
                    status.delete(&self.token).await?;

                    num_deleted += 1;
                }
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
