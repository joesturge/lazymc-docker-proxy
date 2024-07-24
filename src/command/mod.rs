use std::env;
use docker::Docker;
use docker::container::{ContainerListOptions, ContainerListFilters};
use std::process::exit;

fn main() {
  // Check if LAZYMC_GROUP environment variable is set
  if let Ok(lazymc_group) = env::var("LAZYMC_GROUP") {
    // Create a Docker client
    let docker = Docker::connect_with_defaults().expect("Failed to connect to Docker");

    // Get IDs of stopped containers with the label and filters
    let stopped_containers = docker.containers()
      .list(&ContainerListOptions::default()
        .filter(ContainerListFilters::label("lazymc.group", lazymc_group.clone()))
        .filter(ContainerListFilters::status("exited")))
      .expect("Failed to list stopped containers");

    if !stopped_containers.is_empty() {
      println!("Starting stopped containers...");
      for container in stopped_containers {
        docker.containers()
          .start(&container.id)
          .expect("Failed to start container");
      }
    }

    // Get IDs of all containers with the label and filter, including running ones
    let all_containers = docker.containers()
      .list(&ContainerListOptions::default()
        .filter(ContainerListFilters::label("lazymc.group", lazymc_group.clone())))
      .expect("Failed to list containers");

    if all_containers.is_empty() {
      println!("Error: No containers found with label lazymc.group={}.", lazymc_group);
      exit(1);
    }

    // Function to handle SIGTERM signal
    fn handle_sigterm() {
      println!("SIGTERM received. Stopping all containers...");
      for container in &all_containers {
        docker.containers()
          .stop(&container.id, Some(10))
          .expect("Failed to stop container");
      }
      println!("Stopped all containers.");
      exit(0);
    }

    // Trap SIGTERM and stop all containers
    ctrlc::set_handler(handle_sigterm).expect("Error setting SIGTERM handler");

    // Wait indefinitely
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));
    }
  } else {
    println!("Error: Environment variable LAZYMC_GROUP is not set.");
    exit(1);
  }
}
