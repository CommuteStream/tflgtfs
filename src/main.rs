#![feature(custom_derive)]

extern crate hyper;
extern crate rustc_serialize;
extern crate scoped_threadpool;
extern crate csv;

mod geometry;
mod tfl;
mod gtfs;

use std::sync::Arc;
use std::collections::HashSet;

use scoped_threadpool::Pool;

use tfl::*;
use gtfs::*;

fn main() {
    let mut pool = Pool::new(6);
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

    // Generate a report
    let mut line_count = 0;
    let mut line_ids : HashSet<String> = HashSet::new();
    let mut route_section_count = 0;
    let mut route_section_ids: HashSet<String> = HashSet::new();
    let mut schedule_names: HashSet<String> = HashSet::new();
    for line in &lines {
        println!("{}, Duplicate: {}", line.id, line_ids.contains(&line.id));
        for route_section in &line.routeSections {
            let has_timetable = match route_section.timetable {
                Some(ref timetable) => {
                    for schedule in &timetable.first_timetable().unwrap().schedules {
                        schedule_names.insert(schedule.name.clone());
                    }
                    true
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
