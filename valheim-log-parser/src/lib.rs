use regex::Regex;
use lazy_static::lazy_static;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

pub mod event;

lazy_static! {
    pub static ref LOG_LINE_REGEX: Regex = Regex::new(
        r#"(?P<day>\d{2}/\d{2}/\d{4})\s(?P<time>\d{2}:\d{2}:\d{2}):\s(?P<loginfo>.*)"#
    ).unwrap();

    pub static ref CHARACTER_LOCATION_REGEX: Regex = Regex::new(
        r#"Got\scharacter\sZDOID\sfrom\s(?P<charname>.*)\s:\s(?P<location>.*)$"#
    ).unwrap();
}

pub fn parse(line: &str) {
    let caps = LOG_LINE_REGEX.captures(line);
    if let Some(c) = caps {
        let day = &c["day"];
        let ts = &c["time"];
        let info = &c["loginfo"];

        let date = NaiveDate::parse_from_str(day, "%m/%d/%Y").unwrap();
        let time = NaiveTime::parse_from_str(ts, "%T").unwrap();

        let final_ts = NaiveDateTime::new(date, time);

        if let Some(event_captures) = CHARACTER_LOCATION_REGEX.captures(info) {
            println!("Character {} moved set to location {} @ {}", &event_captures["charname"], &event_captures["location"], final_ts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use std::io::{BufRead, BufReader};

    #[test]
    fn test_date_parse() {
        let test_str = "03/11/2021 19:36:10: Starting to load scene:start";
        parse(test_str);
    }

    #[test]
    fn test_parse_file() -> std::io::Result<()> {
        let f = std::fs::File::open("./example_server_logs.txt")?;
        let reader = BufReader::new(f);

        let line = reader.lines().filter_map(|l| l.ok());

        for l in line {
            parse(&l);
        }

        Ok(())

    }
}
