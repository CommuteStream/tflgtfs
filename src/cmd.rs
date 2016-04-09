use scoped_threadpool::Pool;
use std::sync::Arc;
use std::collections::HashSet;
use std::process;

use tfl::*;
use gtfs::*;
use format::{OutputFormat};


pub fn fetch_lines(format: OutputFormat) {
    let lines = load_lines(DataSource::API);

    match format {
        OutputFormat::GTFS => transform_gtfs(lines),
        _ => process::exit(0),
    }
}

pub fn transform(format: OutputFormat) {
    let lines = load_lines(DataSource::Cache);

    match format {
        OutputFormat::GTFS => transform_gtfs(lines),
        _ => process::exit(0),
    }
}


fn load_lines(data_source: DataSource) -> Vec<Line> {
    let mut pool = Pool::new(5);
    let client = Arc::new(Client::new());

    let mut lines = match data_source {
        DataSource::Cache => client.get_cached_lines(),
        DataSource::API   => client.get_lines(),
    };

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

    lines
}

fn transform_gtfs(lines: Vec<Line>) {
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
