use std::collections::HashMap;
use std::env;

/// Backend types supported by lazymc-docker-proxy
#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    Docker,
    Kubernetes,
}

impl BackendType {
    /// Detect backend type from environment variable
    pub fn from_env() -> Self {
        match env::var("LAZYMC_BACKEND").as_ref().map(|s| s.as_str()) {
            Ok("kubernetes") | Ok("k8s") => BackendType::Kubernetes,
            _ => BackendType::Docker, // Default to Docker for backward compatibility
        }
    }
}

/// Backend trait that both Docker and Kubernetes implementations must follow
pub trait Backend: Send + Sync {
    fn stop(&self, group: String);
    fn start(&self, group: String);
    fn stop_all(&self);
    fn get_labels(&self) -> Vec<HashMap<String, String>>;
}

/// Docker backend implementation
pub struct DockerBackend;

impl Backend for DockerBackend {
    fn stop(&self, group: String) {
        crate::docker::stop(group);
    }
    
    fn start(&self, group: String) {
        crate::docker::start(group);
    }
    
    fn stop_all(&self) {
        crate::docker::stop_all_containers();
    }
    
    fn get_labels(&self) -> Vec<HashMap<String, String>> {
        crate::docker::get_container_labels()
    }
}

/// Kubernetes backend implementation
pub struct KubernetesBackend;

impl Backend for KubernetesBackend {
    fn stop(&self, group: String) {
        crate::kubernetes::stop(group);
    }
    
    fn start(&self, group: String) {
        crate::kubernetes::start(group);
    }
    
    fn stop_all(&self) {
        crate::kubernetes::stop_all_pods();
    }
    
    fn get_labels(&self) -> Vec<HashMap<String, String>> {
        crate::kubernetes::get_pod_labels()
    }
}

/// Get the appropriate backend based on environment configuration
pub fn get_backend() -> Box<dyn Backend> {
    match BackendType::from_env() {
        BackendType::Docker => {
            info!(target: "lazymc-docker-proxy::backend", "Using Docker backend");
            Box::new(DockerBackend)
        }
        BackendType::Kubernetes => {
            info!(target: "lazymc-docker-proxy::backend", "Using Kubernetes backend");
            Box::new(KubernetesBackend)
        }
    }
}
