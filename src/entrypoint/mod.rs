mod config;

use std::process::{self, exit, Command, ExitStatus};

pub fn run() {
    config::generate();

    // Create a new Command to run "lazymc start"
    let mut child: process::Child = Command::new("lazymc")
        .arg("start")
        .spawn()
        .expect("Failed to start lazymc process");

    // Wait for the process to finish
    let status: ExitStatus = child.wait().expect("lazymc process exited with error");

    // Check if the process was successful
    if !status.success() {
        eprintln!("lazymc process exited with error: {}", status);
        exit(1);
    }
}
