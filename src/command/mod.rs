use std::{process, thread, time::Duration};

mod docker;

pub fn run() {
    info!(target: "lazymc-docker-proxy::command", "Starting server...");
    // Set a handler for SIGTERM
    ctrlc::set_handler(move || {
        info!(target: "lazymc-docker-proxy::command", "Received SIGTERM, stopping server...");
        docker::stop();
        process::exit(0);
    })
    .unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::command", "Error setting SIGTERM handler: {}", err);
        process::exit(1);
    });

    // Start the command
    docker::start();

    // Wait for SIGTERM
    loop {
        trace!(target: "lazymc-docker-proxy::command", "Waiting for SIGTERM...");
        thread::sleep(Duration::from_secs(1));
    }
}
