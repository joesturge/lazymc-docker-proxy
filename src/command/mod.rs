use std::{process, thread, time::Duration};
use std::sync::Arc;

use crate::backend;

/// Run the command to start a group
pub fn run(group: String) {
    info!(target: "lazymc-docker-proxy::command", "Received command to start group: {}", group);
    let backend = Arc::new(backend::create());
    
    // Set a handler for SIGTERM
    let cloned_group = group.clone();
    let backend_for_handler = Arc::clone(&backend);
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
