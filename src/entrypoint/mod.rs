mod config;
mod docker;

use config::Config;
use std::process::{self, exit, ExitStatus};

pub fn run() {
    // Generate the lazymc config either from labels if set or from env
    let config: Config;
    let container_labels = docker::get_container_labels();
    if container_labels.is_none() {
        config = Config::from_env()
    } else {
        config = Config::from_container_labels(container_labels.unwrap());
    }

    info!(target: "lazymc-docker-proxy::entrypoint", "Starting lazymc process...");
    let mut child: process::Child = config.start_command()
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
