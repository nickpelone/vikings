use std::collections::HashMap;
use std::env;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

use anyhow::Context;
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

pub fn steamid_from_character(character: &str, state: &HashMap<u64, String>) -> u64 {
    *state
        .iter()
        .find_map(|(key, val)| if val == character { Some(key) } else { None })
        .unwrap_or(&0)
}
