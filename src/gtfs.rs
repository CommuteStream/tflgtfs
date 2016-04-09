use csv;

use std::path::Path;
use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::fs;

use tfl::*;
use geometry::{linestrings_to_paths, RouteGraph, Point};

struct Route<'a> {
    line : &'a Line,
    inbound_graph : RouteGraph,
    outbound_graph : RouteGraph,
}

impl<'a> Route<'a> {
    fn new(line : &'a Line) -> Route {
        let inbound_paths = match line.inbound_sequence.as_ref() {
            Some(ref seq) => linestrings_to_paths(&seq.lineStrings),
            None => vec![],
        };
        let outbound_paths = match line.outbound_sequence.as_ref() {
            Some(ref seq) => linestrings_to_paths(&seq.lineStrings),
            None => vec![],
        };
        let mut inbound_graph = RouteGraph::new();
        let mut outbound_graph = RouteGraph::new();
        inbound_graph.add_paths(&inbound_paths);
        outbound_graph.add_paths(&outbound_paths);
        Route {
            line : line,
            inbound_graph : inbound_graph,
            outbound_graph : outbound_graph,
        }
    }
}

fn route_type(line : &Line) -> &'static str {
    match &line.modeName[..] {
        "dlr" | "tram" => "0",
        "tube" | "overground" => "1",
        "national-rail" | "tflrail" => "2",
        "bus" => "3",
        "river-tour" | "river-bus" => "4",
        "cable-car" => "5",
        _ => {
            println!("Missing line modeName match: {}", line.modeName);
            ""
        },
    }
}

fn write_agency(gtfs_path : &str) {
    let fname = format!("{}/{}", gtfs_path, "/agency.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    let records = vec![
        ("agency_id","agency_name","agency_url","agency_timezone"),
        ("tfl","Transport For London","https://tfl.gov.uk","Europe/London")
    ];
    for record in records {
        wtr.encode(record).unwrap();
    }
}

