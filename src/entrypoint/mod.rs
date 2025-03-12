use config::Config;
use log::Level;
use regex::Regex;
use std::io::BufRead;
use std::io::BufReader;
use std::process;
use std::process::exit;

use crate::adapter::Adapter;
use crate::health::healthy;

pub mod config;

/// Entrypoint for the application
pub fn run<T: Adapter>() {
    // Ensure all server containers are stopped before starting
    info!(target: "lazymc-docker-proxy::entrypoint", "Ensuring all server containers are stopped...");

    T::stop_all_containers();

    let configs: Vec<Config> = T::get_container_labels()
        .iter()
        .map(Config::from_container_labels)
        .collect();

    let mut children: Vec<process::Child> = Vec::new();

    for config in configs {
        let group = config.group();

        info!(target: "lazymc-docker-proxy::entrypoint", "Starting lazymc process for group: {}...", group);
        let mut child: process::Child = config
            .start_command::<T>()
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdout = child.stdout.take();

        let group_clone = group.clone();

        std::thread::spawn(move || {
            let stdout_reader = BufReader::new(stdout.take().unwrap());
            for line in stdout_reader.lines() {
                wrap_log::<T>(&group_clone, line);
            }
        });

        let mut stderr = child.stderr.take();

        let group_clone = group.clone();

        std::thread::spawn(move || {
            let stderr_reader = BufReader::new(stderr.take().unwrap());
            for line in stderr_reader.lines() {
                wrap_log::<T>(&group_clone, line)
            }
        });

        children.push(child);
    }

    // If this app receives a signal, stop all server containers
    ctrlc::set_handler(move || {
        info!(target: "lazymc-docker-proxy::entrypoint", "Received exit signal. Stopping all server containers...");
        T::stop_all_containers();
        exit(0);
    }).unwrap();

    // Set the health status to healthy
    healthy();

    // wait indefinitely
    loop {
        std::thread::park();
    }
}

/// Wrap log messages from child processes
fn wrap_log<T: Adapter>(group: &String, line: Result<String, std::io::Error>) {
    if let Ok(line) = line {
        let regex: Regex =
            Regex::new(r"(?P<level>[A-Z]+)\s+(?P<target>[a-zA-Z0-9:_-]+)\s+>\s+(?P<message>.+)$")
                .unwrap();
        if let Some(captures) = regex.captures(&line) {
            let level: Level = captures.name("level").unwrap().as_str().parse().unwrap();
            let target = captures.name("target").unwrap().as_str().to_owned();
            let message = captures.name("message").unwrap().as_str().to_owned();

            let wrapped_target = &format!("{}::{}", group, target);
            log!(target: wrapped_target, level, "{}", message);
            handle_log::<T>(group, &level, &message);
        } else {
            print!("{}", line);
        }
    }
}

/// Handle log messages that require special attention
fn handle_log<T: Adapter>(group: &String, level: &Level, message: &String) {
    match (level, message.as_str()) {
        (Level::Warn, "Failed to stop server, no more suitable stopping method to use") => {
            warn!(target: "lazymc-docker-proxy::entrypoint", "Unexpected server state detected, force stopping {} server container...", group);
            T::stop(group);
            info!(target: "lazymc-docker-proxy::entrypoint", "{} server container forcefully stopped", group);
        }
        _ => {}
    }
}
