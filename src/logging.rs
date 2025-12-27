use std::env;

extern crate pretty_env_logger;

/// Initialize the logger
pub fn init() {
    // Set default log level if none is set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();
}
