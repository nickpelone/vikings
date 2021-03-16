mod lib;

use signal_hook::{iterator::Signals};
use signal_hook::consts::SIGINT;

use std::thread;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use valheim_log_parser::{parse, Event};

use anyhow::Context;

use chrono::Utc;

fn main() -> anyhow::Result<()> {
    // Get config
    let c = lib::Config::build_from_env()?;
    println!("{:#?}", c);

    println!("Starting server");

    // Spawn server and capture stdout
    let mut server = lib::spawn_server(&c);
    let stdout = server.stdout.take().context("Unable to take stdout handle of server")?;

    // Create log file we'll BufWrite to
    let mut logfile = File::create(&format!("valheim-dedicated-server-{}.log", Utc::now().to_rfc3339()))?;

    // Create signal handler
    let mut signals = Signals::new(&[SIGINT])?;
    let handle = thread::spawn(move || {
        for _sig in signals.forever() {
            println!("[WATCHER]: Shutting down server");
            lib::shutdown_server(server.id() as i32);
            server.wait().context("Couldn't finish wait() for server child process")?;
            break;
        }
        Ok::<(), anyhow::Error>(())
    });

    // Create parse loop from captured stdout
    let reader = BufReader::new(stdout);
    let lines = reader.lines();

    for line in lines {
        let l = line.context("Unable to read line, server may have died")?;
        write!(&mut logfile, "{}\n", &l)?;
        if let Some(event) = parse(&l) {
            println!("{:#?}", event);
        }
    }

    logfile.flush()?;

    // Must be unwrapped because thread errors are boxed values related to the panic,
    // and can't be handled by `?`
    // Resulting final Result type matches return signature
    handle.join().unwrap()
}
