use std::{process, thread, time::Duration};

use crate::adapter::Adapter;

/// Run the command to start a group
pub fn run<T: Adapter>(group: String) {
    info!(target: "lazymc-docker-proxy::command", "Received command to start group: {}", group);

    // Set a handler for SIGTERM
    let cloned_group = group.clone();

    ctrlc::set_handler(move || {
        info!(target: "lazymc-docker-proxy::command", "Received SIGTERM, stopping server...");
        T::stop(&cloned_group);
        process::exit(0);
    })
    .unwrap();

    // Start the command
    T::start(&group);

    // Wait for SIGTERM
    loop {
        trace!(target: "lazymc-docker-proxy::command", "Waiting for SIGTERM...");
        thread::sleep(Duration::from_secs(1));
    }
}
