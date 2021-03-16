use std::{path::{Path, PathBuf}, process::{Command, Stdio, Child}};
use std::env;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

#[derive(Clone, Debug)]
pub struct Config {
    pub start_script: PathBuf
}

// TODO: proper errors.

impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> Config {
        let stored = PathBuf::from(path.as_ref());

        Config { start_script: stored }
    }

    pub fn build_from_env() -> anyhow::Result<Config> {
        let start_script = env::var("VALHEIM_START_SCRIPT")?;

        Ok(Config::new(&start_script))
    }
}

pub fn spawn_server(config: &Config) -> Child  {
    env::set_current_dir(&config.start_script.parent().unwrap()).expect("Unable to change directory to Valheim dedicated server location.");
    Command::new("bash")
        .arg(&config.start_script)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Unable to launch Valheim dedicated server. Check the VALHEIM_START_SCRIPT env var.")
}

pub fn shutdown_server(pid: i32) {
    signal::kill(Pid::from_raw(pid), Signal::SIGINT).expect("Unable to send SIGTERM to server");
}