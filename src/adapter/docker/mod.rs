use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::exit;

use bollard::container::{ListContainersOptions, StartContainerOptions, StopContainerOptions};
use bollard::Docker;
use futures::{future, FutureExt};
use log::error;
use tokio::runtime::Runtime;

use crate::adapter::Adapter;
use crate::health::unhealthy;

pub struct DockerAdapter;

impl DockerAdapter {
    fn connect() -> Docker {
        Docker::connect_with_local_defaults().unwrap_or_else( | err| {
            error!(target: "lazymc-docker-proxy::adapter::docker", "Error connecting to docker: {}", err);
            unhealthy();
            exit(1);
        })
    }
}

impl Adapter for DockerAdapter {
    fn stop(group: &String) {
        debug!(target: "lazymc-docker-proxy::adapter::docker", "Stopping containers...");

        let mut list_container_filters: HashMap<String, Vec<String>> =
            HashMap::<String, Vec<String>>::new();

        // find all matching running containers
        list_container_filters.insert(String::from("status"), vec![String::from("running")]);
        list_container_filters.insert(String::from("label"), vec![format!("lazymc.group={}", group)]);

        let connection = DockerAdapter::connect();

        // find all matching containers and then stop them using .then()
        Runtime::new().unwrap().block_on(
            connection
                .list_containers(Some(ListContainersOptions {
                    all: true,
                    filters: list_container_filters,
                    ..Default::default()
                }))
                .then(|containers| async {
                    debug!(target: "lazymc-docker-proxy::adapter::docker", "Found {} container(s) to stop", containers.as_ref().unwrap().len());
                    for container in containers.unwrap() {
                        info!(target: "lazymc-docker-proxy::adapter::docker", "Stopping container: {}", container.names.unwrap().first().unwrap());
                        if let Err(err) = connection
                            .stop_container(
                                container.id.as_ref().unwrap(),
                                None::<StopContainerOptions>
                            )
                            .await
                        {
                            error!(target: "lazymc-docker-proxy::adapter::docker", "Error stopping container: {}", err);
                        }
                    }
                    return future::ready(()).await;
                }),
        );
    }

    fn start(group: &String) {
        debug!(target: "lazymc-docker-proxy::adapter::docker", "Starting containers...");

        let mut list_container_filters: HashMap<String, Vec<String>> =
            HashMap::<String, Vec<String>>::new();

        // find all matching exited containers
        list_container_filters.insert(String::from("status"), vec![String::from("exited")]);
        list_container_filters.insert(String::from("label"), vec![format!("lazymc.group={}", group)]);

        let connection = DockerAdapter::connect();

        // find all matching containers and then stop them using .then()
        Runtime::new().unwrap().block_on(
            connection
                .list_containers(Some(ListContainersOptions {
                    all: true,
                    filters: list_container_filters,
                    ..Default::default()
                }))
                .then(|containers| async {
                    debug!(target: "lazymc-docker-proxy::adapter::docker", "Found {} container(s) to start", containers.as_ref().unwrap().len());
                    for container in containers.unwrap() {
                        info!(target: "lazymc-docker-proxy::adapter::docker", "Starting container: {}", container.names.unwrap().first().unwrap());
                        if let Err(err) = connection
                            .start_container(
                                container.id.as_ref().unwrap(),
                                None::<StartContainerOptions<&str>>,
                            )
                            .await
                        {
                            error!(target: "lazymc-docker-proxy::adapter::docker", "Error starting container: {}", err);
                        }
                    }
                    return future::ready(()).await;
                }),
        );
    }

    fn stop_all_containers() {
        let mut list_container_filters: HashMap<String, Vec<String>> =
            HashMap::<String, Vec<String>>::new();

        // find all docker containers with the label "lazymc.enabled=true"
        list_container_filters.insert(String::from("label"), vec![String::from("lazymc.enabled=true")]);

        let connection = DockerAdapter::connect();

        // find all matching containers and then stop them
        let containers = Runtime::new().unwrap().block_on(
            connection
                .list_containers(Some(ListContainersOptions {
                    all: true,
                    filters: list_container_filters,
                    ..Default::default()
                }))
                .then(|containers| async {
                    debug!(target: "lazymc-docker-proxy::adapter::docker", "Found {} container(s) to stop", containers.as_ref().unwrap().len());
                    return containers.unwrap();
                }),
        );

        for container in containers {
            container.id.as_ref().map(|id| {
                Runtime::new()
                    .unwrap()
                    .block_on(connection.stop_container(id, None).then(|result| async {
                        debug!(target: "lazymc-docker-proxy::adapter::docker", "Stopped container: {}", id);
                        return result.unwrap();
                    }));
            });
        }
    }

    fn get_container_labels() -> Vec<HashMap<String, String>> {
        let mut list_container_filters: HashMap<String, Vec<String>> =
            HashMap::<String, Vec<String>>::new();

        // find all docker containers with the label "lazymc.enabled=true"
        list_container_filters.insert(String::from("label"), vec![String::from("lazymc.enabled=true")]);

        let connection = DockerAdapter::connect();

        // find all matching containers and then get their labels
        let containers = Runtime::new().unwrap().block_on(
            connection
                .list_containers(Some(ListContainersOptions {
                    all: true,
                    filters: list_container_filters,
                    ..Default::default()
                }))
                .then(|containers| async {
                    debug!(target: "lazymc-docker-proxy::adapter::docker", "Found {} container(s) to get labels", containers.as_ref().unwrap().len());
                    return containers.unwrap();
                }),
        );

        let mut label_sets: Vec<HashMap<String, String>> = Vec::new();

        for container in containers {
            let mut labels: HashMap<String, String> = HashMap::new();
            for (key, value) in container.labels.as_ref().unwrap() {
                labels.insert(key.clone(), value.clone());
            }

            // parse port from lazymc.server.address label
            let port = labels
                .get("lazymc.server.address")
                .and_then(|address| {
                    address.parse::<SocketAddr>()
                        .map(|address| address.port())
                        .map_err(|err| {
                            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error parsing container address: {}", err);
                        }).ok()
                });

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
                    warn!(target: "lazymc-docker-proxy::adapter::docker", "**************************************************************************************************************************");
                    warn!(target: "lazymc-docker-proxy::adapter::docker", "WARNING: You should use IPAM to assign a static IP address to your server container otherwise performance may be degraded.");
                    warn!(target: "lazymc-docker-proxy::adapter::docker", "    see: https://github.com/joesturge/lazymc-docker-proxy?tab=readme-ov-file#usage");
                    warn!(target: "lazymc-docker-proxy::adapter::docker", "**************************************************************************************************************************");
                    None
                });

            // if we have a port and an IP address, add the resolved address to the labels
            if port.is_some() && ip_address.is_some() {
                let address = format!("{}:{}", ip_address.unwrap(), port.unwrap());
                debug!(target: "lazymc-docker-proxy::adapter::docker", "Resolved address: {}", address);
                labels.insert(String::from("lazymc.server.address"), address);
            }

            label_sets.push(labels);
        }

        label_sets
    }
}
