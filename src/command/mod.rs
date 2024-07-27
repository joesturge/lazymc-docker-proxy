use std::{process, thread, time::Duration};

mod docker;

pub fn run() {
    // Set a handler for SIGTERM
    ctrlc::set_handler(move || {
        docker::stop();
        process::exit(0);
    })
    .expect("Error setting SIGTERM handler");

    // Start the command
    docker::start();

    // Wait for SIGTERM
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}