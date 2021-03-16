use regex::Regex;
use lazy_static::lazy_static;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

pub mod event;
pub use event::{Event, SpawnData, EventData, SaveData, ConnectionData};

lazy_static! {
    pub static ref LOG_LINE_REGEX: Regex = Regex::new(
        r#"(?P<day>\d{2}/\d{2}/\d{4})\s(?P<time>\d{2}:\d{2}:\d{2}):\s(?P<loginfo>.*)"#
    ).unwrap();

    pub static ref CHARACTER_LOCATION_REGEX: Regex = Regex::new(
        r#"Got\scharacter\sZDOID\sfrom\s(?P<charname>.*)\s:\s(?P<location>.*)$"#
    ).unwrap();

    pub static ref WORLD_SAVE_REGEX: Regex = Regex::new(
        r#"World\ssaved\s\(\s(?P<timing>.+)ms\s\)"#
    ).unwrap();

    pub static ref USER_CONNECTED_REGEX: Regex = Regex::new(
        r#"Got\sconnection\sSteamID\s(?P<steamid>\d+)$"#
    ).unwrap();

    pub static ref USER_DISCONNECTED_REGEX: Regex = Regex::new(
        r#"Closing\ssocket\s(?P<steamid>\d+)$"#
    ).unwrap();

    pub static ref WRONG_PASSWORD_REGEX: Regex = Regex::new(
        r#"Peer\s(?P<steamid>\d+)\shas\swrong\spassword$"#
    ).unwrap();
}

// TODO: model errors w/ thiserror
pub fn parse(line: &str) -> Option<Event> {
    let caps = LOG_LINE_REGEX.captures(line);
    if let Some(c) = caps {
        let day = &c["day"];
        let ts = &c["time"];
        let info = &c["loginfo"];

        let date = NaiveDate::parse_from_str(day, "%m/%d/%Y").unwrap();
        let time = NaiveTime::parse_from_str(ts, "%T").unwrap();

        let timestamp = NaiveDateTime::new(date, time);

        if let Some(event_captures) = CHARACTER_LOCATION_REGEX.captures(info) {
            let character = String::from(&event_captures["charname"]);
            let coords: Vec<&str> = event_captures["location"].split(":").collect();
            let x: i64 = coords[0].parse().unwrap();
            let y: i64 = coords[1].parse().unwrap();
            let location = (x,y);

            let ev = SpawnData { timestamp, character, location };

            if x == 0 && y == 0 {
                return Some(Event::CharacterDied(ev));
            } else {
                return Some(Event::CharacterSpawned(ev));
            }
        }

        if let Some(save) = WORLD_SAVE_REGEX.captures(info) {
            let save_time: f64 = save["timing"].parse().unwrap();
            return Some(Event::WorldSaved(SaveData{ timestamp, time_spent: save_time }));
        }

        if let Some(connect) = USER_CONNECTED_REGEX.captures(info) {
            let steam_id: u64 = connect["steamid"].parse().unwrap();
            return Some(Event::UserConnected(ConnectionData{ timestamp, steam_id }));
        }

        if let Some(disconnect) = USER_DISCONNECTED_REGEX.captures(info) {
            let steam_id: u64 = disconnect["steamid"].parse().unwrap();
            return Some(Event::UserDisconnected(ConnectionData{ timestamp, steam_id }));
        }

        if let Some(wrong) = WRONG_PASSWORD_REGEX.captures(info) {
            let steam_id: u64 = wrong["steamid"].parse().unwrap();
            return Some(Event::IncorrectPasswordGiven(ConnectionData{ timestamp, steam_id}))
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::Event;
    use std::io::{BufRead, BufReader};

    #[test]
    fn test_date_parse() {
        let test_str = "03/11/2021 19:36:10: Starting to load scene:start";
        parse(test_str);
    }

    #[test]
    fn test_parse_connection() {
        let test_str = "03/11/2021 19:47:02: Got connection SteamID 76561199036446150";
        assert!(parse(test_str).is_some());
    }

    #[test]
    fn test_parse_file() -> std::io::Result<()> {
        let f = std::fs::File::open("./example_server_logs.txt")?;
        let reader = BufReader::new(f);

        let events = reader.lines().filter_map(|l| {
            if let Ok(s) = l {
                parse(&s)
            } else {
                None
            }
        });

        for e in events {
            println!("{:#?}", e);
        }

        Ok(())
    }

    #[test]
    fn test_invalid_password() {
        let logstr = "03/16/2021 13:45:19: Peer 76561197969472572 has wrong password";
        let res = parse(&logstr);

        if let Some(Event::IncorrectPasswordGiven(_)) = res {
            //
        } else {
            panic!("Didn't get incorrect password event from parse");
        }
    }
}
