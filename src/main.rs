mod command;
mod entrypoint;

use std::process::exit;

use clap::Parser;

/// Wrapper for lazymc to run against a docker minecraft server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Execute with this flag when running as a lazymc start command
    #[arg(short, long)]
    command: bool,
}

fn main() {
    let args: Args = Args::parse();

    if args.command {
        // Set a handler for SIGTERM
        ctrlc::set_handler(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(command::stop());
            exit(0);
        })
        .expect("Error setting SIGTERM handler");

        // Start the command
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(command::start());

        // Wait for SIGTERM
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    } else {
        entrypoint::run();
    }
}