fn write_routes(gtfs_path : &str, routes : &Vec<Route>) {
    let fname = format!("{}/{}", gtfs_path, "/routes.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    wtr.encode(("route_id", "agency_id", "route_color", "route_short_name", "route_long_name", "route_type")).unwrap();
    for route in routes {
        let line = &route.line;
        let line_color = line.color();
        wtr.encode((&line.id, "tfl", &line_color, &line.name, "", route_type(&line))).unwrap();
    }
}

fn write_stops(gtfs_path : &str, routes : &Vec<Route>) -> HashMap<String, (f64, f64)> {
    let fname = format!("{}/{}", gtfs_path, "/stops.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    let mut written_stops = HashMap::<String, (f64, f64)>::new();
    wtr.encode(("stop_id", "stop_name", "stop_lat", "stop_lon")).unwrap();
    for route in routes {
        let stops = route.line.stops.as_ref().unwrap();
        for stop in stops {
            match written_stops.contains_key(&stop.naptanId) {
                true => (),
                false => {
                    wtr.encode((stop.naptanId.clone(), stop.commonName.clone(), stop.lat, stop.lon)).unwrap();
                    written_stops.insert(stop.naptanId.clone(), (stop.lat, stop.lon));
                    for child in &stop.children {
                        match written_stops.contains_key(&child.naptanId) {
                            true => (),
                            false => {
                                wtr.encode((child.naptanId.clone(), child.commonName.clone(), stop.lat, stop.lon)).unwrap();
                                written_stops.insert(child.naptanId.clone(), (stop.lat, stop.lon));
                            },
                        }
                    }
                },
            };
        }
        for section in &route.line.routeSections {
            match section.timetable {
                Some(ref timetable) => {
                    for station in &timetable.stations {
                        match written_stops.contains_key(&station.id) {
                            true => (),
                            false => {
                                wtr.encode((station.id.clone(), station.name.clone(), station.lat, station.lon)).unwrap();
                                written_stops.insert(station.id.clone(), (station.lat, station.lon));
                            },
                        }
                    }
                    for stop in &timetable.stops {
                        match written_stops.contains_key(&stop.id) {
                            true => (),
                            false => {
                                wtr.encode((stop.id.clone(), stop.name.clone(), stop.lat, stop.lon)).unwrap();
                                written_stops.insert(stop.id.clone(), (stop.lat, stop.lon));
                            },
                        }
                    }
                },
                None => (),
            }
        }
    }
    written_stops
}

fn write_calendar(gtfs_path : &str) {
    let fname = format!("{}/{}", gtfs_path, "/calendar.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    let start_date = "20151031";
    let end_date = "20161031";
    let records = vec![
        ("service_id", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday", "start_date", "end_date"),
        ("School Monday", "1", "0", "0", "0", "0", "0", "0", &start_date, &end_date),
        ("Sunday Night/Monday Morning", "1", "0", "0", "0", "0", "0", "1", &start_date, &end_date),
        ("School Monday, Tuesday, Thursday & Friday", "1", "1", "0", "1", "1", "0", "0", &start_date, &end_date),
        ("Tuesday", "0", "1", "0", "0", "0", "0", "0", &start_date, &end_date),
        ("Monday - Thursday", "1", "1", "1", "1", "0", "0", "0", &start_date, &end_date),
        ("Saturday", "0", "0", "0", "0", "0", "0", "1", &start_date, &end_date),
        ("Saturday and Sunday", "0", "0", "0", "0", "0", "1","1", &start_date, &end_date),
        ("Sunday", "0", "0", "0", "0", "0", "0", "1", &start_date, &end_date),
        ("School Tuesday", "0", "1", "0", "0", "0", "0", "0", &start_date, &end_date),
        ("Saturday Night/Sunday Morning", "0", "0", "0", "0", "0", "1", "1", &start_date, &end_date),
        ("Mo-Fr Night/Tu-Sat Morning", "1", "1", "1", "1","1", "1", "0", &start_date, &end_date),
        ("Monday to Thursday", "1", "1", "1", "1", "0", "0", "0", &start_date, &end_date),
        ("Mo-Th Nights/Tu-Fr Morning", "1", "1", "1", "1", "1", "0", "0", &start_date, &end_date),
        ("Saturday (also Good Friday)", "0", "0", "0", "0", "0", "1", "0", &start_date, &end_date),
        ("Mon-Th Schooldays", "1", "1", "1", "1", "0", "0", "0", &start_date, &end_date),
        ("Saturdays and Public Holidays", "0", "0", "0", "0", "0", "1", "0", &start_date, &end_date),
        ("Friday Night/Saturday Morning", "0", "0", "0", "0", "1", "1", "0", &start_date, &end_date),
        ("Friday", "0", "0", "0", "0", "1", "0", "0", &start_date, &end_date),
        ("Thursdays", "0", "0", "0", "1", "0", "0", "0", &start_date, &end_date),
        ("Sunday night/Monday morning - Thursday night/Friday morning", "1", "1", "1", "1", "1", "0", "1", &start_date, &end_date),
        ("School Thursday", "0", "0", "0", "1", "0", "0", "0", &start_date, &end_date),
        ("School Friday", "0", "0", "0", "0", "1", "0", "0", &start_date, &end_date),
        ("Daily", "1", "1", "1", "1", "1", "1", "1", &start_date, &end_date),
        ("Tuesday, Wednesday & Thursday", "0", "1", "1", "1", "0", "0", "0", &start_date, &end_date),
        ("Mon-Fri Schooldays", "1", "1", "1", "1", "1", "0", "0", &start_date, &end_date),
        ("Wednesday", "0", "0", "1", "0", "0", "0", "0", &start_date, &end_date),
        ("Monday, Tuesday and Thursday", "1", "1", "0", "1", "0", "0", "0", &start_date, &end_date),
        ("Wednesdays", "0", "0", "1", "0", "0", "0", "0", &start_date, &end_date),
        ("Monday to Friday", "1", "1", "1", "1", "1", "0", "0", &start_date, &end_date),
        ("Monday", "1", "0", "0", "0", "0", "0", "0", &start_date, &end_date),
        ("Sunday and other Public Holidays", "0", "0", "0", "0", "0", "0", "1", &start_date, &end_date),
        ("School Wednesday", "0", "0", "1", "0", "0", "0", "0", &start_date, &end_date),
        ("Monday - Friday", "1", "1", "1", "1", "1", "0", "0", &start_date, &end_date),
        ];
    for record in records {
        wtr.encode(record).unwrap();
    }
}

fn trip_id(line : &Line, section : &RouteSection, schedule : &Schedule, journey : &KnownJourney) -> String {
    let tfmt = time_offset_fmt(journey, 0.0);
    format!("{} {} to {} scheduled {} departs {}", line.id, section.originator, section.destination, schedule.name, tfmt)
}

fn write_route_section_trips(wtr : &mut csv::Writer<File>, shape_id : &String, line : &Line, section : &RouteSection) {
    let mut written_trips : HashSet<String> = HashSet::new();
    let direction = match &section.direction[..] {
        "inbound" => "1".to_owned(),
        "outbound" => "0".to_owned(),
        _ => "".to_owned(),
    };
    match section.timetable.as_ref() {
        None => (),
        Some(timetable) => {
            let first: Option<&TimeTable> = timetable.first_timetable();

            match first {
                None => (),
                Some(ref x) => {
                    for schedule in &x.schedules {
                        for journey in &schedule.knownJourneys {
                            let id = trip_id(line, section, schedule, journey);
                            match written_trips.contains(&id) {
                                true => (),
                                false => {
                                    written_trips.insert(id.clone());
                                    wtr.encode((&line.id, &schedule.name, &id, &direction, &shape_id)).unwrap();
                                },
                            }
                        }
                    }
                }
            }
        },
    }
}

pub fn route_section_id(line : &Line, section : &RouteSection) -> String {
    return line.id.clone() + " " + &section.originator + " to " + &section.destination;
}

fn write_trips(gtfs_path : &str, routes : &Vec<Route>) {
    let fname = format!("{}/{}", gtfs_path, "/trips.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    wtr.encode(("route_id", "service_id", "trip_id", "direction", "shape_id")).unwrap();
    for route in routes {
        let mut written_route_sections = HashSet::<String>::new();
        let route_sections = &route.line.routeSections;
        for route_section in route_sections {
            let id = route_section_id(route.line, route_section);
            match written_route_sections.contains(&id) {
                true => (),
                false => {
                    write_route_section_trips(&mut wtr, &id, route.line, route_section);
                    written_route_sections.insert(id);
                },
            };
        }
    }
}
fn time_offset_fmt(journey : &KnownJourney, offset : f64) -> String {
    let dep_hour : u64 = journey.hour.parse().unwrap();
    let dep_minute : u64 = journey.minute.parse().unwrap();
    let rounded_offset : u64 = offset.floor() as u64;
    let minute_offset : u64 = dep_minute + rounded_offset;
    let hour : u64 = dep_hour + minute_offset / 60;
    let minute : u64 = minute_offset % 60;
    format!("{:02}:{:02}", hour, minute)
}

fn write_journey_stop_times(wtr : &mut csv::Writer<File>, line : &Line, section : &RouteSection, schedule : &Schedule, journey : &KnownJourney, interval : &StationInterval) {
    let mut stop_seq = 1;
    let trip_id = trip_id(line, section, schedule, journey);
    let dep_time = time_offset_fmt(journey, 0.0);
    wtr.encode((&trip_id, &section.originator, stop_seq, &dep_time, &dep_time)).unwrap();
    for stop in &interval.intervals {
        stop_seq += 1;
        let dep_time = time_offset_fmt(journey, stop.timeToArrival);
        wtr.encode((&trip_id, &stop.stopId, stop_seq, &dep_time, &dep_time)).unwrap();
    }
}

fn intervals<'a>(station_intervals: &'a Vec<StationInterval>) -> HashMap<i64, &'a StationInterval> {
    station_intervals.iter().map(|x| (x.id, x)).collect()
}

fn write_route_section_stop_times(wtr : &mut csv::Writer<File>, line : &Line, section : &RouteSection) {
    match section.timetable.as_ref() {
        None => (),
        Some(timetable) => {
            let mut written_trips : HashSet<String> = HashSet::new();
            let record: Option<&TimeTable> = timetable.first_timetable();

            if let Some(ref datum) = record {
                let intervals = intervals(&datum.stationIntervals);
                for schedule in &datum.schedules {
                    for journey in &schedule.knownJourneys {
                        intervals.get(&journey.intervalId)
                                 .map(|interval| {
                                    let id = trip_id(line, section, schedule, journey);

                                    if !written_trips.contains(&id) {
                                        written_trips.insert(id.clone());
                                        write_journey_stop_times(wtr, line, section, schedule, journey, interval);
                                    }
                                 })
                                 .unwrap_or_else(|| { println!("Error, Could not find interval for schedule!!!!"); });
                    }
                }
            }
        },
    }
}

fn write_stop_times(gtfs_path : &str, routes : &Vec<Route>) {
    let fname = format!("{}/{}", gtfs_path, "/stop_times.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    wtr.encode(("trip_id", "stop_id", "stop_sequence", "arrival_time", "departure_time")).unwrap();
    for route in routes {
        let mut written_route_sections = HashSet::<String>::new();
        let route_sections = &route.line.routeSections;
        for route_section in route_sections {
            let id = route_section_id(route.line, route_section);
            match written_route_sections.contains(&id) {
                true => (),
                false => {
                    write_route_section_stop_times(&mut wtr, route.line, route_section);
                    written_route_sections.insert(id);
                },
            };
        }
    }
}

fn write_shape_path(wtr : &mut csv::Writer<File>, shape_id : &String, path : &Vec<Point>) {
    let mut seq = 0;
    for pt in path {
        wtr.encode((shape_id, pt.lat(), pt.lon(), seq)).unwrap();
        seq += 1;
    }
}

fn write_shape(wtr : &mut csv::Writer<File>, shape_id : &String, route: &Route, section : &RouteSection, stops : &HashMap<String, (f64, f64)>, graph : &RouteGraph) {
    match stops.get(&section.originator) {
        Some(&(start_lat, start_lon)) => {
            let start_pt = Point::new(start_lat, start_lon);
            match stops.get(&section.destination) {
                Some(&(end_lat, end_lon)) => {
                    let end_pt = Point::new(end_lat, end_lon);
                    match graph.path(start_pt, end_pt) {
                        Some(path) => write_shape_path(wtr, shape_id, &path),
                        None => {
                            println!("could not find shape for {}!!!", shape_id);
                        },
                    }
                },
                None => (),
            }
        },
        None => (),
    }
}

fn write_shapes(gtfs_path : &str, routes : &Vec<Route>, stops : &HashMap<String, (f64, f64)>) {
    let fname = format!("{}/{}", gtfs_path, "/shapes.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    wtr.encode(("shape_id", "shape_pt_lat", "shape_pt_lon", "shape_pt_sequence")).unwrap();
    for route in routes {
        let mut written_shapes = HashSet::<String>::new();
        let route_sections = &route.line.routeSections;
        for route_section in route_sections {
            let shape_id = route_section_id(route.line, route_section);
            match written_shapes.contains(&shape_id) {
                true => (),
                false => {
                    let graph = match &route_section.direction[..] {
                        "inbound" => Some(&route.inbound_graph),
                        "outbound" => Some(&route.outbound_graph),
                        _ => None,
                    };
                    match graph {
                        Some(graph) => {
                            write_shape(&mut wtr, &shape_id, route, route_section, stops, graph);
                            written_shapes.insert(shape_id);
                        },
                        None => (),
                    }
                },
            };
        }
    }
}

pub fn write_gtfs(lines : &Vec<Line>) {

    let routes = lines.iter().map(|line| Route::new(line)).collect();
    let gtfs_path : &Path = Path::new("./gtfs");
    let gtfs_path_str = gtfs_path.to_str().unwrap();
    let _ = fs::create_dir(gtfs_path_str);
    write_agency(gtfs_path_str);
    write_routes(gtfs_path_str, &routes);
    let all_stops = write_stops(gtfs_path_str, &routes);
    write_calendar(gtfs_path_str);
    write_trips(gtfs_path_str, &routes);
    write_stop_times(gtfs_path_str, &routes);
    write_shapes(gtfs_path_str, &routes, &all_stops);
}

