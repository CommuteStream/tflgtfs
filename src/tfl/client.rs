use ansi_term::Colour::Red;
use hyper::header::{Accept, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper;
use serde_json;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use tfl::line::{Line, TimeTableResponse, Sequence, Stop};

pub enum DataSource {
    API,
    Cache
}

#[derive(Clone, Default)]
pub struct Client {
    client: Arc<hyper::Client>,
    app_id: String,
    app_key: String,
    cache_dir: String,
}

impl Client {
    pub fn new() -> Client {
        let cache_path: &Path = Path::new("./cache");
        let _ = fs::create_dir(cache_path);

        Client {
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
            Some(x) => serde_json::from_str(&x).unwrap(),
            None => vec![]
        }
    }

    pub fn get_lines(&self) -> Vec<Line> {
        let body = self.get("/line/route");
        serde_json::from_str(&body).unwrap()
    }

    pub fn get_timetable(&self, line_id : &str, originator: &str, destination : &str) -> Option<TimeTableResponse> {
        let req_uri = format!("/line/{}/timetable/{}/to/{}", line_id, originator, destination);
        let body = self.get(&req_uri);
        match serde_json::from_str::<TimeTableResponse>(&body) {
            Ok(ttresp) =>  Some(ttresp.clone()),
            Err(err) => {
                println!("{}: {}", Red.bold().paint("Error decoding timetable"), err);
                None
            },
        }
    }

    pub fn get_stops(&self, line_id : &str) -> Vec<Stop> {
        let req_uri = format!("/line/{}/stoppoints", line_id);
        let body = self.get(&req_uri);
        match serde_json::from_str::<Vec<Stop>>(&body) {
            Ok(stops) => stops,
            Err(err) => {
                println!("{}: {}", Red.bold().paint("Error decoding stops"), err);
                Vec::<Stop>::new()
            }
        }
    }

    pub fn get_sequence(&self, line_id : &str, direction : &str) -> Option<Sequence> {
        let req_uri = format!("/line/{}/route/sequence/{}", line_id, direction);
        let body = self.get(&req_uri);
        match serde_json::from_str::<Sequence>(&body) {
            Ok(seq) => Some(seq),
            Err(err) => {
                println!("{}: {}", Red.bold().paint("Error decoding sequence"), err);
                None
            }
        }
    }
}
