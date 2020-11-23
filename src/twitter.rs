// Deletes tweets older than the age specified in the config.
use egg_mode::{
    auth::verify_tokens,
    tweet::user_timeline,
    Token,
};

use crate::config::Config;
use crate::errors::Error;

mod status;
use status::*;

// Maximum number of tweets we can request per API call.
// https://developer.twitter.com/en/docs/twitter-api/v1/tweets/timelines/api-reference/get-statuses-user_timeline
const MAX_TIMELINE_COUNT: i32 = 200;

#[derive(Debug)]
pub struct Twitter {
    config: Config,
    token: Token,
    user_id: u64,
}

impl Twitter {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let token = config.access_token();
        let verified = verify_tokens(&token).await?;

        let client = Self {
            config: config,
            token: token,
            user_id: verified.id,
        };

        Ok(client)
    }

    // Process the timeline, deleting tweets older than max_age
    pub async fn process_timeline(&self) -> Result<usize, Error> {
        let dry_run = self.config.dry_run();
        let max_age = self.config.max_tweet_age();

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
                    if dry_run {
                        println!("DryRun - Would have deleted: {}", status);
                    }
                    else {
                        status.delete(&self.token).await?;
                        num_deleted += 1;
                    }
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
