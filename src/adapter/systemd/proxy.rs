use serde::{Deserialize, Serialize};
use zbus::proxy;
use zbus::zvariant::{OwnedObjectPath, Type};

#[derive(Type, Serialize, Deserialize)]
pub struct ListUnitEntry {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub activate_state: String,
    pub sub_state: String,
    pub following: String,
    pub path: OwnedObjectPath,
    pub job_id: u32,
    pub job_type: String,
    pub job_path: OwnedObjectPath,
}

#[proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.systemd1.Manager"
)]
pub trait Manager {
    /// Get list of all `Unit`'s
    async fn list_units(
        &self,
    ) -> zbus::Result<Vec<ListUnitEntry>>;
}

#[proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.systemd1.Service"
)]
pub trait Service {
    /// Get list of `Environment` properties
    #[zbus(property)]
    fn environment(&self) -> zbus::Result<Vec<String>>;
}

#[proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.systemd1.Unit"
)]
pub trait Unit {
    /// Starts the unit - returns `org.freedesktop.systemd1.Job` path
    fn start(&self, mode: &String) -> zbus::Result<OwnedObjectPath>;

    /// Stops the unit - returns `org.freedesktop.systemd1.Job` path
    fn stop(&self, mode: &String) -> zbus::Result<OwnedObjectPath>;
}