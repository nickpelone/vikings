use valheim_log_parser::parse;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let f = File::open("./ValheimServerLogs2Login.txt")?;
    let reader = BufReader::new(f);

    let events = reader.lines().filter_map(|x| {
        if let Ok(s) = x {
            parse(&s).unwrap()
        } else {
            None
        }
    });

    let mut state: HashMap<u64, String> = HashMap::new();
    let mut pending_steam_ids: Vec<u64> = Vec::new();
    let mut pending_characters: Vec<String> = Vec::new();

    for e in events {
        println!("Got event: {:?}", e);
        match e {
            valheim_log_parser::Event::UserConnected(cd) => {
                pending_steam_ids.push(cd.steam_id);
            }
            valheim_log_parser::Event::CharacterSpawned(sd) => {
                pending_characters.push(sd.character);
            }
            _ => {}
        };

        if let Some(id) = pending_steam_ids.get(0) {
            if let Some(c) = pending_characters.get(0) {
                state.insert(*id, c.clone());
                pending_characters.remove(0);
                pending_steam_ids.remove(0);
            }
        }

        println!("The state is now:");
        println!("{:#?}", state);
    }

    println!("Final state");
    println!("{:#?}", state);

    Ok(())
}
