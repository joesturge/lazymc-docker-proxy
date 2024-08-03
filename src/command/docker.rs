use std::collections::HashMap;
use std::process::exit;

use bollard::container::{ListContainersOptions, StartContainerOptions, StopContainerOptions};
use bollard::Docker;
use futures::{future, FutureExt};
use log::error;
use tokio::runtime::Runtime;

pub fn stop(group: String) {
    debug!(target: "lazymc-docker-proxy::command::docker", "Stopping containers...");
    let docker: Docker = Docker::connect_with_local_defaults().unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::command::docker", "Error connecting to docker: {}", err);
        exit(1)
    });

    let mut list_container_filters: HashMap<String, Vec<String>> =
        HashMap::<String, Vec<String>>::new();

    // find all matching running containers
    list_container_filters.insert("status".to_string(), vec!["running".to_string()]);
    list_container_filters.insert(
        "label".to_string(),
        vec![format!("lazymc.group={}", group)],
    );

    // find all matching containers and then stop them using .then()
    Runtime::new().unwrap().block_on(
        docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: list_container_filters,
                ..Default::default()
            }))
            .then(|containers| async {
                debug!(target: "lazymc-docker-proxy::command::docker", "Found {} container(s) to stop", containers.as_ref().unwrap().len());
                for container in containers.unwrap() {
                    info!(target: "lazymc-docker-proxy::command::docker", "Stopping container: {}", container.names.unwrap().first().unwrap());
                    if let Err(err) = docker
                        .stop_container(
                            container.id.as_ref().unwrap(), 
                            None::<StopContainerOptions>
                        )
                        .await
                    {
                        error!(target: "lazymc-docker-proxy::command::docker", "Error stopping container: {}", err);
                    }
                }
                return future::ready(()).await;
            }),
    );
}

pub fn start(group: String) {
    debug!(target: "lazymc-docker-proxy::command::docker", "Starting containers...");
    let docker: Docker = Docker::connect_with_local_defaults().expect("Error connecting to docker");

    let mut list_container_filters: HashMap<String, Vec<String>> =
        HashMap::<String, Vec<String>>::new();

    // find all matching exited containers
    list_container_filters.insert("status".to_string(), vec!["exited".to_string()]);
    list_container_filters.insert(
        "label".to_string(),
        vec![format!("lazymc.group={}", group)],
    );

    // find all matching containers and then stop them using .then()
    Runtime::new().unwrap().block_on(
        docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: list_container_filters,
                ..Default::default()
            }))
            .then(|containers| async {
                debug!(target: "lazymc-docker-proxy::command::docker", "Found {} container(s) to start", containers.as_ref().unwrap().len());
                for container in containers.unwrap() {
                    info!(target: "lazymc-docker-proxy::command::docker", "Starting container: {}", container.names.unwrap().first().unwrap());
                    if let Err(err) = docker
                        .start_container(
                            container.id.as_ref().unwrap(),
                            None::<StartContainerOptions<&str>>,
                        )
                        .await
                    {
                        error!(target: "lazymc-docker-proxy::command::docker", "Error starting container: {}", err);
                    }
                }
                return future::ready(()).await;
            }),
    );
}
