use hyper;
use hyper::header::{Accept, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use std::path::Path;
use std::io::{Read, Write};
use std::sync::Arc;
use std::fs;

use std::collections::HashSet;

use rustc_serialize::json;

pub enum DataSource {
    API,
    Cache
}

#[derive(Clone)]
pub struct Client {
    client : Arc<hyper::Client>,
    app_id : String,
    app_key : String,
    cache_dir : String,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Line {
    pub id : String,
    pub name : String,
    pub modeName : String,
    pub routeSections : Vec<RouteSection>,
    pub stops : Option<Vec<Stop>>,
    pub inbound_sequence : Option<Sequence>,
    pub outbound_sequence : Option<Sequence>,
}

/// Default color string, use null so the importer can choose
const DEFAULT_COLOR : &'static str = "";

impl Line {
    /// Tube Color
    fn tube_color(&self) -> &str {
        match &self.name[..] {
            "Bakerloo" => "894E24",
            "Central" => "DC241F",
            "Circle" => "FFCE00",
            "District" => "007229",
            "Hammersmith & City" => "D799AF",
            "Jubilee" => "6A7278",
            "Metropolitan" => "751056",
            "Northern" => "000",
            "Piccadilly" => "0019A8",
            "Victoria" => "00A0E2",
            "Waterloo & City" => "76D0BD",
            _ => {
                println!("Missing tube color for {}", self.name);
                DEFAULT_COLOR
            },
        }
    }

    /// Tram Color
    fn tram_color(&self) -> &str {
        match &self.name[..] {
            "Tram 1" => "C6D834",
            "Tram 2" => "C6D834",
            "Tram 3" => "79C23F",
            "Tram 4" => "336B14",
            _ => {
                println!("Missing tram color for {}", self.name);
                DEFAULT_COLOR
            },
        }
    }

    /// National Rail Color
    fn national_rail_color(&self) -> &str {
        match &self.name[..] {
            "South West Trains" => "F11815",
            "Southeastern" => "0071BF",
            "Southern" => "00A74B",
            "Great Northern" => "00A6E2",
            "Arriva Trains Wales" => "00B9B4",
            "c2c" => "F0188C",
            "Chiltern Railways" => "B389C1",
            "Cross Country" => "A03467",
            "East Midlands Trains" => "E16C16",
            "First Great Western" => "2D2B94",
            "First Hull Trains" => "1B903F",
            "First TransPennine Express" => "F265A0",
            "Gatwick Express" => "231F20",
            "Grand Central" => "3F3F40",
            "Greater Anglia" => "8B8FA5",
            "Heathrow Connect" => "F6858D",
            "Heathrow Express" => "55C4BF",
            "Island Line" => "F8B174",
            "London Midland" => "8BC831",
            "Merseyrail" => "FEC95F",
            "Northern Rail" => "0569A8",
            "ScotRail" => "96A3A9",
            "Thameslink" => "DA4290",
            "Virgin Trains" => "A8652C",
            "Virgin Trains East Coast" => "9C0101",
            _ => {
                println!("Missing national rail color for {}", self.name);
                DEFAULT_COLOR
            },
        }
    }

    /// River Bus Color
    fn river_bus_color(&self) -> &str {
        match &self.name[..] {
            "RB1" => "2D3039",
            "RB2" => "0072BC",
            "RB4" => "61C29D",
            "RB5" => "BA6830",
            "RB6" => "DF64B0",
            "Woolwich Ferry" => "F7931D",
            _ => {
                println!("Missing rail color for {}", self.name);
                DEFAULT_COLOR
            },
        }
    }

    fn cable_car_color(&self) -> &str {
        match &self.name[..] {
            "Emirates Air Line" => "E51937",
            _ => {
                println!("Missing rail color for {}", self.name);
                DEFAULT_COLOR
            },
        }
    }

    /// The Line's Color based on the TFL colors on tfl.gov.uk
    pub fn color(&self) -> &str {
        match &self.modeName[..] {
            "dlr" => "00AFAD",
            "overground" => "E86A10",
            "tflrail" => "0019A8",
            "tube" => self.tube_color(),
            "tram" => self.tram_color(),
            "national-rail" => self.national_rail_color(),
            "river-bus" | "river-ferry" => self.river_bus_color(),
            "cable-car" => self.cable_car_color(),
            _ => DEFAULT_COLOR,
        }
    }
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Stop {
    pub naptanId : String,
    pub commonName : String,
    pub lat : f64,
    pub lon : f64,
    pub children : Vec<Stop>,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct RouteSection {
    pub name : String,
    pub direction : String,
    pub originator : String,
    pub destination : String,
    pub timetable : Option<TimeTableResponse>,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Interval {
    pub stopId : String,
    pub timeToArrival: f64,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct StationInterval {
    pub id : i64,
    pub intervals : Vec<Interval>
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct KnownJourney {
    pub intervalId : i64,
    pub hour : String,
    pub minute : String,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Schedule {
    pub name : String,
    pub knownJourneys : Vec<KnownJourney>,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct TimeTable {
    pub stationIntervals : Vec<StationInterval>,
    pub schedules : Vec<Schedule>,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct RoutesTimeTables {
    pub routes : Vec<TimeTable>,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Station {
    pub id : String,
    pub name : String,
    pub lat : f64,
    pub lon : f64,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct TimeTableResponse {
    pub stations : Vec<Station>,
    pub stops : Vec<Station>,
    pub timetable : RoutesTimeTables,
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct Sequence {
    pub lineStrings : Vec<String>,
}

impl TimeTableResponse {
   pub fn first_timetable(&self) -> Option<&TimeTable> {
        match self.timetable.routes.len() > 0 {
            true => Some(&self.timetable.routes[0]),
            false => None,
        }
    }
}

pub fn collect_schedule_names(timetable: &TimeTableResponse) -> HashSet<String> {
    let mut schedule_names: HashSet<String> = HashSet::new();
    let record: Option<&TimeTable> = timetable.first_timetable();

    match record {
        None => schedule_names,
        Some(ref datum) => {
            for schedule in &datum.schedules {
                schedule_names.insert(schedule.name.clone());
            }
            schedule_names
        }
    }
}

impl Client {
    pub fn new() -> Client {
        let cache_path : &Path = Path::new("./cache");
        let _ = fs::create_dir(cache_path);
        return Client{
            client : Arc::new(hyper::Client::new()),
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
        let mut f = fs::File::create(self.cache_fname(endpoint)).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        body
    }

    fn cache_get(&self, endpoint : &str) -> Option<String> {
        let mut body = String::new();
        match fs::File::open(self.cache_fname(endpoint)) {
            Ok(ref mut f) => {
                f.read_to_string(&mut body).unwrap();
                Some(body)
            },
            Err(_) => None,
        }
    }

    pub fn get_cached_lines(&self) -> Vec<Line> {
        let body = self.cache_get("/line/route");
        match body {
            Some(x) => json::decode(&x).unwrap(),
            None => vec![]
        }
    }

    pub fn get_lines(&self) -> Vec<Line> {
        let body = self.get("/line/route");
        json::decode(&body).unwrap()
    }

    pub fn get_timetable(&self, line_id : &str, originator: &str, destination : &str) -> Option<TimeTableResponse> {
        let req_uri = format!("/line/{}/timetable/{}/to/{}", line_id, originator, destination);
        let body = self.get(&req_uri);
        match json::decode::<TimeTableResponse>(&body) {
            Ok(ttresp) =>  Some(ttresp.clone()),
            Err(err) => {
                println!("Error decoding timetable {}", err);
                None
            },
        }
    }

    pub fn get_stops(&self, line_id : &str) -> Vec<Stop> {
        let req_uri = format!("/line/{}/stoppoints", line_id);
        let body = self.get(&req_uri);
        match json::decode::<Vec<Stop>>(&body) {
            Ok(stops) => stops,
            Err(err) => {
                println!("Error decoding stops: {}", err);
                Vec::<Stop>::new()
            }
        }
    }

    pub fn get_sequence(&self, line_id : &str, direction : &str) -> Option<Sequence> {
        let req_uri = format!("/line/{}/route/sequence/{}", line_id, direction);
        let body = self.get(&req_uri);
        match json::decode::<Sequence>(&body) {
            Ok(seq) => Some(seq),
            Err(err) => {
                println!("Error decoding sequence: {}", err);
                None
            }
        }
    }
}


