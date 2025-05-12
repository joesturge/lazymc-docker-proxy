#[macro_use]
extern crate log;

mod adapter;
mod command;
mod entrypoint;
mod health;
mod logging;

use std::process::exit;
use clap::Parser;

use crate::adapter::docker::DockerAdapter;
use crate::adapter::systemd::SystemdAdapter;
use crate::adapter::Adapter;
use crate::health::unhealthy;

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

    /// Choose the appropriate adapter
    #[arg(short, long)]
    adapter: Option<String>,
}

fn main() {
    logging::init();

    let args: Args = Args::parse();

    match args.adapter.clone().or_else(|| std::env::var("ADAPTER").ok()).as_deref() {
        Some("systemd") => {
            run_with_adapter::<SystemdAdapter>(args);
        }
        Some("docker") | None => {
            run_with_adapter::<DockerAdapter>(args);
        }
        Some(adapter) => {
            error!(target: "lazymc-docker-proxy", "Unknown adapter: {}", adapter);
            unhealthy();
            exit(1);
        }
    };
}

fn run_with_adapter<T: Adapter>(args: Args) {
    if args.command {
        command::run::<T>(args.group.unwrap());
    } else if args.health {
        health::run();
    } else {
        entrypoint::run::<T>();
    }
}
