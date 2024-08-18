use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::var;
use std::fs::File;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::process::{exit, Command};
use version_compare::Version;

const DEFAULT_PORT: i32 = 25565;

/// lazymc dropped support minecraft servers with version less than 1.20.3
fn is_legacy(version: Option<String>) -> bool {
    if version.is_none() {
        return false;
    }

    return Version::from(version.unwrap().as_ref()) < Version::from("1.20.3");
}

#[derive(Serialize, Deserialize)]
struct ServerSection {
    address: Option<String>,
    block_banned_ips: Option<bool>,
    command: Option<String>,
    directory: Option<String>,
    drop_banned_ips: Option<bool>,
    forge: Option<bool>,
    freeze_process: Option<bool>,
    probe_on_start: Option<bool>,
    send_proxy_v2: Option<bool>,
    wake_on_crash: Option<bool>,
    wake_on_start: Option<bool>,
    wake_whitelist: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct PublicSection {
    address: Option<String>,
    version: Option<String>,
    protocol: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct TimeSection {
    minimum_online_time: Option<i32>,
    sleep_after: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct MotdSection {
    sleeping: Option<String>,
    starting: Option<String>,
    stopping: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AdvancedSection {
    rewrite_server_properties: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct ConfigSection {
    version: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    advanced: AdvancedSection,
    config: ConfigSection,
    motd: MotdSection,
    public: PublicSection,
    server: ServerSection,
    time: TimeSection,
    #[serde(skip)]
    start_command: String,
    #[serde(skip)]
    config_file: String,
    #[serde(skip)]
    group: String,
}

impl Config {
    pub fn start_command(&self) -> Command {
        let mut command: Command = Command::new(self.start_command.clone());
        command.arg("start");
        command.arg("--config");
        command.arg(self.config_file.clone());
        return command;
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    fn as_toml_string(&self) -> String {
        toml::to_string(self).unwrap()
    }

    fn create_file(&self) {
        let toml = self.as_toml_string();
        let file_name: &String = &format!("lazymc.{}.toml", self.group.clone());
        let path: &Path = Path::new(file_name);
        let mut file = File::create(path).unwrap();
        file.write(toml.as_ref()).unwrap();
        debug!(target: "lazymc-docker-proxy::entrypoint::config", "`generated`: {}\n\n{}", path.display(), toml);
    }

    pub fn from_container_labels(labels: HashMap<String, String>) -> Self {
        // Check for required labels
        labels.get("lazymc.server.address").unwrap_or_else(|| {
            error!(target: "lazymc-docker-proxy::entrypoint::config", "lazymc.server.address is not set");
            exit(1);
        });
        labels.get("lazymc.group").unwrap_or_else(|| {
            error!(target: "lazymc-docker-proxy::entrypoint::config", "lazymc.group is not set");
            exit(1);
        });

        let server_section: ServerSection = ServerSection {
            address: labels.get("lazymc.server.address")
                .and_then(|address| address.to_socket_addrs().ok())
                .and_then(|addrs| addrs.filter(|addr| addr.is_ipv4()).next())
                .and_then(|addr| addr.to_string().parse().ok())
                .or_else(|| {
                    error!(target: "lazymc-docker-proxy::entrypoint::config", "Failed to resolve IP address from lazymc.server.address");
                    exit(1);
                }),
            directory: Some(
                labels
                    .get("lazymc.server.directory")
                    .cloned()
                    .unwrap_or_else(|| "/server".to_string()),
            ),
            command: Some(format!(
                "lazymc-docker-proxy --command --group {}",
                labels.get("lazymc.group").unwrap()
            )),
            freeze_process: Some(false),
            wake_on_start: Some(true),
            wake_on_crash: Some(true),
            wake_whitelist: labels
                .get("lazymc.server.wake_whitelist")
                .map(|x| x == "true"),
            block_banned_ips: labels
                .get("lazymc.server.block_banned_ips")
                .map(|x| x == "true"),
            drop_banned_ips: labels
                .get("lazymc.server.drop_banned_ips")
                .map(|x| x == "true"),
            probe_on_start: labels
                .get("lazymc.server.probe_on_start")
                .map(|x| x == "true"),
            forge: labels.get("lazymc.server.forge").map(|x| x == "true"),
            send_proxy_v2: labels
                .get("lazymc.server.send_proxy_v2")
                .map(|x| x == "true"),
        };

        let time_section: TimeSection = TimeSection {
            sleep_after: labels
                .get("lazymc.time.sleep_after")
                .and_then(|x| x.parse().ok()),
            minimum_online_time: labels
                .get("lazymc.time.minimum_online_time")
                .and_then(|x| x.parse().ok()),
        };

        let public_section: PublicSection = PublicSection {
            address: Some(format!(
                "0.0.0.0:{}",
                labels
                    .get("lazymc.port")
                    .cloned()
                    .unwrap_or_else(|| DEFAULT_PORT.to_string())
            )),
            version: labels.get("lazymc.public.version").cloned(),
            protocol: labels
                .get("lazymc.public.protocol")
                .and_then(|x| x.parse().ok()),
        };

        let motd_section: MotdSection = MotdSection {
            sleeping: labels.get("lazymc.motd.sleeping").cloned(),
            starting: labels.get("lazymc.motd.starting").cloned(),
            stopping: labels.get("lazymc.motd.stopping").cloned(),
        };

        let advanced_section: AdvancedSection = AdvancedSection {
            rewrite_server_properties: Some(false),
        };

        let config_section: ConfigSection = ConfigSection {
            version: match is_legacy(labels.get("lazymc.public.version").cloned()) {
                true => var("LAZYMC_LEGACY_VERSION")
                    .unwrap_or_else(|err| {
                        error!(target: "lazymc-docker-proxy::entrypoint::config", "LAZYMC_LEGACY_VERSION is not set: {}", err);
                        exit(1);
                    })
                    .into(),
                false => var("LAZYMC_VERSION")
                    .unwrap_or_else(|err| {
                        error!(target: "lazymc-docker-proxy::entrypoint::config", "LAZYMC_VERSION is not set: {}", err);
                        exit(1);
                    })
                    .into(),
            },
        };

        let config: Config = Config {
            server: server_section,
            public: public_section,
            time: time_section,
            motd: motd_section,
            advanced: advanced_section,
            config: config_section,
            start_command: match is_legacy(labels.get("lazymc.public.version").cloned()) {
                true => format!("lazymc-legacy"),
                false => format!("lazymc"),
            },
            config_file: format!(
                "lazymc.{}.toml",
                labels.get("lazymc.group").unwrap().clone()
            ),
            group: labels.get("lazymc.group").unwrap().clone(),
        };

        // Generate the lazymc config file
        config.create_file();

        return config;
    }

    #[deprecated(since = "2.1.0", note = "Use `from_container_labels` instead")]
    pub fn from_env() -> Self {
        warn!(target: "lazymc-docker-proxy::entrypoint::config", "***************************************************************************************************************");
        warn!(target: "lazymc-docker-proxy::entrypoint::config", "DEPRECATED: Using Environment Variables to configure lazymc is deprecated. Please use container labels instead.");
        warn!(target: "lazymc-docker-proxy::entrypoint::config", "       see: https://github.com/joesturge/lazymc-docker-proxy?tab=readme-ov-file#usage");
        warn!(target: "lazymc-docker-proxy::entrypoint::config", "***************************************************************************************************************");

        let mut labels: HashMap<String, String> = HashMap::new();
        if let Ok(value) = var("LAZYMC_GROUP") {
            labels.insert("lazymc.group".to_string(), value);
        }
        if let Ok(value) = var("LAZYMC_PORT") {
            labels.insert("lazymc.port".to_string(), value);
        }
        if let Ok(value) = var("MOTD_SLEEPING") {
            labels.insert("lazymc.motd.sleeping".to_string(), value);
        }
        if let Ok(value) = var("MOTD_STARTING") {
            labels.insert("lazymc.motd.starting".to_string(), value);
        }
        if let Ok(value) = var("MOTD_STOPPING") {
            labels.insert("lazymc.motd.stopping".to_string(), value);
        }
        if let Ok(value) = var("PUBLIC_PROTOCOL") {
            labels.insert("lazymc.public.protocol".to_string(), value);
        }
        if let Ok(value) = var("PUBLIC_VERSION") {
            labels.insert("lazymc.public.version".to_string(), value);
        }
        if let Ok(value) = var("SERVER_ADDRESS") {
            labels.insert("lazymc.server.address".to_string(), value);
        }
        if let Ok(value) = var("SERVER_BLOCK_BANNED_IPS") {
            labels.insert("lazymc.server.block_banned_ips".to_string(), value);
        }
        if let Ok(value) = var("SERVER_DIRECTORY") {
            labels.insert("lazymc.server.directory".to_string(), value);
        }
        if let Ok(value) = var("SERVER_DROP_BANNED_IPS") {
            labels.insert("lazymc.server.drop_banned_ips".to_string(), value);
        }
        if let Ok(value) = var("SERVER_FORGE") {
            labels.insert("lazymc.server.forge".to_string(), value);
        }
        if let Ok(value) = var("SERVER_PROBE_ON_START") {
            labels.insert("lazymc.server.probe_on_start".to_string(), value);
        }
        if let Ok(value) = var("SERVER_SEND_PROXY_V2") {
            labels.insert("lazymc.server.send_proxy_v2".to_string(), value);
        }
        if let Ok(value) = var("SERVER_WAKE_WHITELIST") {
            labels.insert("lazymc.server.wake_whitelist".to_string(), value);
        }
        if let Ok(value) = var("TIME_MINIMUM_ONLINE_TIME") {
            labels.insert("lazymc.time.minimum_online_time".to_string(), value);
        }
        if let Ok(value) = var("TIME_SLEEP_AFTER") {
            labels.insert("lazymc.time.sleep_after".to_string(), value);
        }

        return Config::from_container_labels(labels);
    }
}
