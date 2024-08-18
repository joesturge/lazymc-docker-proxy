use std::collections::HashMap;
use std::process::exit;

use bollard::container::ListContainersOptions;
use bollard::Docker;
use futures::FutureExt;
use log::error;
use tokio::runtime::Runtime;

pub fn get_container_labels() -> Vec<HashMap<std::string::String, std::string::String>> {
    let docker: Docker = Docker::connect_with_local_defaults().unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::entrypoint::docker", "Error connecting to docker: {}", err);
        exit(1)
    });

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
                debug!(target: "lazymc-docker-proxy::entrypoint::docker", "Found {} container(s) to get labels", containers.as_ref().unwrap().len());
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
                warn!(target: "lazymc-docker-proxy::entrypoint::docker", "**************************************************************************************************************************");
                warn!(target: "lazymc-docker-proxy::entrypoint::docker", "WARNING: You should use IPAM to assign a static IP address to your server container otherwise performance may be degraded.");
                warn!(target: "lazymc-docker-proxy::entrypoint::docker", "    see: https://github.com/joesturge/lazymc-docker-proxy?tab=readme-ov-file#usage");
                warn!(target: "lazymc-docker-proxy::entrypoint::docker", "**************************************************************************************************************************");
                None
            });

        // if we have a port and an IP address, add the resolved address to the labels
        if port.is_some() && ip_address.is_some() {
            let address = format!("{}:{}", ip_address.unwrap(), port.unwrap());
            debug!(target: "lazymc-docker-proxy::entrypoint::docker", "Resolved address: {}", address);
            labels.insert("lazymc.server.address".to_string(), address);
        }

        label_sets.push(labels);
    }

    return label_sets;
}
