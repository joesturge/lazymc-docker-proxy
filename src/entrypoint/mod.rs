mod config;

use std::process::{self, exit, Command, ExitStatus};

pub fn run() {
    // Generate the lazymc config file
    config::generate();

    // Create a new Command to run "lazymc start"
    info!(target: "lazymc-docker-proxy::entrypoint", "Starting lazymc process...");
    let mut child: process::Child = Command::new("lazymc")
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
