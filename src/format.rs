use std::str::FromStr;

pub enum OutputFormat {
    GTFS,
    JSON,
}

impl FromStr for OutputFormat {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "gtfs" => Ok(OutputFormat::GTFS),
            "json" => Ok(OutputFormat::JSON),
            _     => Err("Unexpected output format")
        }
    }
}
