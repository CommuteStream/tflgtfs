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

fn arg_format<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("format")
        .help("Output format")
        .possible_values(&["gtfs"])
        .long("format")
        .value_name("format")
}

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("tfl")
                      .version(env!("CARGO_PKG_VERSION"))
                      .about("Tfl consumer")
                      .subcommand(SubCommand::with_name("fetch-lines")
                                             .about("Fetch lines from Tfl")
                                             .arg(arg_format())
                                             .arg(Arg::with_name("threads")
                                                      .help("Number of threads. Defaults to 5")
                                                      .long("threads")
                                                      .value_name("number")))
                      .subcommand(SubCommand::with_name("transform")
                                             .about("Transform cached data to the given format")
                                             .arg(arg_format()
                                                      .index(1)
                                                      .required(true))
                                             .arg(Arg::with_name("threads")
                                                      .help("Number of threads. Defaults to 5")
                                                      .long("threads")
                                                      .value_name("number")))
                      .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("fetch-lines") {
        let format = value_t!(matches, "format", OutputFormat).unwrap_or(OutputFormat::None);
        let thread_number = value_t!(matches, "threads", u32).unwrap_or(5);
        cmd::fetch_lines(format, thread_number);
    }

    if let Some(ref matches) = matches.subcommand_matches("transform") {
        let format = value_t!(matches, "format", OutputFormat).unwrap_or_else(|e| e.exit());
        let thread_number = value_t!(matches, "threads", u32).unwrap_or(5);
        cmd::transform(format, thread_number);
    }
}
