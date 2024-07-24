mod config;

use std::process::Command;

pub fn run() {
    config::generate();
    
    // Create a new Command to run "lazymc start"
    let mut child: std::process::Child = Command::new("lazymc")
        .arg("start")
        .spawn()
        .expect("Failed to start lazymc process");

    // Wait for the process to finish
    let status: std::process::ExitStatus = child.wait().expect("lazymc process exited with error");

    // Check if the process was successful
    if !status.success() {
        eprintln!("lazymc process exited with error: {}", status);
        std::process::exit(1);
    }
}
