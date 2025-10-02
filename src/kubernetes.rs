use std::collections::HashMap;
use std::process::exit;

use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams, DeleteParams},
    Client,
};
use log::error;
use tokio::runtime::Runtime;

use crate::health;

/// Connect to the Kubernetes cluster
pub fn connect() -> Client {
    Runtime::new().unwrap().block_on(async {
        Client::try_default().await.unwrap_or_else(|err| {
            error!(target: "lazymc-docker-proxy::kubernetes", "Error connecting to Kubernetes: {}", err);
            health::unhealthy();
            exit(1)
        })
    })
}

/// Stop pods with the label "lazymc.group=group"
pub fn stop(group: String) {
    debug!(target: "lazymc-docker-proxy::kubernetes", "Stopping pods...");
    let client = connect();
    
    Runtime::new().unwrap().block_on(async {
        let pods: Api<Pod> = Api::default_namespaced(client);
        let lp = ListParams::default()
            .labels(&format!("lazymc.group={}", group));
        
        match pods.list(&lp).await {
            Ok(pod_list) => {
                debug!(target: "lazymc-docker-proxy::kubernetes", "Found {} pod(s) to stop", pod_list.items.len());
                for pod in pod_list.items {
                    if let Some(name) = pod.metadata.name.as_ref() {
                        info!(target: "lazymc-docker-proxy::kubernetes", "Stopping pod: {}", name);
                        if let Err(err) = pods.delete(name, &DeleteParams::default()).await {
                            error!(target: "lazymc-docker-proxy::kubernetes", "Error stopping pod: {}", err);
                        }
                    }
                }
            }
            Err(err) => {
                error!(target: "lazymc-docker-proxy::kubernetes", "Error listing pods: {}", err);
            }
        }
    });
}

/// Start pods with the label "lazymc.group=group"
/// Note: In Kubernetes, pods are typically managed by controllers (Deployments, StatefulSets, etc.)
/// This function scales up the controller instead of directly creating pods
pub fn start(group: String) {
    debug!(target: "lazymc-docker-proxy::kubernetes", "Starting pods...");
    let client = connect();
    
    Runtime::new().unwrap().block_on(async {
        use k8s_openapi::api::apps::v1::{Deployment, StatefulSet};
        use kube::api::Patch;
        use kube::api::PatchParams;
        
        // Try to find and scale up Deployment first
        let deployments: Api<Deployment> = Api::default_namespaced(client.clone());
        let lp = ListParams::default()
            .labels(&format!("lazymc.group={}", group));
        
        match deployments.list(&lp).await {
            Ok(deployment_list) => {
                debug!(target: "lazymc-docker-proxy::kubernetes", "Found {} deployment(s) to start", deployment_list.items.len());
                for deployment in deployment_list.items {
                    if let Some(name) = deployment.metadata.name.as_ref() {
                        info!(target: "lazymc-docker-proxy::kubernetes", "Starting deployment: {}", name);
                        let patch = serde_json::json!({
                            "spec": {
                                "replicas": 1
                            }
                        });
                        if let Err(err) = deployments.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await {
                            error!(target: "lazymc-docker-proxy::kubernetes", "Error starting deployment: {}", err);
                        }
                    }
                }
            }
            Err(err) => {
                error!(target: "lazymc-docker-proxy::kubernetes", "Error listing deployments: {}", err);
            }
        }
        
        // Also try StatefulSets
        let statefulsets: Api<StatefulSet> = Api::default_namespaced(client.clone());
        match statefulsets.list(&lp).await {
            Ok(statefulset_list) => {
                debug!(target: "lazymc-docker-proxy::kubernetes", "Found {} statefulset(s) to start", statefulset_list.items.len());
                for statefulset in statefulset_list.items {
                    if let Some(name) = statefulset.metadata.name.as_ref() {
                        info!(target: "lazymc-docker-proxy::kubernetes", "Starting statefulset: {}", name);
                        let patch = serde_json::json!({
                            "spec": {
                                "replicas": 1
                            }
                        });
                        if let Err(err) = statefulsets.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await {
                            error!(target: "lazymc-docker-proxy::kubernetes", "Error starting statefulset: {}", err);
                        }
                    }
                }
            }
            Err(err) => {
                error!(target: "lazymc-docker-proxy::kubernetes", "Error listing statefulsets: {}", err);
            }
        }
    });
}

