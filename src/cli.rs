// cli: Handle command line parsing
use clap::{
    crate_description,
    crate_name,
    crate_version,
    App,
    Arg,
    ArgMatches,
};

fn is_valid_max_tweet_age(v: String) -> Result<(), String> {
    let i: i64 = match v.parse() {
        Ok(i)  => Ok(i),
        Err(e) => Err(format!("{}", e)),
    }?;

    if i < 0 {
        return Err("value cannot be a negative number of days".into())
    }

    Ok(())
}

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        // Flags
        .arg(
            Arg::with_name("DRY_RUN")
                .env("DRY_RUN")
                .long("dry-run")
                .help("Show what would have been deleted without deleting.")
                .takes_value(false)
        )
        // Options
        .arg(
            Arg::with_name("MAX_TWEET_AGE")
                .env("MAX_TWEET_AGE")
                .hide_env_values(true)
                .value_name("DAYS")
                .long("max-tweet-age")
                .help("Maximum age of a Tweet in days before it's eligible for deletion.")
                .takes_value(true)
                .validator(is_valid_max_tweet_age)
        )
}

pub fn parse_args<'a>() -> ArgMatches<'a> {
    create_app().get_matches()
}
