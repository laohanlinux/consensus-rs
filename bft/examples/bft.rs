#[macro_use]
extern crate bft;
extern crate clap;
#[macro_use]
extern crate log;
extern crate actix;

use ::actix::prelude::*;
use clap::{Arg, App, SubCommand, ArgMatches};

use std::sync::mpsc::channel;

fn main() {
    let _config = bft::config::Config::default();
    let matches = App::new("bft-consensus")
        .version("v0.1")
        .author("Rg. <daimaldd@gmail.com>")
        .about("bft consensus block chain implements")
        .subcommand(SubCommand::with_name("start")
            .about("star bft-rs")
            .arg(
                Arg::with_name("config")
                    .long("config")
                    .default_value("config.toml")
                    .short("c")
                    .value_name("CONFIG")))
        .get_matches();
    let result = run(matches);
    if let Err(err) = result {
        println!("--->{}", err);
    }
}

fn run(matches: ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("start", Some(m)) => {
            run_start(&m)
        }
        _ => Err("not matches any command".to_string())
    }
}

fn run_start(matches: &ArgMatches) -> Result<(), String> {
    let config = matches.value_of("config").expect("config is None");
    let (tx, rx) = channel();
    bft::cmd::start_node(config, tx)?;
    rx.recv().unwrap();
    Ok(())
}