use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Backend types supported by lazymc-docker-proxy
#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    Docker,
    Kubernetes,
}

impl BackendType {
    /// Detect backend type from environment variable or auto-detect
    pub fn detect() -> Self {
        // First check if explicitly set via environment variable
        if let Ok(backend) = env::var("LAZYMC_BACKEND") {
            return match backend.as_str() {
                "kubernetes" | "k8s" => BackendType::Kubernetes,
                "docker" => BackendType::Docker,
                _ => {
                    warn!(target: "lazymc-docker-proxy::backend", "Unknown LAZYMC_BACKEND value: {}, falling back to auto-detection", backend);
                    Self::auto_detect()
                }
            };
        }
        
        // Auto-detect based on environment
        Self::auto_detect()
    }
    
    /// Auto-detect if running in Kubernetes or Docker
    fn auto_detect() -> Self {
        // Check for Kubernetes service account token (standard location in k8s pods)
        if Path::new("/var/run/secrets/kubernetes.io/serviceaccount/token").exists() {
            info!(target: "lazymc-docker-proxy::backend", "Auto-detected Kubernetes environment");
            return BackendType::Kubernetes;
        }
        
        // Check for Docker socket
        if Path::new("/var/run/docker.sock").exists() {
            info!(target: "lazymc-docker-proxy::backend", "Auto-detected Docker environment");
            return BackendType::Docker;
        }
        
        // Default to Docker for backward compatibility
        warn!(target: "lazymc-docker-proxy::backend", "Could not auto-detect environment, defaulting to Docker");
        BackendType::Docker
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
pub fn create() -> Box<dyn Backend> {
    match BackendType::detect() {
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
