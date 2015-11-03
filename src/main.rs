#![feature(custom_derive)]

extern crate hyper;
extern crate rustc_serialize;

use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::{HashSet, HashMap};

use hyper::client::{Client, Response, RequestBuilder};
use hyper::header::{Accept, Connection, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;

struct MyClient {
    client : Client,
    app_id : String,
    app_key : String,
    cache_dir : String,
}

#[derive(Clone, Debug, RustcEncodeable, RustcDecodable)]
struct Line {
    id : String,
    name : String,
    modeName : String,
    routeSections : Vec<RouteSection>
}

#[derive(Clone, Debug, RustcEncodeable, RustcDecodable)]
struct RouteSection {
    name : String,
    direction : String,
    originator : String,
    destination : String,
    timetable : Option<TimeTable>,
}

#[derive(Clone, Debug, RustcEncodeable, RustcDecodable)]
struct Interval {
    stopId : String, 
    timeToArrival: f64,
}

#[derive(Clone, Debug, RustcEncodeable, RustcDecodable)]
struct StationInterval {
    id : i64,
    intervals : Vec<Interval>
}

#[derive(Clone, Debug, RustcEncodeable, RustcDecodable)]
struct KnownJourney {
    intervalId : i64,
    hour : String,
    minute : String,
}

#[derive(Clone, Debug, RustcEncodeable, RustcDecodable)]
struct Schedule {
    name : String,
    knownJourneys : Vec<KnownJourney>,
}

#[derive(Clone, Debug, RustcEncodeable, RustcDecodable)]
struct TimeTable {
    stationIntervals : Vec<StationInterval>,
    schedules : Vec<Schedule>,
}

#[derive(Debug, RustcEncodeable, RustcDecodable)]
struct RoutesTimeTables {
    routes : Vec<TimeTable>,
}

#[derive(Debug, RustcEncodeable, RustcDecodable)]
struct TimeTableResponse {
    timetable : RoutesTimeTables,
}

impl MyClient {
    fn new() -> MyClient {
        let cachePath : &Path = Path::new("./cache");
        fs::create_dir(cachePath);
        return MyClient{
            client : Client::new(),
            app_id : String::new(),
            app_key : String::new(),
            cache_dir : String::from("./cache"),
        }
    }

    fn get(&mut self, endpoint : &str) -> String {
        match self.cache_get(endpoint) {
            Some(body) => body,
            None => self.remote_get(endpoint)
        }
    }

    fn remote_get(&mut self, endpoint : &str) -> String {
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

    fn cache_fname(&mut self, endpoint : &str) -> String {
        let fname = String::from(endpoint);
        let fname0 = fname.replace("/", "_");
        self.cache_dir.clone() + "/" + &fname0
    }

    fn cache_put(&mut self, endpoint : &str, body : String) -> String {
        let mut f = File::create(self.cache_fname(endpoint)).unwrap();
        f.write_all(body.as_bytes());
        body
    }

    fn cache_get(&mut self, endpoint : &str) -> Option<String> {
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

fn get_lines(client : &mut MyClient) -> Vec<Line> {
    let body = client.get("/line/route");
    json::decode(&body).unwrap()
}

fn get_timetable(client : &mut MyClient, line_id : &str, originator: &str, destination : &str) -> Option<TimeTable> {
    let req_uri = format!("/line/{}/timetable/{}/to/{}", line_id, originator, destination);
    let body = client.get(&req_uri);
    match json::decode::<TimeTableResponse>(&body) {
        Ok(ttresp) => Some(ttresp.timetable.routes[0].clone()),
        Err(_) => None,
    }
}

fn main() {
    let mut line_names : HashSet<String> = HashSet::new();
    let mut route_section_names : HashSet<String> = HashSet::new();
    let mut client = MyClient::new();
    let mut lines = get_lines(&mut client);
    for line in &mut lines {
        let line_known = line_names.contains(&line.name);
        println!("{}, Duplicate: {}:", line.name, line_known);
        line_names.insert(line.name.clone());
        for routeSection in &mut line.routeSections {
            let route_section_known = route_section_names.contains(&routeSection.name);
            println!("\t{}, Duplicate: {}", routeSection.name, route_section_known);
            route_section_names.insert(routeSection.name.clone());
            routeSection.timetable = get_timetable(&mut client, &line.id, &routeSection.originator, &routeSection.destination);
        }
    }
}
