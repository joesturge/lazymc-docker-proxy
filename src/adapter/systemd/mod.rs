use crate::adapter::systemd::proxy::{ManagerProxy, ServiceProxy, UnitProxy};
use crate::adapter::Adapter;
use crate::entrypoint::config::Config;
use crate::health::unhealthy;
use futures::{future, FutureExt};
use std::collections::HashMap;
use std::env::var;
use std::error::Error;
use std::process::exit;
use tokio::runtime::Runtime;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;
use zbus::connection::Builder;

mod proxy;

pub struct SystemdAdapter;

struct ContainerService {
    name: String,
    path: OwnedObjectPath,
    environment: HashMap<String, String>,
}

const UNIT_START_MODE: &str = "replace";
const UNIT_STOP_MODE: &str = "replace";

impl SystemdAdapter {
    async fn connect() -> Connection {
        let connection = match var("DBUS_TARGET") {
            Ok(dbus_target) if dbus_target == "system" => Ok(Connection::system().await),
            Ok(dbus_target) if dbus_target == "session" || dbus_target == "user" => {
                Ok(Connection::session().await)
            }
            Ok(dbus_target) => {
                Ok(Builder::address(dbus_target.as_str()).unwrap().build().await)
            }
            Err(dbus_target) => Err(dbus_target),
        };

        connection.unwrap_or_else(|err| {
            error!(target: "lazymc-docker-proxy::adapter::systemd", "Environment variable DBUS_TARGET is invalid: {}", err);
            unhealthy();
            exit(1);
        }).unwrap_or_else( | err| {
            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error connecting to dbus: {}", err);
            unhealthy();
            exit(1);
        })
    }

    async fn list_containers(
        connection: &Connection,
        required_environment: Vec<(&str, &str)>,
    ) -> Result<Vec<ContainerService>, Box<dyn Error>> {
        let manager_proxy = ManagerProxy::new(connection, "/org/freedesktop/systemd1").await?;

        let list_units = manager_proxy.list_units().await?;

        let container_services_tasks = list_units.into_iter().map(|unit| {
            tokio::spawn({
                let connection = connection.clone();

                async move {
                    let service = ServiceProxy::new(&connection, &unit.path).await.unwrap();

                    service
                        .environment()
                        .await
                        .ok()
                        .map(|environment| ContainerService {
                            name: unit.name,
                            path: unit.path,
                            environment: environment
                                .iter()
                                .filter_map(|environment| environment.split_once("="))
                                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                                .collect(),
                        })
                }
            })
        });

        let container_services_results = futures::future::join_all(container_services_tasks).await;

        let container_services = container_services_results
            .into_iter()
            .filter_map(|service| {
                service
                    .unwrap_or_else(|err| {
                        error!(target: "lazymc-docker-proxy::adapter::systemd", "Error joining service future: {}", err);
                        None
                    })
                    .filter(|service| {
                        required_environment.iter().all(|(key, value)| {
                            service.environment.get(key.to_owned()).map(String::as_str) == Some(value.to_owned())
                        })
                    })
            })
            .collect();

        Ok(container_services)
    }
}

impl Adapter for SystemdAdapter {
    fn stop(group: &String) {
        debug!(target: "lazymc-docker-proxy::adapter::systemd", "Stopping containers...");

        Runtime::new().unwrap().block_on(async {
            let connection = SystemdAdapter::connect().await;

            let container_required_environment = vec![
                ("LAZYMC_ENABLED", "true"),
                ("LAZYMC_GROUP", group),
            ];

            SystemdAdapter::list_containers(&connection, container_required_environment)
                .then(|containers| async {
                    match containers {
                        Ok(containers) => {
                            debug!(target: "lazymc-docker-proxy::adapter::systemd", "Found {} container(s) to stop", containers.len());

                            for container in containers {
                                info!(target: "lazymc-docker-proxy::adapter::systemd", "Stopping container: {}", container.name);

                                match UnitProxy::new(&connection, &container.path).await {
                                    Ok(unit_proxy) => {
                                        if let Err(err) = unit_proxy.stop(&String::from(UNIT_STOP_MODE)).await {
                                            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error stopping container: {}", err);
                                        }
                                    }
                                    Err(err) => {
                                        error!(target: "lazymc-docker-proxy::adapter::systemd", "Error connecting to container: {}", err)
                                    },
                                };
                            }
                        }
                        Err(err) => {
                            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error listing containers: {}", err);
                        }
                    }
                    future::ready(()).await;
                }).await
        })
    }

