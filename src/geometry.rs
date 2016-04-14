use serde_json;
use std::collections::{HashSet, HashMap};
use std::f64::consts::PI;
use std::fmt;


const PRECISION: f64 = 10000.0;

/// Point containing latitude and longitude values as integers.
/// Integers are used here due to Rust itself not providing some basic
/// floating point functionality at the moment. All due to the undefined
/// behavior revolving around `NaN` float values. For example, what is the boolean
/// result of `0.0 > NaN` ? Its undefined, so therefore > is not implemented by
/// Rust Proper! Perhaps they'll fix this massive inconvienence in the future.
/// TODO use floats
#[derive(PartialEq, Eq, Clone, Debug, Hash, Copy)]
pub struct Point {
    lat: i64,
    lon: i64,
}

/// Degress to Radians
fn deg2rad(deg: f64) -> f64 {
    ((2.0 * PI) / 180.0) * deg
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.lat(), self.lon())
    }
}

impl Point {
    /// New integer stored Point from floating point lat/lon coordinates
    pub fn new(lat: f64, lon: f64) -> Point {
        Point {
            lat: (lat * PRECISION).floor() as i64,
            lon: (lon * PRECISION).floor() as i64,
        }
    }

    /// Latitude value
    pub fn lat(&self) -> f64 {
        (self.lat as f64) / PRECISION
    }

    /// Longitude value
    pub fn lon(&self) -> f64 {
        (self.lon as f64) / PRECISION
    }

    /// Spheroid distance calculation given earth coordinates as lat/lon values.
    /// Returns the distance in meters.
    pub fn geo_distance(&self, p: &Point) -> f64 {
        let r = 6371000.0; // metres
        let lat1 = p.lat();
        let lon1 = p.lon();
        let lat2 = self.lat();
        let lon2 = self.lon();
        let sig1 = deg2rad(lat1);
        let sig2 = deg2rad(lat2);
        let deltasig = deg2rad(lat2 - lat1);
        let deltalambda = deg2rad(lon2 - lon1);
        let a = (deltasig / 2.0).sin() * (deltasig / 2.0).sin() +
            sig1.cos() * sig2.cos() *
            (deltalambda / 2.0).sin() * (deltalambda / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

}

/// Path
pub type Path = Vec<Point>;

/// Maintains a sparse routing graph
#[derive(Default)]
pub struct RouteGraph {
    vertices: HashSet<Point>,
    edges: HashMap<Point, Vec<Point>>,
    paths: HashMap<(Point, Point), Path>,
}

/// Convert the TFL lineStrings attribute to a simple flat vectory of paths.
/// lineStrings in TFL data is a JSON array of string values, containing
/// either an array of points or an array of arrays of points, we handle both.
pub fn linestrings_to_paths(line_strings: &[String]) -> Vec<Path> {
    let mut paths : Vec<Path> = Vec::new();
    for line_string in line_strings {
        match serde_json::from_str::<Vec<(f64, f64)>>(&line_string) {
            Ok(raw_path) => paths.push(raw_path.iter().map(|&(lon, lat)| Point::new(lat, lon)).collect()),
            Err(err) => {
                match serde_json::from_str::<Vec<Vec<(f64, f64)>>>(&line_string) {
                    Ok(raw_paths) => {
                        for raw_path in raw_paths {
                            paths.push(raw_path.iter().map(|&(lon, lat)| Point::new(lat, lon)).collect());
                        }
                    },
                    Err(err2) => println!("Errors decoding line string, single line {}, multi line {}", err, err2),
                }
            },
        }
    }
    paths
}

impl RouteGraph {
    /// New Route Graph
    pub fn new() -> RouteGraph {
        RouteGraph {
            vertices: HashSet::new(),
            edges: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// Add many paths
    pub fn add_paths(&mut self, paths: &[Path]) {
        for path in paths {
            self.add_path(path);
        }
    }

    /// Add single path to the graph
    pub fn add_path(&mut self, path: &Path) {
        // add points
        let first = path.first().unwrap();
        let last = path.last().unwrap();
        self.vertices.insert(*first);
        self.vertices.insert(*last);

        // add bidirectional edges
        self.edges.entry(*first).or_insert_with(|| vec![*last]).push(*last);
        self.edges.entry(*last).or_insert_with(|| vec![*first]).push(*first);

        // add paths
        self.paths.insert((*first, *last), path.clone());
        self.paths.insert((*last, *first), path.iter().rev().cloned().collect());
        assert!(self.paths.contains_key(&(first.clone(), last.clone())));
        assert!(self.paths.contains_key(&(last.clone(), first.clone())));
    }

    /// Find the closest actual point (vertex) in our graph since they are not
    /// going to be exact matches
    fn closest_point(&self, pt: &Point) -> (Point, f64) {
        let far_pt = Point::new(pt.lat() + 90.0, pt.lon() + 90.0);
        let far_dist = far_pt.geo_distance(pt);
        let (min_pt, min_dist) = self.edges.keys().fold((&far_pt, far_dist), |(min_pt, min_dist), vert| {
            let vert_dist = vert.geo_distance(pt);
            if vert_dist < min_dist {
                (vert, vert_dist)
            } else {
                (min_pt, min_dist)
            }
        });

        (*min_pt, min_dist)
    }

    /// Find a path between two points if one exists
    pub fn path(&self, p0: Point, p1: Point) -> Option<Path> {
        let (start, start_dist) = self.closest_point(&p0);
        if start_dist > 2000.0 {
            println!("start point {} to closest {} distance is {} > 2000", p0, start, start_dist);
            return None;
        }
        let (end, end_dist) = self.closest_point(&p1);
        if end_dist > 2000.0 {
            println!("end point {} to closest {} distance is {} > 2000", p1, end, end_dist);
            return None;
        }
        let visited: HashSet<Point> = HashSet::new();
        self.find_path(start, end, &visited)
    }

    // Recursively search the path graph from p0 to p1, when a path is found
    // the callstack will inherently build up a single Path containing all the
    // points between the two.
    // Visited is a simple HashSet marking points we've already visited to avoid
    // infinitely recursing in circles.
    fn find_path(&self, p0: Point, p1: Point, visited: &HashSet<Point>) -> Option<Path> {
        match self.edges.get(&p0) {
            Some(to_vertices) => {
                if visited.contains(&p0) {
                    None
                } else {
                    for next in to_vertices {
                        if *next == p1 {
                            return Some(self.paths.get(&(p0, *next)).unwrap().clone());
                        }

                        let mut visited0 = visited.clone();
                        visited0.insert(*next);

                        if let Some(path) = self.find_path(*next, p1, &visited0) {
                            return Some(vec![&(self.paths.get(&(p0, *next)).unwrap())[..], &path[..]].concat());
                        }
                    }

                    None
                }
            },
            None => None,
        }
    }
}
