#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate env_logger;


use clap::{Arg, App, SubCommand};



extern crate hyper;
extern crate rustc_serialize;
extern crate scoped_threadpool;
extern crate csv;

mod format;
mod geometry;
mod tfl;
mod gtfs;

use std::sync::Arc;
use std::collections::HashSet;

use scoped_threadpool::Pool;

use tfl::*;
use gtfs::*;
use format::{OutputFormat};

fn fetch_lines() {
    let mut pool = Pool::new(5);
    let client = Arc::new(Client::new());

    // Fetch data
    let mut lines = client.get_lines();
    pool.scoped(|scope| {
        for line in &mut lines {
            let client = client.clone();
            scope.execute(move || {
                line.inbound_sequence = client.get_sequence(&line.id, "inbound");
                line.outbound_sequence = client.get_sequence(&line.id, "outbound");
                line.stops = Some(client.get_stops(&line.id));
                for route_section in &mut line.routeSections {
                    println!("Getting Timetable for Line: {}, Route Section: {} ...", line.name, route_section.name);
                    route_section.timetable = client.get_timetable(&line.id, &route_section.originator, &route_section.destination);
                }
            });
        }
    });
}

fn transform(format: OutputFormat) {
    match format {
        OutputFormat::GTFS => write_gtfs_temp(),
        _ => println!("nope")
    }
}

fn write_gtfs_temp() {
    let mut pool = Pool::new(5);
    let client = Arc::new(Client::new());

    // Fetch data
    let mut lines = client.get_cached_lines();
    pool.scoped(|scope| {
        for line in &mut lines {
            let client = client.clone();
            scope.execute(move || {
                line.inbound_sequence = client.get_sequence(&line.id, "inbound");
                line.outbound_sequence = client.get_sequence(&line.id, "outbound");
                line.stops = Some(client.get_stops(&line.id));
                for route_section in &mut line.routeSections {
                    println!("Getting Timetable for Line: {}, Route Section: {} ...", line.name, route_section.name);
                    route_section.timetable = client.get_timetable(&line.id, &route_section.originator, &route_section.destination);
                }
            });
        }
    });


    // Generate a report
    let mut line_count = 0;
    let mut line_ids: HashSet<String> = HashSet::new();
    let mut route_section_count = 0;
    let mut route_section_ids: HashSet<String> = HashSet::new();
    let mut schedule_names: HashSet<String> = HashSet::new();

    for line in &lines {
        println!("{}, Duplicate: {}", line.id, line_ids.contains(&line.id));
        for route_section in &line.routeSections {
            let has_timetable = true; //TODO: Fix me

            match route_section.timetable {
                Some(ref timetable) => {
                    let names = collect_schedule_names(timetable);
                    schedule_names = schedule_names.union(&names).cloned().collect::<HashSet<String>>();
                    names.is_empty()
                },
                None => false,
            };

            let id = route_section_id(&line, &route_section);
            println!("\t{}, Has Timetable: {}, Duplicate: {}", id, has_timetable, route_section_ids.contains(&id));
            route_section_ids.insert(id.clone());
            route_section_count += 1;
        }
        line_count += 1;
        line_ids.insert(line.id.clone());
    }
    println!("Duplicate Lines: {}, Duplicate Route Sections: {}", line_count-line_ids.len(), route_section_count-route_section_ids.len());

    println!("Schedule Names:");
    for schedule_name in &schedule_names {
        println!("\t{}", schedule_name);
    }

    // Generate CSV files from fetched data
    write_gtfs(&lines);
}

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("tfl")
                      .version(env!("CARGO_PKG_VERSION"))
                      .about("Tfl consumer")
                      .subcommand(SubCommand::with_name("fetch-lines")
                                             .about("Fetch lines from Tfl."))
                      .subcommand(SubCommand::with_name("transform")
                                             .about("Transform cached data to format")
                                             .arg(Arg::with_name("format")
                                                      .help("Repo path (e.g. ustwo/mastermind)")
                                                      .index(1)
                                                      .possible_values(&["gtfs"])
                                                      .required(true)))
                      .get_matches();

    if let Some(_) = matches.subcommand_matches("fetch-lines") {
        fetch_lines();
    }

    if let Some(ref matches) = matches.subcommand_matches("transform") {
        let format = value_t!(matches, "format", OutputFormat).unwrap_or_else(|e| e.exit());
        transform(format);
    }
}
