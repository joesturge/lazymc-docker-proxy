mod config;

use std::{
    env::var,
    process::{self, exit, Command, ExitStatus},
};
use version_compare::Version;

pub fn run() {
    // lazymc dropped support minecraft servers with version less than 1.20.3
    let is_legacy: bool = match var("PUBLIC_VERSION") {
        Ok(version) => Version::from(version.as_ref()) < Version::from("1.20.3"),
        Err(_) => false,
    };

    // Generate the lazymc config file
    config::generate(is_legacy);

    // Create a new Command to run "lazymc start"
    info!(target: "lazymc-docker-proxy::entrypoint", "Starting lazymc process...");

    // if is_legacy is true, run lazymc-legacy instead of lazymc
    let program: &str = match is_legacy {
        true => {
            debug!(target: "lazymc-docker-proxy::entrypoint", "Running legacy version of lazymc as server protocol version is less than 1.20.3");
            "lazymc-legacy"
        },
        false => {
            debug!(target: "lazymc-docker-proxy::entrypoint", "Running newer version of lazymc as server protocol version is 1.20.3 or higher");
            "lazymc"
        },
    };

    let mut child: process::Child = Command::new(program)
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