    fn start(group: &String) {
        debug!(target: "lazymc-docker-proxy::adapter::systemd", "Starting containers...");

        Runtime::new().unwrap().block_on(async {
            let connection = SystemdAdapter::connect().await;

            let container_required_environment = vec![
                ("LAZYMC_ENABLED", "true"),
                ("LAZYMC_GROUP", group),
            ];

            SystemdAdapter::list_containers(&connection, container_required_environment)
                .then(|containers| async {
                    match containers {
                        Ok(containers) => {
                            debug!(target: "lazymc-docker-proxy::adapter::systemd", "Found {} container(s) to start", containers.len());

                            for container in containers {
                                info!(target: "lazymc-docker-proxy::adapter::systemd", "Starting container: {}", container.name);

                                match UnitProxy::new(&connection, &container.path).await {
                                    Ok(unit_proxy) => {
                                        if let Err(err) = unit_proxy.start(&String::from(UNIT_START_MODE)).await {
                                            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error starting container: {}", err);
                                        }
                                    }
                                    Err(err) => {
                                        error!(target: "lazymc-docker-proxy::adapter::systemd", "Error connecting to container: {}", err)
                                    },
                                };
                            }
                        }
                        Err(err) => {
                            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error listing containers: {}", err);
                        }
                    }
                    future::ready(()).await;
                }).await
        })
    }

    fn stop_all_containers() {
        debug!(target: "lazymc-docker-proxy::adapter::systemd", "Stopping all containers...");

        Runtime::new().unwrap().block_on(async {
            let connection = SystemdAdapter::connect().await;

            let container_required_environment = vec![
                ("LAZYMC_ENABLED", "true"),
            ];

            SystemdAdapter::list_containers(&connection, container_required_environment)
                .then(|containers| async {
                    match containers {
                        Ok(containers) => {
                            debug!(target: "lazymc-docker-proxy::adapter::systemd", "Found all {} container(s) to stop", containers.len());

                            for container in containers {
                                info!(target: "lazymc-docker-proxy::adapter::systemd", "Stopping container: {}", container.name);

                                match UnitProxy::new(&connection, &container.path).await {
                                    Ok(unit_proxy) => {
                                        if let Err(err) = unit_proxy.start(&String::from(UNIT_STOP_MODE)).await {
                                            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error stopping container: {}", err);
                                        }
                                    }
                                    Err(err) => {
                                        error!(target: "lazymc-docker-proxy::adapter::systemd", "Error connecting to container: {}", err)
                                    },
                                };
                            }
                        }
                        Err(err) => {
                            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error listing containers: {}", err);
                        }
                    }
                    future::ready(()).await;
                }).await
        })
    }

    fn get_container_labels() -> Vec<HashMap<String, String>> {
        debug!(target: "lazymc-docker-proxy::adapter::systemd", "Retrieving container labels...");

        let containers = Runtime::new().unwrap().block_on(async {
            let connection = SystemdAdapter::connect().await;

            let container_required_environment = vec![("LAZYMC_ENABLED", "true")];

            SystemdAdapter::list_containers(&connection, container_required_environment).await
        });

        containers.map_or_else(|err| {
            error!(target: "lazymc-docker-proxy::adapter::systemd", "Error listing containers: {}", err);
            Vec::new()
        }, |containers| {
            debug!(target: "lazymc-docker-proxy::adapter::systemd", "Found {} container(s) for labels", containers.len());
            containers
                .into_iter()
                .map(|container| Config::environment_to_label(container.environment))
                .collect()
        })
    }
}
