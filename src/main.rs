#![feature(custom_derive)]

extern crate hyper;
extern crate rustc_serialize;
extern crate scoped_threadpool;
extern crate csv;

use std::fs;
use std::sync::Arc;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::{HashSet, HashMap};

use hyper::client::{Client, Response, RequestBuilder};
use hyper::header::{Accept, Connection, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;

use scoped_threadpool::Pool;

#[derive(Clone)]
struct MyClient {
    client : Arc<Client>,
    app_id : String,
    app_key : String,
    cache_dir : String,
}

#[derive(Clone, Debug, RustcDecodable)]
struct Line {
    id : String,
    name : String,
    modeName : String,
    routeSections : Vec<RouteSection>
}

#[derive(Clone, Debug, RustcDecodable)]
struct RouteSection {
    name : String,
    direction : String,
    originator : String,
    destination : String,
    timetable : Option<TimeTable>,
}

#[derive(Clone, Debug, RustcDecodable)]
struct Interval {
    stopId : String, 
    timeToArrival: f64,
}

#[derive(Clone, Debug, RustcDecodable)]
struct StationInterval {
    id : i64,
    intervals : Vec<Interval>
}

#[derive(Clone, Debug, RustcDecodable)]
struct KnownJourney {
    intervalId : i64,
    hour : String,
    minute : String,
}

#[derive(Clone, Debug, RustcDecodable)]
struct Schedule {
    name : String,
    knownJourneys : Vec<KnownJourney>,
}

#[derive(Clone, Debug, RustcDecodable)]
struct TimeTable {
    stationIntervals : Vec<StationInterval>,
    schedules : Vec<Schedule>,
}

#[derive(Clone, Debug, RustcDecodable)]
struct RoutesTimeTables {
    routes : Vec<TimeTable>,
}

#[derive(Debug, RustcDecodable)]
struct TimeTableResponse {
    timetable : RoutesTimeTables,
}

impl MyClient {
    fn new() -> MyClient {
        let cachePath : &Path = Path::new("./cache");
        fs::create_dir(cachePath);
        return MyClient{
            client : Arc::new(Client::new()),
            app_id : String::new(),
            app_key : String::new(),
            cache_dir : String::from("./cache"),
        }
    }

    fn get(&self, endpoint : &str) -> String {
        match self.cache_get(endpoint) {
            Some(body) => body,
            None => self.remote_get(endpoint)
        }
    }

    fn remote_get(&self, endpoint : &str) -> String {
        let req_uri = format!("https://api.tfl.gov.uk{}?app_id={}&app_key={}", endpoint, self.app_id, self.app_key);
        let mut body = String::new();
        let mut resp = self.client.get(&req_uri)
            .header(Accept(vec![
                qitem(Mime(TopLevel::Application,
                    SubLevel::Ext("json".to_owned()), vec![])),
            ]))
            .send().unwrap();
        resp.read_to_string(&mut body).unwrap();
        self.cache_put(endpoint, body)
    }

    fn cache_fname(&self, endpoint : &str) -> String {
        let fname = String::from(endpoint);
        let fname0 = fname.replace("/", "_");
        self.cache_dir.clone() + "/" + &fname0
    }

    fn cache_put(&self, endpoint : &str, body : String) -> String {
        let mut f = File::create(self.cache_fname(endpoint)).unwrap();
        f.write_all(body.as_bytes());
        body
    }

    fn cache_get(&self, endpoint : &str) -> Option<String> {
        let mut body = String::new();
        match File::open(self.cache_fname(endpoint)) {
            Ok(ref mut f) => {
                f.read_to_string(&mut body);
                Some(body)
            },
            Err(_) => None,
        }
    }
}

fn get_lines(client : &MyClient) -> Vec<Line> {
    let body = client.get("/line/route");
    json::decode(&body).unwrap()
}

fn get_timetable(client : &MyClient, line_id : &str, originator: &str, destination : &str) -> Option<TimeTable> {
    let req_uri = format!("/line/{}/timetable/{}/to/{}", line_id, originator, destination);
    let body = client.get(&req_uri);
    match json::decode::<TimeTableResponse>(&body) {
        Ok(ttresp) =>  Some(ttresp.timetable.routes[0].clone()),
        Err(err) => {
            println!("Error decoding timetable {}", err);
            None
        },
    }
}

fn route_section_id(line : &Line, section : &RouteSection) -> String {
    return line.id.clone() + " " + &section.originator + " to " + &section.destination;
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
        wtr.encode(record);
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

fn write_routes(gtfs_path : &str, lines : &Vec<Line>) {
    let fname = format!("{}/{}", gtfs_path, "/routes.txt");
    let fpath = Path::new(&fname);
    let mut wtr = csv::Writer::from_file(fpath).unwrap();
    let records = vec![
        ("agency_id","agency_name","agency_url","agency_timezone"),
        ("tfl","Transport For London","https://tfl.gov.uk","Europe/London")
    ];
    wtr.encode(("route_id", "agency_id", "route_short_name", "route_long_name", "route_type"));

    for line in lines.into_iter() {
        wtr.encode((&line.id, "tfl", &line.name, "", route_type(&line)));
    }
}

fn write_stops(gtfs_path : &str, lines : &Vec<Line>) {
}

fn write_calendar(gtfs_path : &str) {
}

fn write_trips(gtfs_path : &str, lines : &Vec<Line>) {
}

fn write_stop_times(gtfs_path : &str, lines : &Vec<Line>) {
}

fn write_gtfs(lines : &Vec<Line>) {
        let gtfs_path : &Path = Path::new("./gtfs");
        let gtfs_path_str = gtfs_path.to_str().unwrap();
        fs::create_dir(gtfs_path_str);
        write_agency(gtfs_path_str);
        write_routes(gtfs_path_str, lines);
        write_stops(gtfs_path_str, lines);
        write_calendar(gtfs_path_str);
        write_trips(gtfs_path_str, lines);
        write_stop_times(gtfs_path_str, lines);
}

fn main() {
    // Fetch data
    let client = Arc::new(MyClient::new());
    let mut lines = get_lines(&client);
    let mut pool = Pool::new(10);

    pool.scoped(|scope| {
        for line in &mut lines {
            let client = client.clone();
            scope.execute(move || {
                for route_section in &mut line.routeSections {
                    println!("Getting Timetable for Line: {}, Route Section: {} ...", line.name, route_section.name);
                    route_section.timetable = get_timetable(&client, &line.id, &route_section.originator, &route_section.destination);
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
                    for schedule in &timetable.schedules {
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
