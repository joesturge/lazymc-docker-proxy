#[macro_use]
extern crate log;

mod backend;
mod command;
mod docker;
mod entrypoint;
mod health;
mod kubernetes;
mod logging;

use clap::Parser;

/// Wrapper for lazymc to run against a docker minecraft server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Execute with this flag when running as a lazymc start command
    #[arg(short, long)]
    command: bool,

    /// The lazymc group name
    #[arg(short, long, requires_if("command", "true"))]
    group: Option<String>,

    /// Execute with this flag when running as a health check
    #[arg(short, long)]
    health: bool,
}

/// Main entrypoint for the application
fn main() {
    logging::init();

    let args: Args = Args::parse();

    if args.command {
        command::run(args.group.unwrap());
    } else if args.health {
        health::run();
    } else {
        entrypoint::run();
    }
}
