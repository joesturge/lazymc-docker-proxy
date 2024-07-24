mod command;
mod entrypoint;

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
        command::run();
    } else {
        entrypoint::run();
    }
}
