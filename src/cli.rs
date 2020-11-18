// cli: Handle command line parsing
use clap::{
    crate_description,
    crate_name,
    crate_version,
    App,
    Arg,
    ArgMatches,
};

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        // Flags
        .arg(
            Arg::with_name("DRY_RUN")
                .long("dry-run")
                .help("Show what would have been deleted without deleting.")
                .takes_value(false)
        )
}

pub fn parse_args<'a>() -> ArgMatches<'a> {
    create_app().get_matches()
}
