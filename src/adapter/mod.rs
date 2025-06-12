use std::collections::HashMap;

pub mod docker;
pub mod systemd;

pub trait Adapter {
    /// Stop container with the label "lazymc.group=group"
    fn stop(group: &String);

    /// Start container with the label "lazymc.group=group"
    fn start(group: &String);

    /// Stop all containers with the label "lazymc.enabled=true"
    fn stop_all_containers();

    /// Get all labels for containers with the label "lazymc.enabled=true"
    fn get_container_labels() -> Vec<HashMap<String, String>>;
}
