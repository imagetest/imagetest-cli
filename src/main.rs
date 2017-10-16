#[macro_use] extern crate serde_derive;

extern crate clap;
extern crate colored;
extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate toml;

mod config;
mod validate;
pub mod test;

use clap::{App};
use std::fmt::Display;
use colored::Colorize;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("ImageTest CLI")
        .version(VERSION)
        .subcommand(validate::command())
        .subcommand(test::command())
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(validate::NAME) {
        validate::run(matches);
    } else if let Some(matches) = matches.subcommand_matches(test::NAME) {
        test::run(matches);
    }
}

pub fn exit_with_error<S: Display + Colorize>(message: S, print_raw: bool) {
    if print_raw {
        println!("{}", message);
    } else {
        println!("{}", message.red());
    }

    ::std::process::exit(1);
}
