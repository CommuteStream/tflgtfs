use ansi_term::Colour::{Green, Blue, Red};
use std::fmt;
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize)]
pub struct Line {
    pub id: String,
    pub name: String,
    #[serde(rename="modeName")]
    pub mode_name: String,
    #[serde(rename="routeSections")]
    pub route_sections: Vec<RouteSection>,
    pub stops: Option<Vec<Stop>>,
    pub inbound_sequence: Option<Sequence>,
    pub outbound_sequence: Option<Sequence>,
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
            "Tram 1" | "Tram 2" => "C6D834",
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
        match &self.mode_name[..] {
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

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id: &str = &self.id;
        write!(f, "{} {}", Green.bold().paint("Line"), Blue.bold().paint(id))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Stop {
    #[serde(rename="naptanId")]
    pub naptan_id: String,
    #[serde(rename="commonName")]
    pub common_name: String,
    pub lat: f64,
    pub lon: f64,
    pub children: Vec<Stop>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RouteSection {
    pub name: String,
    pub direction: String,
    pub originator: String,
    pub destination: String,
    pub timetable: Option<TimeTableResponse>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Interval {
    #[serde(rename="stopId")]
    pub stop_id: String,
    #[serde(rename="timeToArrival")]
    pub time_to_arrival: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StationInterval {
    pub id: i64,
    pub intervals: Vec<Interval>
}

#[derive(Clone, Debug, Deserialize)]
pub struct KnownJourney {
    #[serde(rename="intervalId")]
    pub interval_id: i64,
    pub hour: String,
    pub minute: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Schedule {
    pub name: String,
    #[serde(rename="knownJourneys")]
    pub known_journeys: Vec<KnownJourney>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTable {
    #[serde(rename="stationIntervals")]
    pub station_intervals: Vec<StationInterval>,
    pub schedules: Vec<Schedule>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RoutesTimeTables {
    pub routes: Vec<TimeTable>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TimeTableResponse {
    pub stations: Vec<Station>,
    pub stops: Vec<Station>,
    pub timetable: RoutesTimeTables,
    #[serde(rename="statusErrorMessage")]
    pub status_error_message: Option<String>,
    #[serde(rename="lineId")]
    line_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Sequence {
    #[serde(rename="lineStrings")]
    pub line_strings: Vec<String>,
}

impl TimeTableResponse {
    pub fn first_timetable(&self) -> Option<&TimeTable> {
        if let Some(ref message) = self.status_error_message {
            println!("{} (line {}): {}", Red.bold().paint("Error"), Blue.bold().paint(self.line_id.clone()), message);
            None
        } else {
            Some(&self.timetable.routes[0])
        }
    }

    pub fn schedule_names(&self) -> HashSet<String> {
        if let Some(record) = *&self.first_timetable() {
            return record.schedules.iter()
                                   .map(|x| x.name.clone())
                                   .collect();
        }

        HashSet::new()
    }
}
