mod docker;

pub fn run() {
    // Set a handler for SIGTERM
    ctrlc::set_handler(move || {
        docker::stop();
        std::process::exit(0);
    })
    .expect("Error setting SIGTERM handler");

    // Start the command
    docker::start();

    // Wait for SIGTERM
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}