use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::var;
use std::fs::File;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::process::{exit, Command};
use version_compare::Version;

use crate::adapter::Adapter;
use crate::health::unhealthy;

const DEFAULT_PORT: i32 = 25565;

/// lazymc dropped support minecraft servers with version less than 1.20.3
fn is_legacy(version: Option<String>) -> bool {
    if version.is_none() {
        return false;
    }

    Version::from(version.unwrap().as_ref()) < Version::from("1.20.3")
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
    start_timeout: Option<i32>,
    stop_timeout: Option<i32>,
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
struct JoinSection {
    methods: Option<Vec<String>>,
    kick: JoinKickSection,
    hold: JoinHoldSection,
    forward: JoinForwardSection,
    lobby: JoinLobbySection,
}

#[derive(Serialize, Deserialize, Clone)]
struct JoinKickSection {
    starting: Option<String>,
    stopping: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct JoinHoldSection {
    timeout: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct JoinForwardSection {
    address: Option<String>,
    send_proxy_v2: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct JoinLobbySection {
    timeout: Option<i32>,
    message: Option<String>,
    ready_sound: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MotdSection {
    sleeping: Option<String>,
    starting: Option<String>,
    stopping: Option<String>,
    from_server: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct LockoutSection {
    enabled: Option<bool>,
    message: Option<String>,
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
    join: JoinSection,
    lockout: LockoutSection,
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
    #[serde(skip)]
    resolved_ip: bool,
}

/// Configuration for the lazymc server
impl Config {
    /// Generate the start command for the lazymc server
    pub fn start_command<T: Adapter>(&self) -> Command {
        // Start the container if the IP address has not been resolved
        if !self.resolved_ip {
            T::start(self.group());
        }

        let mut command: Command = Command::new(self.start_command.clone());

        command.arg("start");
        command.arg("--config");
        command.arg(self.config_file.clone());

        command
    }

    /// Get the group name for the lazymc server
    pub fn group(&self) -> &String {
        &self.group
    }

    /// Convert the configuration to a TOML string
    fn as_toml_string(&self) -> String {
        toml::to_string(self).unwrap()
    }

    /// Create the lazymc configuration file
    fn create_file(&self) {
        debug!(target: "lazymc-docker-proxy::entrypoint::config", "current dir: {}", std::env::current_dir().unwrap().to_str().unwrap());
        let toml = self.as_toml_string();
        let file_name: &String = &format!("lazymc.{}.toml", self.group.clone());
        let path: &Path = Path::new(file_name);
        debug!(target: "lazymc-docker-proxy::entrypoint::config", "config file: {}", &path.display());
        let mut file = File::create(path).unwrap();
        file.write(toml.as_ref()).unwrap();
        debug!(target: "lazymc-docker-proxy::entrypoint::config", "`generated`: {}\n\n{}", path.display(), toml);
    }

    /// Create a new configuration from container labels
    pub fn from_container_labels(labels: &HashMap<String, String>) -> Self {
        // Check for required labels
        labels.get("lazymc.server.address").unwrap_or_else(|| {
            error!(target: "lazymc-docker-proxy::entrypoint::config", "lazymc.server.address is not set");
            unhealthy();
            exit(1);
        });

        labels.get("lazymc.group").unwrap_or_else(|| {
            error!(target: "lazymc-docker-proxy::entrypoint::config", "lazymc.group is not set");
            unhealthy();
            exit(1);
        });

        // Check if the IP address has been resolved
        let mut resolved_ip = true;

        let server_section: ServerSection = ServerSection {
            address: labels.get("lazymc.server.address")
                .and_then(|address| address.to_socket_addrs().ok())
                .and_then(|addrs| addrs.filter(|addr| addr.is_ipv4()).next())
                .and_then(|addr| Some(addr.to_string()))
                .or_else(|| {
                    warn!(target: "lazymc-docker-proxy::entrypoint::config", "Failed to resolve IP address from lazymc.server.address. Falling back to the value provided.");
                    resolved_ip = false;
                    labels.get("lazymc.server.address").cloned()
                }),
            directory: Some(
                labels
                    .get("lazymc.server.directory")
                    .cloned()
                    .unwrap_or_else(|| String::from("/server")),
            ),
            command: Some(format!(
                "lazymc-docker-proxy --command --adapter systemd --group {}",
                labels.get("lazymc.group").unwrap()
            )),
            freeze_process: Some(false),
            // If the IP address was not resolved, wake_on_start should be true
            wake_on_start: Some(!resolved_ip),
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
            start_timeout: labels
                .get("lazymc.server.start_timeout")
                .and_then(|x| x.parse().ok()),
            stop_timeout: labels
                .get("lazymc.server.stop_timeout")
                .and_then(|x| x.parse().ok()),
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

        let join_kick_section: JoinKickSection = JoinKickSection {
            starting: labels.get("lazymc.join.kick.starting").cloned(),
            stopping: labels.get("lazymc.join.kick.stopping").cloned(),
        };

        let join_hold_section: JoinHoldSection = JoinHoldSection {
            timeout: labels
                .get("lazymc.join.hold.timeout")
                .and_then(|x| x.parse().ok()),
        };

        let join_forward_section: JoinForwardSection = JoinForwardSection {
            address: labels.get("lazymc.join.forward.address").cloned(),
            send_proxy_v2: labels
                .get("lazymc.join.forward.send_proxy_v2")
                .map(|x| x == "true"),
        };

        let join_lobby_section: JoinLobbySection = JoinLobbySection {
            timeout: labels
                .get("lazymc.join.lobby.timeout")
                .and_then(|x| x.parse().ok()),
            message: labels.get("lazymc.join.lobby.message").cloned(),
            ready_sound: labels.get("lazymc.join.lobby.sound").cloned(),
        };

        let join_section: JoinSection = JoinSection {
            methods: labels
                .get("lazymc.join.methods")
                .and_then(|x| {
                    Some(x.split(",").map(|s| s.to_owned()).collect())
                        .filter(|m: &Vec<String>| !m.is_empty())
                })
                .or_else(|| None),
            kick: join_kick_section.clone(),
            hold: join_hold_section.clone(),
            forward: join_forward_section.clone(),
            lobby: join_lobby_section.clone(),
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
            from_server: labels.get("lazymc.motd.from_server").map(|x| x == "true"),
        };

        let lockout_section: LockoutSection = LockoutSection {
            enabled: labels.get("lazymc.lockout.enabled").map(|x| x == "true"),
            message: labels.get("lazymc.lockout.message").cloned(),
        };

        let advanced_section: AdvancedSection = AdvancedSection {
            rewrite_server_properties: Some(false),
        };

        let config_section: ConfigSection = ConfigSection {
            version: match is_legacy(labels.get("lazymc.public.version").cloned()) {
                true => var("LAZYMC_LEGACY_VERSION")
                    .unwrap_or_else(|err| {
                        error!(target: "lazymc-docker-proxy::entrypoint::config", "LAZYMC_LEGACY_VERSION is not set: {}", err);
                        unhealthy();
                        exit(1);
                    })
                    .into(),
                false => var("LAZYMC_VERSION")
                    .unwrap_or_else(|err| {
                        error!(target: "lazymc-docker-proxy::entrypoint::config", "LAZYMC_VERSION is not set: {}", err);
                        unhealthy();
                        exit(1);
                    })
                    .into(),
            },
        };

        let config: Config = Config {
            server: server_section,
            public: public_section,
            time: time_section,
            join: join_section,
            motd: motd_section,
            lockout: lockout_section,
            advanced: advanced_section,
            config: config_section,
            start_command: match is_legacy(labels.get("lazymc.public.version").cloned()) {
                true => String::from("lazymc-legacy"),
                false => String::from("lazymc"),
            },
            config_file: format!(
                "lazymc.{}.toml",
                labels.get("lazymc.group").unwrap().clone()
            ),
            group: labels.get("lazymc.group").unwrap().clone(),
            resolved_ip,
        };

        // Generate the lazymc config file
        config.create_file();

        config
    }

    /// Convert environment map to label equivalent
    pub fn environment_to_label(environment: HashMap<String, String>) -> HashMap<String, String> {
        environment
            .into_iter()
            .filter_map(|(key, value)| {
                match key.as_str() {
                    "LAZYMC_SERVER_ADDRESS"             => Some(("lazymc.server.address", value)),
                    "LAZYMC_GROUP"                      => Some(("lazymc.group", value)),
                    "LAZYMC_SERVER_DIRECTORY"           => Some(("lazymc.server.directory", value)),
                    "LAZYMC_SERVER_WAKE_WHITELIST"      => Some(("lazymc.server.wake_whitelist", value)),
                    "LAZYMC_SERVER_BLOCK_BANNED_IPS"    => Some(("lazymc.server.block_banned_ips", value)),
                    "LAZYMC_SERVER_DROP_BANNED_IPS"     => Some(("lazymc.server.drop_banned_ips", value)),
                    "LAZYMC_SERVER_PROBE_ON_START"      => Some(("lazymc.server.probe_on_start", value)),
                    "LAZYMC_SERVER_FORGE"               => Some(("lazymc.server.forge", value)),
                    "LAZYMC_SERVER_START_TIMEOUT"       => Some(("lazymc.server.start_timeout", value)),
                    "LAZYMC_SERVER_STOP_TIMEOUT"        => Some(("lazymc.server.stop_timeout", value)),
                    "LAZYMC_SERVER_SEND_PROXY_V2"       => Some(("lazymc.server.send_proxy_v2", value)),
                    "LAZYMC_TIME_SLEEP_AFTER"           => Some(("lazymc.time.sleep_after", value)),
                    "LAZYMC_TIME_MINIMUM_ONLINE_TIME"   => Some(("lazymc.time.minimum_online_time", value)),
                    "LAZYMC_JOIN_KICK_STARTING"         => Some(("lazymc.join.kick.starting", value)),
                    "LAZYMC_JOIN_KICK_STOPPING"         => Some(("lazymc.join.kick.stopping", value)),
                    "LAZYMC_JOIN_HOLD_TIMEOUT"          => Some(("lazymc.join.hold.timeout", value)),
                    "LAZYMC_JOIN_FORWARD_ADDRESS"       => Some(("lazymc.join.forward.address", value)),
                    "LAZYMC_JOIN_FORWARD_SEND_PROXY_V2" => Some(("lazymc.join.forward.send_proxy_v2", value)),
                    "LAZYMC_JOIN_LOBBY_TIMEOUT"         => Some(("lazymc.join.lobby.timeout", value)),
                    "LAZYMC_JOIN_LOBBY_MESSAGE"         => Some(("lazymc.join.lobby.message", value)),
                    "LAZYMC_JOIN_LOBBY_SOUND"           => Some(("lazymc.join.lobby.sound", value)),
                    "LAZYMC_JOIN_METHODS"               => Some(("lazymc.join.methods", value)),
                    "LAZYMC_PORT"                       => Some(("lazymc.port", value)),
                    "LAZYMC_PUBLIC_VERSION"             => Some(("lazymc.public.version", value)),
                    "LAZYMC_PUBLIC_PROTOCOL"            => Some(("lazymc.public.protocol", value)),
                    "LAZYMC_MOTD_SLEEPING"              => Some(("lazymc.motd.sleeping", value)),
                    "LAZYMC_MOTD_STARTING"              => Some(("lazymc.motd.starting", value)),
                    "LAZYMC_MOTD_STOPPING"              => Some(("lazymc.motd.stopping", value)),
                    "LAZYMC_MOTD_FROM_SERVER"           => Some(("lazymc.motd.from_server", value)),
                    "LAZYMC_LOCKOUT_ENABLED"            => Some(("lazymc.lockout.enabled", value)),
                    "LAZYMC_LOCKOUT_MESSAGE"            => Some(("lazymc.lockout.message", value)),
                    _ => None
                }
            })
            .map(|(key, value)| {(key.to_owned(), value.to_owned())})
            .collect()
    }
}
