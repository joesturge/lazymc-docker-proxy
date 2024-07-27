#[macro_use]
extern crate log;

mod command;
mod entrypoint;
mod logging;

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
    logging::init();

    let args: Args = Args::parse();

    if args.command {
        command::run();
    } else {
        entrypoint::run();
    }
}
