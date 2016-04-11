use std::str::FromStr;

#[derive(Debug)]
pub enum OutputFormat {
    GTFS,
    JSON,
    None,
}

impl FromStr for OutputFormat {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "gtfs" => Ok(OutputFormat::GTFS),
            "json" => Ok(OutputFormat::JSON),
            "none" => Ok(OutputFormat::None),
            _     => Err("Unexpected output format")
        }
    }
}
