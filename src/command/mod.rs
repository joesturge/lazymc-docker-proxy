use std::collections::HashMap;

use bollard::container::{ListContainersOptions, StartContainerOptions, StopContainerOptions};
use bollard::Docker;
use futures::FutureExt;

pub fn lazymc_group() -> String {
    return std::env::var("LAZYMC_GROUP").expect("LAZYMC_GROUP must be set");
}

pub fn stop() {
    let docker: Docker = Docker::connect_with_local_defaults().expect("Error connecting to docker");

    let mut list_container_filters: HashMap<String, Vec<String>> =
        HashMap::<String, Vec<String>>::new();

    // find all matching running containers
    list_container_filters.insert("status".to_string(), vec!["running".to_string()]);
    list_container_filters.insert("label".to_string(), vec![format!("lazymc.group={}", lazymc_group())]);

    // find all matching containers and then stop them using .then()
    tokio::runtime::Runtime::new().unwrap().block_on(
        docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: list_container_filters,
                ..Default::default()
            }))
            .then(|containers| async {
                for container in containers.unwrap() {
                    if let Err(err) = docker
                        .stop_container(&container.id.unwrap(), None::<StopContainerOptions>)
                        .await
                    {
                        println!("Error stopping container: {:?}", err);
                    }
                }
                return futures::future::ready(()).await;
            }),
    );
}

pub fn start() {
    let docker: Docker = Docker::connect_with_local_defaults().expect("Error connecting to docker");

    let mut list_container_filters: HashMap<String, Vec<String>> =
        HashMap::<String, Vec<String>>::new();

    // find all matching exited containers
    list_container_filters.insert("status".to_string(), vec!["exited".to_string()]);
    list_container_filters.insert("label".to_string(), vec![format!("lazymc.group={}", lazymc_group())]);

    // find all matching containers and then stop them using .then()
    tokio::runtime::Runtime::new().unwrap().block_on(
        docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: list_container_filters,
                ..Default::default()
            }))
            .then(|containers| async {
                for container in containers.unwrap() {
                    if let Err(err) = docker
                        .start_container(
                            &container.id.unwrap(),
                            None::<StartContainerOptions<&str>>,
                        )
                        .await
                    {
                        println!("Error starting container: {:?}", err);
                    }
                }
                return futures::future::ready(()).await;
            }),
    );
}
