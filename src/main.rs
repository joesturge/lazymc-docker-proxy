#[macro_use]
extern crate log;

mod command;
mod entrypoint;
mod logging;
mod docker;

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
}

fn main() {
    logging::init();

    let args: Args = Args::parse();

    if args.command {
        command::run(args.group.clone().unwrap());
    } else {
        entrypoint::run();
    }
}
