#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate env_logger;

extern crate hyper;
extern crate rustc_serialize;
extern crate scoped_threadpool;
extern crate csv;

mod format;
mod geometry;
mod tfl;
mod gtfs;
mod cmd;

use clap::{Arg, App, SubCommand};
use format::{OutputFormat};

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("tfl")
                      .version(env!("CARGO_PKG_VERSION"))
                      .about("Tfl consumer")
                      .subcommand(SubCommand::with_name("fetch-lines")
                                             .about("Fetch lines from Tfl.")
                                             .arg(Arg::with_name("format")
                                                      .help("Output format")
                                                      .long("format")
                                                      .value_name("format")
                                                      .possible_values(&["gtfs"])))
                      .subcommand(SubCommand::with_name("transform")
                                             .about("Transform cached data to format")
                                             .arg(Arg::with_name("format")
                                                      .help("Output format")
                                                      .index(1)
                                                      .possible_values(&["gtfs"])
                                                      .required(true)))
                      .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("fetch-lines") {
        let format = value_t!(matches, "format", OutputFormat).unwrap_or(OutputFormat::None);
        cmd::fetch_lines(format);
    }

    if let Some(ref matches) = matches.subcommand_matches("transform") {
        let format = value_t!(matches, "format", OutputFormat).unwrap_or_else(|e| e.exit());
        cmd::transform(format);
    }
}
