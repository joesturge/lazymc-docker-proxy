use std::collections::HashMap;
use std::process::exit;

use bollard::container::{ListContainersOptions, StartContainerOptions, StopContainerOptions};
use bollard::Docker;
use futures::{future, FutureExt};
use log::error;
use tokio::runtime::Runtime;

use crate::health;

/// Connect to the docker daemon
pub fn connect() -> Docker {
    let docker: Docker = Docker::connect_with_local_defaults().unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::docker", "Error connecting to docker: {}", err);
        health::unhealthy();
        exit(1)
    });

    return docker;
}

/// Stop container with the label "lazymc.group=group"
pub fn stop(group: String) {
    debug!(target: "lazymc-docker-proxy::docker", "Stopping containers...");
    let docker: Docker = connect();

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
                debug!(target: "lazymc-docker-proxy::docker", "Found {} container(s) to stop", containers.as_ref().unwrap().len());
                for container in containers.unwrap() {
                    info!(target: "lazymc-docker-proxy::docker", "Stopping container: {}", container.names.unwrap().first().unwrap());
                    if let Err(err) = docker
                        .stop_container(
                            container.id.as_ref().unwrap(), 
                            None::<StopContainerOptions>
                        )
                        .await
                    {
                        error!(target: "lazymc-docker-proxy::docker", "Error stopping container: {}", err);
                    }
                }
                return future::ready(()).await;
            }),
    );
}

/// Start container with the label "lazymc.group=group"
pub fn start(group: String) {
    debug!(target: "lazymc-docker-proxy::docker", "Starting containers...");
    let docker: Docker = connect();

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
                debug!(target: "lazymc-docker-proxy::docker", "Found {} container(s) to start", containers.as_ref().unwrap().len());
                for container in containers.unwrap() {
                    info!(target: "lazymc-docker-proxy::docker", "Starting container: {}", container.names.unwrap().first().unwrap());
                    if let Err(err) = docker
                        .start_container(
                            container.id.as_ref().unwrap(),
                            None::<StartContainerOptions<&str>>,
                        )
                        .await
                    {
                        error!(target: "lazymc-docker-proxy::docker", "Error starting container: {}", err);
                    }
                }
                return future::ready(()).await;
            }),
    );
}

/// Stop all containers with the label "lazymc.enabled=true"
pub fn stop_all_containers() {
    let docker: Docker = connect();

    let mut list_container_filters: HashMap<String, Vec<String>> =
        HashMap::<String, Vec<String>>::new();

    // find all docker containers with the label "lazymc.enabled=true"
    list_container_filters.insert("label".to_string(), vec![format!("lazymc.enabled=true")]);

    // find all matching containers and then stop them
    let containers = Runtime::new().unwrap().block_on(
        docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: list_container_filters,
                ..Default::default()
            }))
            .then(|containers| async {
                debug!(target: "lazymc-docker-proxy::docker", "Found {} container(s) to stop", containers.as_ref().unwrap().len());
                return containers.unwrap();
            }),
    );

    for container in containers {
        container.id.as_ref().map(|id| {
            Runtime::new().unwrap().block_on(
                docker
                    .stop_container(id, None::<StopContainerOptions>)
                    .then(|result| async {
                        debug!(target: "lazymc-docker-proxy::docker", "Stopped container: {}", id);
                        return result.unwrap();
                    }),
            );
        });
    }

}

/// Get all labels for containers with the label "lazymc.enabled=true"
pub fn get_container_labels() -> Vec<HashMap<std::string::String, std::string::String>> {
    let docker: Docker = connect();

    let mut list_container_filters: HashMap<String, Vec<String>> =
        HashMap::<String, Vec<String>>::new();

    // find all docker containers with the label "lazymc.enabled=true"
    list_container_filters.insert("label".to_string(), vec![format!("lazymc.enabled=true")]);

    // find all matching containers and then get their labels
    let containers = Runtime::new().unwrap().block_on(
        docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: list_container_filters,
                ..Default::default()
            }))
            .then(|containers| async {
                debug!(target: "lazymc-docker-proxy::docker", "Found {} container(s) to get labels", containers.as_ref().unwrap().len());
                return containers.unwrap();
            }),
    );

    let mut label_sets: Vec<HashMap<String, String>> = Vec::new();

    for container in containers {
        let mut labels: HashMap<String, String> = HashMap::new();
        for (key, value) in container.labels.as_ref().unwrap() {
            labels.insert(key.clone(), value.replace("\\n", "\n"));
        }

        // parse port from lazymc.server.address label
        let port: Option<u16> = labels
            .get("lazymc.server.address")
            .and_then(|address| address.rsplit(':').next())
            .and_then(|port_str| port_str.parse().ok());

        // try to get the container's IP address from the ipam config as optional
        let ip_address: Option<String> = container
            .network_settings
            .as_ref()
            .and_then(|network_settings| {
                network_settings
                    .networks
                    .as_ref()
                    .and_then(|networks| {
                    networks
                        .values()
                        .find(|network| network.ipam_config.is_some())
                    })
                    .and_then(|network| network.ipam_config.as_ref())
                    .and_then(|ipam_config| ipam_config.ipv4_address.clone())
            })
            .or_else(|| {
                warn!(target: "lazymc-docker-proxy::docker", "**************************************************************************************************************************");
                warn!(target: "lazymc-docker-proxy::docker", "WARNING: You should use IPAM to assign a static IP address to your server container otherwise performance may be degraded.");
                warn!(target: "lazymc-docker-proxy::docker", "    see: https://github.com/joesturge/lazymc-docker-proxy?tab=readme-ov-file#usage");
                warn!(target: "lazymc-docker-proxy::docker", "**************************************************************************************************************************");
                None
            });

        // if we have a port and an IP address, add the resolved address to the labels
        if port.is_some() && ip_address.is_some() {
            let address = format!("{}:{}", ip_address.unwrap(), port.unwrap());
            debug!(target: "lazymc-docker-proxy::docker", "Resolved address: {}", address);
            labels.insert("lazymc.server.address".to_string(), address);
        }

        label_sets.push(labels);
    }

    return label_sets;
}