/// Stop all pods with the label "lazymc.enabled=true"
pub fn stop_all_pods() {
    let client = connect();
    
    Runtime::new().unwrap().block_on(async {
        use k8s_openapi::api::apps::v1::{Deployment, StatefulSet};
        use kube::api::Patch;
        use kube::api::PatchParams;
        
        let lp = ListParams::default()
            .labels("lazymc.enabled=true");
        
        // Scale down all deployments
        let deployments: Api<Deployment> = Api::default_namespaced(client.clone());
        match deployments.list(&lp).await {
            Ok(deployment_list) => {
                debug!(target: "lazymc-docker-proxy::kubernetes", "Found {} deployment(s) to stop", deployment_list.items.len());
                for deployment in deployment_list.items {
                    if let Some(name) = deployment.metadata.name.as_ref() {
                        let patch = serde_json::json!({
                            "spec": {
                                "replicas": 0
                            }
                        });
                        if let Err(err) = deployments.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await {
                            error!(target: "lazymc-docker-proxy::kubernetes", "Error stopping deployment: {}", err);
                        } else {
                            debug!(target: "lazymc-docker-proxy::kubernetes", "Stopped deployment: {}", name);
                        }
                    }
                }
            }
            Err(err) => {
                error!(target: "lazymc-docker-proxy::kubernetes", "Error listing deployments: {}", err);
            }
        }
        
        // Scale down all statefulsets
        let statefulsets: Api<StatefulSet> = Api::default_namespaced(client.clone());
        match statefulsets.list(&lp).await {
            Ok(statefulset_list) => {
                debug!(target: "lazymc-docker-proxy::kubernetes", "Found {} statefulset(s) to stop", statefulset_list.items.len());
                for statefulset in statefulset_list.items {
                    if let Some(name) = statefulset.metadata.name.as_ref() {
                        let patch = serde_json::json!({
                            "spec": {
                                "replicas": 0
                            }
                        });
                        if let Err(err) = statefulsets.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await {
                            error!(target: "lazymc-docker-proxy::kubernetes", "Error stopping statefulset: {}", err);
                        } else {
                            debug!(target: "lazymc-docker-proxy::kubernetes", "Stopped statefulset: {}", name);
                        }
                    }
                }
            }
            Err(err) => {
                error!(target: "lazymc-docker-proxy::kubernetes", "Error listing statefulsets: {}", err);
            }
        }
    });
}

/// Get all labels for pods with the label "lazymc.enabled=true"
pub fn get_pod_labels() -> Vec<HashMap<std::string::String, std::string::String>> {
    let client = connect();
    
    Runtime::new().unwrap().block_on(async {
        let pods: Api<Pod> = Api::default_namespaced(client);
        let lp = ListParams::default()
            .labels("lazymc.enabled=true");
        
        match pods.list(&lp).await {
            Ok(pod_list) => {
                debug!(target: "lazymc-docker-proxy::kubernetes", "Found {} pod(s) to get labels", pod_list.items.len());
                
                let mut label_sets: Vec<HashMap<String, String>> = Vec::new();
                
                for pod in pod_list.items {
                    if let Some(labels) = pod.metadata.labels {
                        let mut processed_labels: HashMap<String, String> = HashMap::new();
                        for (key, value) in labels {
                            processed_labels.insert(key.clone(), value.replace("\\n", "\n"));
                        }
                        
                        // Get pod IP if available
                        if let Some(status) = pod.status {
                            if let Some(pod_ip) = status.pod_ip {
                                // Parse port from lazymc.server.address label if it exists
                                let port: Option<u16> = processed_labels
                                    .get("lazymc.server.address")
                                    .and_then(|address| address.rsplit(':').next())
                                    .and_then(|port_str| port_str.parse().ok());
                                
                                if let Some(port) = port {
                                    let address = format!("{}:{}", pod_ip, port);
                                    debug!(target: "lazymc-docker-proxy::kubernetes", "Resolved address: {}", address);
                                    processed_labels.insert("lazymc.server.address".to_string(), address);
                                }
                            }
                        }
                        
                        label_sets.push(processed_labels);
                    }
                }
                
                return label_sets;
            }
            Err(err) => {
                error!(target: "lazymc-docker-proxy::kubernetes", "Error listing pods: {}", err);
                return Vec::new();
            }
        }
    })
}
