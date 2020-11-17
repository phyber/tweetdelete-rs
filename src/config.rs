// Deletes tweets older than the age specified in the config.
use egg_mode::{
    KeyPair,
    Token,
};
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_yaml;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::errors::Error;

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
pub struct Config {
    api: ApiConfig,
    general: GeneralConfig,
}

impl Config {
    // Gets a new config from the path
    pub fn new(file_path: &str) -> Result<Self, Error> {
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
    pub fn access_token(&self) -> Token {
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
    pub fn log_file(&self) -> &Option<String> {
        &self.general.log_file
    }

    // Return the maximum age of Tweets we want to keep
    pub fn max_tweet_age(&self) -> i64 {
        self.general.max_tweet_age
    }
}
