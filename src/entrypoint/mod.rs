mod config;

use std::{
    env::var,
    process::{self, exit, Command, ExitStatus},
};
use version_compare::Version;

pub fn run() {
    // Generate the lazymc config file
    config::generate();

    // Create a new Command to run "lazymc start"
    info!(target: "lazymc-docker-proxy::entrypoint", "Starting lazymc process...");

    // true if PUBLIC_VERSION env var is set and is less than 1.20.3
    let is_legacy: bool = match var("PUBLIC_VERSION") {
        Ok(version) => Version::from(version.as_ref()) < Version::from("1.20.3"),
        Err(_) => false,
    };

    // if is_legacy is true, run lazymc-legacy instead of lazymc
    match is_legacy {
        true => debug!(target: "lazymc-docker-proxy::entrypoint", "Running legacy version of lazymc as server protocol version is less than 1.20.3"),
        false => debug!(target: "lazymc-docker-proxy::entrypoint", "Running newer version of lazymc as server protocol version is 1.20.3 or higher"),
    }
    let mut child: process::Child = Command::new(match is_legacy {
        true => "lazymc-legacy",
        false => "lazymc",
    })
        .arg("start")
        .spawn()
        .unwrap_or_else(|err| {
            error!(target: "lazymc-docker-proxy::entrypoint", "Failed to start lazymc process: {}", err);
            exit(1);
        });

    // Wait for the process to finish
    let status: ExitStatus = child.wait()
    .unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::entrypoint", "lazymc process exited with error: {}", err);
        exit(1);
    });

    // Check if the process was successful
    if !status.success() {
        error!(target: "lazymc-docker-proxy::entrypoint", "lazymc process exited with error: {}", status);
        exit(1);
    }

    info!(target: "lazymc-docker-proxy::entrypoint", "lazymc process exited successfully");
}
