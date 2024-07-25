use bollard::container::{ListContainersOptions, StartContainerOptions};
use bollard::Docker;
use std::collections::HashMap;
use std::env;
use std::process::exit;

async fn get_containers(
    docker: &Docker,
    lazymc_group: String,
    status: String,
) -> Vec<bollard::secret::ContainerSummary> {
    return docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: HashMap::from_iter(vec![
                (
                    "label".to_string(),
                    vec![format!("lazymc.group={}", lazymc_group)],
                ),
                ("status".to_string(), vec![status]),
            ]),
            ..Default::default()
        }))
        .await
        .unwrap();
}

pub async fn stop() {
    // Check if LAZYMC_GROUP environment variable is set
    if let Ok(lazymc_group) = env::var("LAZYMC_GROUP") {
        // Create a Docker client
        let docker: Docker =
            Docker::connect_with_socket_defaults().expect("Failed to connect to Docker");

        // Get IDs of running containers with the label and filters
        let running_containers: Vec<bollard::secret::ContainerSummary> =
            get_containers(&docker, lazymc_group, "running".to_string()).await;

        // Stop running containers
        if !running_containers.is_empty() {
            for container in running_containers {
                docker
                    .stop_container(
                        container.id.as_ref().unwrap(),
                        None::<bollard::container::StopContainerOptions>,
                    )
                    .await
                    .unwrap();
            }
        }
    } else {
        println!("Error: Environment variable LAZYMC_GROUP is not set.");
        exit(1);
    }
}

pub async fn start() {
    // Check if LAZYMC_GROUP environment variable is set
    if let Ok(lazymc_group) = env::var("LAZYMC_GROUP") {
        // Create a Docker client
        let docker: Docker =
            Docker::connect_with_socket_defaults().expect("Failed to connect to Docker");

        // Get IDs of stopped containers with the label and filters
        let stopped_containers: Vec<bollard::secret::ContainerSummary> =
            get_containers(&docker, lazymc_group, "exited".to_string()).await;

        // Start stopped containers
        if !stopped_containers.is_empty() {
            for container in stopped_containers {
                docker
                    .start_container(
                        container.id.as_ref().unwrap(),
                        None::<StartContainerOptions<&str>>,
                    )
                    .await
                    .unwrap();
            }
        }
    } else {
        println!("Error: Environment variable LAZYMC_GROUP is not set.");
        exit(1);
    }
}
