use std::{
    collections::{HashMap},
    env,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    fmt
};

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

use anyhow::Context;

pub const STEAM_COMMUNITY: &str = "https://steamcommunity.com/profiles/";
pub const DEATH_GIFS: &[&str] = &[
    "https://media.giphy.com/media/BwRzjeqPnC6Dm/giphy.gif", // Kim K tragic gif
    "https://media.giphy.com/media/3o6Mb8ARo4g0MBKxvq/giphy.gif", // homer simpson crying
    "https://media.giphy.com/media/26vIdFahaDedVAb8k/giphy.gif", // blue planet fish getting eaten
    "https://media.giphy.com/media/65i0TaZCsNlks/giphy.gif", // betty white golden girls
    "https://media.giphy.com/media/JvEMPOQubkyQx9YLQ5/giphy.gif", // mindy office
    "https://media.giphy.com/media/yF3ci8XI6RY8E/giphy.gif", // rocko's modern life fish
    "https://media.giphy.com/media/3oFzmlxO0EvgAh2qaI/giphy.gif", // portlandia fred armisen
];

#[derive(Clone, Debug)]
pub struct State {
    pub table: HashMap<u64, String>
}

impl State {
    pub fn steamid_from_character(&self, character: &str) -> u64 {
        *self.table
            .iter()
            .find_map(|(key, val)| if val == character { Some(key) } else { None })
            .unwrap_or(&0)
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.table.len() == 0 {
            write!(f, "No users connected.\n")
        } else {
            for (k, v) in self.table.iter() {
                write!(f, "{} - https://steamcommunity.com/profiles/{}\n", v, k)?;
            }
            write!(f, "\n")
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub start_script: PathBuf,
    pub bot_key: String,
    pub channel_id: u64,
}

impl Config {
    pub fn new<P: AsRef<Path>>(path: P, bot_key: String, channel_id: u64) -> Config {
        let stored = PathBuf::from(path.as_ref());

        Config {
            start_script: stored,
            bot_key,
            channel_id,
        }
    }

    pub fn build_from_env() -> anyhow::Result<Config> {
        let start_script = env::var("VALHEIM_START_SCRIPT")?;
        let bot_key = env::var("DISCORD_KEY")?;
        let channel_id: u64 = env::var("CHANNEL_ID")?.parse()?;

        Ok(Config::new(&start_script, bot_key, channel_id))
    }
}

pub fn spawn_server(config: &Config) -> anyhow::Result<Child> {
    env::set_current_dir(&config.start_script.parent().unwrap())
        .expect("Unable to change directory to Valheim dedicated server location.");
    Command::new("bash")
        .arg(&config.start_script)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context(
            "Unable to launch Valheim dedicated server. Check the VALHEIM_START_SCRIPT env var.",
        )
}

pub fn shutdown_server(pid: i32) -> anyhow::Result<()> {
    signal::kill(Pid::from_raw(pid), Signal::SIGINT).context("Unable to send SIGTERM to server")
}
