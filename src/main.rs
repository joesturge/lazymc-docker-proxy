mod command;
mod entrypoint;

use std::process::exit;

use clap::Parser;

#[derive(Parser, Debug)]
enum Command {
    /// Start the server
    #[clap(name = "start")]
    Start,
    /// Stop the server
    #[clap(name = "stop")]
    Stop,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
}

fn main() {
    let args: Args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Command::Start => {
                command::start();
                exit(0);
            }
            Command::Stop => {
                command::stop();
                exit(0);
            }
        }
    } else {
        entrypoint::run();
    }
}
