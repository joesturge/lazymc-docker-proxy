use std::{process, thread, time::Duration};

use crate::backend;

/// Run the command to start a group
pub fn run(group: String) {
    info!(target: "lazymc-docker-proxy::command", "Received command to start group: {}", group);
    let backend = backend::get_backend();
    
    // Set a handler for SIGTERM
    let cloned_group = group.clone();
    let backend_for_handler = backend::get_backend();
    ctrlc::set_handler(move || {
        info!(target: "lazymc-docker-proxy::command", "Received SIGTERM, stopping server...");
        backend_for_handler.stop(cloned_group.clone());
        process::exit(0);
    }).unwrap();

    // Start the command
    backend.start(group.clone());

    // Wait for SIGTERM
    loop {
        trace!(target: "lazymc-docker-proxy::command", "Waiting for SIGTERM...");
        thread::sleep(Duration::from_secs(1));
    }
}
