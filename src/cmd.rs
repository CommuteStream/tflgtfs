use ansi_term::Colour::{Green, Red, White, Blue};
use rand::distributions::{IndependentSample, Range};
use rand;
use scoped_threadpool::Pool;
use std::collections::HashSet;
use std::process;
use std::sync::Arc;

use format::{OutputFormat};
use gtfs::{write_gtfs, route_section_id};
use tfl::line::{Line};
use tfl::client::{Client, DataSource};


pub fn fetch_lines(format: OutputFormat, thread_number: u32, sample_size: Option<usize>) {
    let lines = load_lines(DataSource::API, thread_number, sample_size);

    match format {
        OutputFormat::GTFS => transform_gtfs(lines),
        _ => process::exit(0),
    }
}

pub fn transform(format: OutputFormat, thread_number: u32, sample_size: Option<usize>) {
    let lines = load_lines(DataSource::Cache, thread_number, sample_size);

    match format {
        OutputFormat::GTFS => transform_gtfs(lines),
        _ => process::exit(0),
    }
}

fn sample<T: Clone>(xs: Vec<T>, size: usize) -> Vec<T> {
    let len = xs.len();

    if size > len { return xs }

    let between = Range::new(0usize, (len - size));
    let mut rng = rand::thread_rng();
    let seed = between.ind_sample(&mut rng);
    let lower = seed;
    let upper = seed + size;

    println!("{}: {}..{}", Green.bold().paint("Sample window"), lower, upper);

    xs[lower .. upper].to_vec()
}

#[test]
fn sample_test() {
    assert_eq!(sample(vec![0; 100], 200).len(), 100);
    assert_eq!(sample(vec![0; 100], 10).len(), 10);
}

fn load_lines(data_source: DataSource, thread_number: u32, sample_size: Option<usize>) -> Vec<Line> {
    let mut pool = Pool::new(thread_number);
    let client = Arc::new(Client::new());

    let mut lines = match data_source {
        DataSource::Cache => client.get_cached_lines(),
        DataSource::API   => client.get_lines(),
    };

    if let Some(n) = sample_size {
        lines = sample(lines, n);
    }

    pool.scoped(|scope| {
        for line in &mut lines {
            let client = client.clone();
            scope.execute(move || {
                line.inbound_sequence = client.get_sequence(&line.id, "inbound");
                line.outbound_sequence = client.get_sequence(&line.id, "outbound");
                line.stops = Some(client.get_stops(&line.id));
                for route_section in &mut line.route_sections {
                    println!("{} Timetable", Green.bold().paint("Getting"));
                    println!("\tLine: {}", Blue.bold().paint(line.name.clone()));
                    println!("\tRoute Section: {} ...", White.bold().paint(route_section.name.clone()));
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
        let is_duplicated = if line_ids.contains(&line.id) {
            Red.paint("yes")
        } else {
            Green.paint("no")
        };

        println!("{}; Duplicate: {}", line, is_duplicated);

        for route_section in &line.route_sections {
            let has_timetable = match route_section.timetable {
                Some(ref timetable) => {
                    let names = timetable.schedule_names();
                    schedule_names = schedule_names.union(&names).cloned().collect::<HashSet<String>>();
                    names.is_empty()
                },
                None => false,
            };

            let id = route_section_id(&line, &route_section);
            println!("     {}, Has Timetable: {}, Duplicate: {}", id, has_timetable, route_section_ids.contains(&id));
            route_section_ids.insert(id.clone());
            route_section_count += 1;
        }
        line_count += 1;
        line_ids.insert(line.id.clone());
    }

    if lines.is_empty() {
        println!("No lines found in the cache, try fetching some data first");
        process::exit(0);
    }

    println!("Duplicate Lines: {}, Duplicate Route Sections: {}", line_count - line_ids.len(), route_section_count-route_section_ids.len());

    println!("Schedule Names:");
    for schedule_name in &schedule_names {
        println!("\t{}", schedule_name);
    }

    // Generate CSV files from fetched data
    write_gtfs(&lines);
}
