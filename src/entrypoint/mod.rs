mod config;
mod docker;
use config::Config;
use regex::Regex;
use std::{io::{BufRead, BufReader}, process::{self, exit, ExitStatus}};

pub fn run() {
    let labels_list = docker::get_container_labels();
    let mut configs: Vec<Config> = Vec::new();
    let mut children: Vec<process::Child> = Vec::new();

    for label in labels_list {
        configs.push(Config::from_container_labels(label));
    }

    if configs.is_empty() {
        configs.push(Config::from_env());
    }

    for config in configs {
        let group: String = config.group().into();

        info!(target: "lazymc-docker-proxy::entrypoint", "Starting lazymc process for group: {}...", group.clone());
        let mut child: process::Child = config.start_command()
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap_or_else(|err| {
                error!(target: "lazymc-docker-proxy::entrypoint", "Failed to start lazymc process for group: {}: {}", group.clone(), err);
                exit(1);
            });

        let mut stdout = child.stdout.take();
        let group_clone = group.clone();
        std::thread::spawn(move || {
            let stdout_reader = BufReader::new(stdout.take().unwrap());
            for line in stdout_reader.lines() {
                wrap_log(&group_clone, line);
            }
        });

        let mut stderr = child.stderr.take();
        std::thread::spawn(move || {
            let stderr_reader = BufReader::new(stderr.take().unwrap());
            for line in stderr_reader.lines() {
                wrap_log(&group.clone(), line)
            }
        });

        children.push(child);
    }

    for mut child in children {
        let exit_status: ExitStatus = child.wait().unwrap_or_else(|err| {
            error!(target: "lazymc-docker-proxy::entrypoint", "Failed to wait for lazymc process to exit: {}", err);
            exit(1);
        });

        if !exit_status.success() {
            error!(target: "lazymc-docker-proxy::entrypoint", "lazymc process exited with non-zero status: {}", exit_status);
            exit(1);
        }
    }
}

fn wrap_log(group: &String, line: Result<String, std::io::Error>) {
    if let Ok(line) = line {
        let regex: Regex = Regex::new(r"(?P<level>[A-Z]+)\s+(?P<target>[a-zA-Z0-9:_-]+)\s+>\s+(?P<message>.+)$").unwrap();
        if let Some(captures) = regex.captures(&line) {
            let level = captures.name("level").unwrap().as_str();
            let target = captures.name("target").unwrap().as_str();
            let message = captures.name("message").unwrap().as_str();

            let wrapped_target = &format!("{}::{}", group, target);
            match level {
                "TRACE" => trace!(target: wrapped_target, "{}", message),
                "DEBUG" => debug!(target: wrapped_target, "{}", message),
                "INFO" => info!(target: wrapped_target, "{}", message),
                "WARN" => warn!(target: wrapped_target, "{}", message),
                "ERROR" => error!(target: wrapped_target, "{}", message),
                "CRITICAL" => error!(target: wrapped_target, "{}", message),
                _ => warn!(target: "lazymc-docker-proxy::entrypoint", "Could not parse log line: {}", line),
            }
        } else {
            warn!(target: "lazymc-docker-proxy::entrypoint", "Could not parse log line: {}", line);
        }
    }
}
