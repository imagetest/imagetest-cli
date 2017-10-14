use clap::{Arg, ArgMatches, App, SubCommand};
use toml::{from_str};

use config::ConfigV1;
use ::exit_with_error;

pub const NAME: &'static str = "test";

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Run tests")
        .arg(Arg::with_name("config file")
             .long("config")
             .default_value(".imagetest.toml")
        )
}

pub fn run(matches: &ArgMatches) {
    let path = matches.value_of("config file").unwrap();

    let contents = ::config::read_config_file(path);
    let config_res: Result<ConfigV1, _> = from_str(&contents);
    let _ = match config_res {
        Err(_) => {
            exit_with_error("Could not parse config file into expected config format", false);
            return;
        },
        Ok(val) => val,
    };
}
