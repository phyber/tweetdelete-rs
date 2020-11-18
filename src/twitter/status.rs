// Deletes tweets older than the age specified in the config.
use chrono::{
    DateTime,
    Duration,
    Utc,
};
use egg_mode::{
    tweet::delete,
    tweet::Tweet,
    Response,
    Token,
};
use std::fmt;

use crate::errors::Error;

// The stored Duration here isn't used for now, but in the future the delete
// output should mention the age of the deleted Tweet.
#[derive(Debug)]
pub enum StatusAction {
    Delete(Duration),
    Keep(Duration),
}

#[derive(Debug)]
pub struct Status<'a> {
    tweet: &'a Tweet,
}

impl<'a> Status<'a> {
    pub fn new(tweet: &'a Tweet) -> Self {
        Self {
            tweet: tweet,
        }
    }

    // Checks the tweet age against a given max_age
    pub fn action(&self, max_age: i64) -> StatusAction {
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
    pub async fn delete(&self, token: &Token) -> Result<Response<Tweet>, Error> {
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
