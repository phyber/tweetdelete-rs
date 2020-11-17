// Deletes tweets older than the age specified in the config.
mod client;
mod config;
mod errors;

use client::Client;
use config::Config;
use errors::Error;

// Path to config file.
const USER_CONFIG_PATH: &str = "~/.tweetdelete.yaml";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::new(USER_CONFIG_PATH)?;
    let max_age = config.max_tweet_age();

    let client = Client::new(config).await?;
    let num_deleted = client.process_timeline(max_age).await?;

    println!("Finished. {count} tweets deleted.", count=num_deleted);

    Ok(())
}
