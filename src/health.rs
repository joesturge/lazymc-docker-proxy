use std::{fs, process::exit};

/// The status of the health check
pub enum Status {
    STARTING,
    HEALTHY,
    UNHEALTHY,
}

/// Convert a string to a status
impl From<&str> for Status {
    fn from(status: &str) -> Status {
        match status {
            "STARTING" => Status::STARTING,
            "HEALTHY" => Status::HEALTHY,
            "UNHEALTHY" => Status::UNHEALTHY,
            _ => Status::UNHEALTHY,
        }
    }
}

/// Convert a status to a string
impl From<Status> for String {
    fn from(status: Status) -> String {
        match status {
            Status::STARTING => String::from("STARTING"),
            Status::HEALTHY => String::from("HEALTHY"),
            Status::UNHEALTHY => String::from("UNHEALTHY"),
        }
    }
}

/// Check the status
pub fn check() -> Status {
    let status = fs::read_to_string("/app/health").unwrap_or_else(|_| Status::UNHEALTHY.into());
    debug!(target: "lazymc-docker-proxy::health", "Health status: {}", status);
    status.trim().into()
}

/// Set the status
fn set(status: Status) {
    let status_str: String = status.into();
    debug!(target: "lazymc-docker-proxy::health", "Setting health status to: {}", status_str);
    fs::write("/app/health", status_str).unwrap();
}

pub fn healthy() {
    set(Status::HEALTHY);
    info!(target: "lazymc-docker-proxy::health", "Application is healthy.");
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
