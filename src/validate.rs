use clap::{Arg, ArgMatches, App, SubCommand};
use colored::Colorize;
use toml::{from_str, Value as Toml};

use config::ConfigV1;
use ::exit_with_error;

pub const NAME: &'static str = "validate";

pub fn command() -> App<'static, 'static> {
    SubCommand::with_name(NAME)
        .about("Validate configuration file")
        .arg(Arg::with_name("config file")
             .long("config")
             .default_value(".imagetest.toml")
        )
}

pub fn run(matches: &ArgMatches) {
    let path = matches.value_of("config file").unwrap();

    let contents = ::config::read_config_file(path);
    let mut full_toml: Toml = from_str(&contents).expect("Could not parse config file as TOML");

    let full_table = full_toml.as_table_mut().expect("Config file TOML is not a table");
    if let Some(version_value) = full_table.remove("version") {
        match version_value {
            Toml::Integer(1) => {
                let config: Result<ConfigV1, _> = from_str(&contents);
                if let Err(_) = config {
                    exit_with_error("Could not parse config file into expected config format", false);
                }
            },
            Toml::Integer(version) => exit_with_error(&*format!("{}{}",
                                               "Invalid config format version ".red(),
                                               version.to_string().bold()), true),
            _ => exit_with_error(&*format!("{}{}",
                          "\"version\"".bold(),
                          " must be an integer".red()), true),
        }
    } else {
        exit_with_error(&*format!("{}{}{}",
                 "Could not find ".red(),
                 "\"version\"".bold(),
                 " key. Can not determine config format version.".red(),
        ), true);
        return;
    }
}
