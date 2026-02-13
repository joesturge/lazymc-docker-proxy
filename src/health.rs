use std::{fs, process::exit};
use strum::{Display, EnumString};

/// The status of the health check

#[derive(EnumString, Display)]
#[allow(clippy::upper_case_acronyms)]
pub enum Status {
    STARTING,
    HEALTHY,
    UNHEALTHY,
}

/// Check the status
pub fn check() -> Status {
    let status =
        fs::read_to_string("/app/health").unwrap_or_else(|_| Status::UNHEALTHY.to_string());
    debug!(target: "lazymc-docker-proxy::health", "Health status: {}", status);
    status.trim().parse().unwrap_or(Status::UNHEALTHY)
}

/// Set the status
fn set(status: Status) {
    let status_str = status.to_string();
    debug!(target: "lazymc-docker-proxy::health", "Setting health status to: {}", status_str);
    fs::write("/app/health", status_str).unwrap();
}

pub fn healthy() {
    info!(target: "lazymc-docker-proxy::health", "Application is healthy.");
    set(Status::HEALTHY);
}

pub fn unhealthy() {
    set(Status::UNHEALTHY);
    error!(target: "lazymc-docker-proxy::health", "Application is unhealthy.");
}

pub fn run() {
    match check() {
        Status::STARTING => exit(1),
        Status::HEALTHY => exit(0),
        Status::UNHEALTHY => exit(1),
    }
}
