// Deletes tweets older than the age specified in the config.
mod cli;
mod config;
mod errors;
mod twitter;

use config::{
    Config,
    Setting,
};
use errors::Error;
use twitter::Twitter;

// Path to config file.
const USER_CONFIG_PATH: &str = "~/.tweetdelete.yaml";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let matches = cli::parse_args();
    let mut config = Config::new(USER_CONFIG_PATH)?;

    // Command line arguments will always override whatever is in the config
    // file.
    if matches.is_present("DRY_RUN") {
        config.set(Setting::DryRun(true));
    }

    let twitter = Twitter::new(config).await?;
    let num_deleted = twitter.process_timeline().await?;

    println!("Finished. {count} tweets deleted.", count=num_deleted);

    Ok(())
}
