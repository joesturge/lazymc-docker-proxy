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

    // find all matching running containers which have labels starting with "lazymc."
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
        label_sets.push(labels);
    }

    return label_sets;
}
