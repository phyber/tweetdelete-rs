// Handles errors
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IoError: {0}")]
    IoError(#[from] ::std::io::Error),

    #[error("Twitter: {0}")]
    Twitter(#[from] egg_mode::error::Error),

    #[error("YamlDecode")]
    YamlDecode(#[from] serde_yaml::Error),
}
