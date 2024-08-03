use serde::{Deserialize, Serialize};
use std::env::var;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};
use version_compare::Version;

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
    
    fn as_toml_string(&self) -> String {
        toml::to_string(self).unwrap()
    }

    fn create_file(&self) {
        let toml = self.as_toml_string();
        let file_name: &String = &format!("lazymc.{}.toml", self.group.clone());
        let path: &Path = Path::new(file_name);
        let mut file = File::create(path).unwrap();
        file.write(toml.as_ref()).unwrap();
        debug!(target: "lazymc-docker-proxy::config", "`generated`: {}\n\n{}", path.display(), toml);
    }

    pub fn from_env() -> Self {
        let server_section: ServerSection = ServerSection {
            address: var("SERVER_ADDRESS")
                .unwrap_or_else(|err| {
                    error!(target: "lazymc-docker-proxy::config", "SERVER_ADDRESS is not set: {}", err);
                    exit(1);
                })
                .into(),
            directory: Some("/server".to_string()),
            command: Some("lazymc-docker-proxy --command".to_string()),
            // It tries to unfreeze the process when the server PID is not created yet
            freeze_process: Some(false),
            // It does not work if 'wake_on_start' is not set to true as the server starts when docker compose starts
            // We need the start command to run at boot so that lazymc has a PID to keep track of
            wake_on_start: Some(true),
            // Probably a good idea to enforce this too, as we suggest that users should use 'restart: no' in the mc server docker compose file
            wake_on_crash: Some(true),
            wake_whitelist: var("SERVER_WAKE_WHITELIST")
                .ok()
                .map(|x: String| x == "true"),
            block_banned_ips: var("SERVER_BLOCK_BANNED_IPS")
                .ok()
                .map(|x: String| x == "true"),
            drop_banned_ips: var("SERVER_DROP_BANNED_IPS")
                .ok()
                .map(|x: String| x == "true"),
            probe_on_start: var("SERVER_PROBE_ON_START")
                .ok()
                .map(|x: String| x == "true"),
            forge: var("SERVER_FORGE").ok().map(|x: String| x == "true"),
            send_proxy_v2: var("SERVER_SEND_PROXY_V2")
                .ok()
                .map(|x: String| x == "true"),
        };

        let time_section: TimeSection = TimeSection {
            sleep_after: var("TIME_SLEEP_AFTER")
                .ok()
                .and_then(|x: String| x.parse().ok()),
            minimum_online_time: var("TIME_MINIMUM_ONLINE_TIME")
                .ok()
                .and_then(|x: String| x.parse().ok()),
        };

        let public_version = var("PUBLIC_VERSION").ok();

        let public_section: PublicSection = PublicSection {
            version: public_version.clone(),
            protocol: var("PUBLIC_PROTOCOL")
                .ok()
                .and_then(|x: String| x.parse().ok()),
        };

        let motd_section: MotdSection = MotdSection {
            sleeping: var("MOTD_SLEEPING").ok(),
            starting: var("MOTD_STARTING").ok(),
            stopping: var("MOTD_STOPPING").ok(),
        };

        let advanced_section: AdvancedSection = AdvancedSection {
            rewrite_server_properties: Some(false),
        };

        let config_section: ConfigSection = ConfigSection {
            version: match is_legacy(public_version.clone()) {
                true => var("LAZYMC_LEGACY_VERSION")
                    .unwrap_or_else(|err| {
                        error!(target: "lazymc-docker-proxy::config", "LAZYMC_LEGACY_VERSION is not set: {}", err);
                        exit(1);
                    })
                    .into(),
                false => var("LAZYMC_VERSION")
                    .unwrap_or_else(|err| {
                        error!(target: "lazymc-docker-proxy::config", "LAZYMC_VERSION is not set: {}", err);
                        exit(1);
                    })
                    .into(),
            },
        };

        let group = var("LAZYMC_GROUP").unwrap_or_else(|err| {
            error!(target: "lazymc-docker-proxy::config", "LAZYMC_GROUP is not set: {}", err);
            exit(1);
        });

        let config: Config = Config {
            server: server_section,
            public: public_section,
            time: time_section,
            motd: motd_section,
            advanced: advanced_section,
            config: config_section,
            start_command: match is_legacy(public_version.clone()) {
                true => format!("lazymc-legacy"),
                false => format!("lazymc"),
            },
            config_file: format!("lazymc.{}.toml", group.clone()),
            group: group.clone(),
        };

        // Generate the lazymc config file
        config.create_file();

        return config;
    }
}
