///
/// valheim-watcher
/// A wrapper program to launch a Valheim server that can log in-game events
/// and send messages / updates about the state of the server as a Discord bot.
///
/// author: Nick Pelone <nick.pelone@gmail.com> / <nick.pelone@calyptix.com>
///
mod lib;

use discord::model::{Channel, ChannelId};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use valheim_log_parser::{parse, Event};

use anyhow::Context;

use chrono::Utc;

fn main() -> anyhow::Result<()> {
    // Get config
    let c = match lib::Config::build_from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Unable to build runtime config from the environment. Check your:\n\
                - VALHEIM_START_SCRIPT\n\
                - DISCORD_KEY\n\
                - CHANNEL_ID\n\
                environment variables and try again."
            );
            return Err(e);
        }
    };

    let bot = Arc::new(Mutex::new(discord::Discord::from_bot_token(&c.bot_key)?));
    let _conn = {
        let bot = bot.lock().unwrap();
        bot.connect()?
    };

    _conn.0.set_game_name("Valheim Dedicated Server".to_owned());

    let channel = {
        let bot = bot.lock().unwrap();
        match bot.get_channel(ChannelId(c.channel_id))? {
            Channel::Public(c) => c,
            _ => panic!("Channel given does not correspond to a public text chat channel, unable to continue.")
        }
    };

    println!("Starting server");

    // Spawn server and capture stdout
    let mut server = lib::spawn_server(&c)?;
    let stdout = server
        .stdout
        .take()
        .context("Unable to take stdout handle of server")?;

    {
        let bot = bot.lock().unwrap();
        bot.send_message(channel.id, "Valheim server started", "", false)?;
    }

    // Create log file we'll write to
    let logfile_name = format!("valheim-dedicated-server-{}.log", Utc::now().to_rfc3339());
    let mut logfile = File::create(&logfile_name)?;

    println!("Logging to: {}", logfile_name);

    // Create copies of the mutex'd discord instance
    // and the PublicChannel we got from discord API
    // that we're going to be chatting in.
    // These will get moved into the signal handler thread.
    let signalbot = bot.clone();
    let signalchannel = channel.clone();

    // Create signal handler
    let mut signals = Signals::new(&[SIGINT])?;
    let handle = thread::spawn(move || {
        for _sig in signals.forever() {
            let arc = signalbot.clone();
            let bot = arc.lock().unwrap();

            println!("[WATCHER]: Shutting down server");
            bot.send_message(signalchannel.id, "Valheim server shutting down", "", false)?;

            lib::shutdown_server(server.id() as i32)?;
            server
                .wait()
                .context("Couldn't finish wait() for server child process")?;
            break;
        }
        Ok::<(), anyhow::Error>(())
    });

    // Create parse loop from captured stdout
    let reader = BufReader::new(stdout);
    let lines = reader.lines();

    // incomplete state transition storage
    let mut pending_ids: Vec<u64> = Vec::new();
    let mut pending_chars: Vec<String> = Vec::new();

    // and the grand state table itself - SteamIDs -> Character names
    let mut state: HashMap<u64, String> = HashMap::new();

    for line in lines {
        // Extract the String from the Result, adding additional context if it fails.
        let l = line.context("Unable to read line, server may have died")?;

        // Write the line to the logfile
        write!(&mut logfile, "{}\n", &l)?;

        match parse(&l) {
            Ok(Some(event)) => {
                let bot = bot.lock().unwrap();

                match event {
                    Event::UserConnected(cd) => {
                        println!("Received new connection from SteamID {}", cd.steam_id);

                        pending_ids.push(cd.steam_id);
                    }
                    Event::UserDisconnected(cd) => {
                        // If the user was successfully removed from the state table, send a bot message
                        if let Some((id, character)) = state.remove_entry(&cd.steam_id) {
                            let msg = format!(
                                "{} has disconnected.\nhttps://steamcommunity.com/profiles/{}",
                                character, id
                            );
                            bot.send_message(channel.id, &msg, "", false)?;

                            println!("{} ({}) disconnected.", character, id);
                        } else {
                            // This might have been one of those double-logging disconnect messages.
                            // TODO: What should be done?
                        }
                    }
                    Event::WorldSaved(s) => {
                        println!("World saved at {}, {}ms", s.timestamp, s.time_spent);
                    }
                    Event::CharacterDied(sp) => {
                        let steamid = lib::steamid_from_character(&sp.character, &state);

                        let msg = format!("{} died an uneventful death. GGWP", sp.character);
                        bot.send_message(channel.id, &msg, "", false)?;

                        println!("{} ({}) died.", sp.character, steamid);
                    }
                    Event::CharacterSpawned(sp) => {
                        let steamid = lib::steamid_from_character(&sp.character, &state);

                        println!("{} ({}) spawned.", sp.character, steamid);

                        let is_character_tracked =
                            state.values().any(|x| x.clone() == sp.character);
                        if !is_character_tracked {
                            pending_chars.push(sp.character);
                        }
                    }
                    Event::IncorrectPasswordGiven(cd) => {
                        pending_ids.retain(|x| *x != cd.steam_id);
                        bot.send_message(channel.id, &format!("A user gave the wrong password.\nhttps://steamcommunity.com/profiles/{}", cd.steam_id), "", false)?;
                        println!("SteamID {} gave wrong password, rejected.", cd.steam_id);
                    }
                };

                if let Some(id) = pending_ids.get(0) {
                    if let Some(c) = pending_chars.get(0) {
                        let id = *id;
                        let character = c.clone();

                        state.insert(id, character.clone());
                        pending_chars.remove(0);
                        pending_ids.remove(0);

                        let msg = format!(
                            "{} has connected.\nhttps://steamcommunity.com/profiles/{}",
                            character, id
                        );
                        bot.send_message(channel.id, &msg, "", false)?;
                    }
                }
            }
            Ok(None) => {} // we don't care, it was a useless line
            Err(e) => {
                eprintln!("Unable to parse Valheim log line: {}", e);
                continue;
            }
        };
    }

    // Ensure any OS-buffered logs are written to disk before shutdown
    logfile.flush()?;

    // Join on the signal handling thread, exiting once it is done with its work.
    handle.join().unwrap()
}
