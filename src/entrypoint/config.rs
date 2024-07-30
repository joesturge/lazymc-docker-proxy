use serde::{Deserialize, Serialize};
use std::env::var;
use std::process::exit;
use std::{fs, path::Path};
use toml::Value;

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
struct Config {
    advanced: AdvancedSection,
    config: ConfigSection,
    motd: MotdSection,
    public: PublicSection,
    server: ServerSection,
    time: TimeSection,
}

pub fn generate() {
    info!(target: "lazymc-docker-proxy::config", "Generating lazymc.toml...");

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

    let public_section: PublicSection = PublicSection {
        version: var("PUBLIC_VERSION").ok(),
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
        version: var("LAZYMC_VERSION")
        .unwrap_or_else(|err| {
            error!(target: "lazymc-docker-proxy::config", "LAZYMC_VERSION is not set: {}", err);
            exit(1);
        })
        .into(),
    };

    let config: Config = Config {
        server: server_section,
        public: public_section,
        time: time_section,
        motd: motd_section,
        advanced: advanced_section,
        config: config_section,
    };

    // Convert the config struct to a toml::Value
    let toml_data: Value = Value::try_from(config).unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::config", "Failed to convert to TOML data: {}", err);
        exit(1);
    });

    // Convert the toml::Value to a string
    let toml_string: String = toml::to_string(&toml_data).unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::config", "Failed to serialize TOML data: {}", err);
        exit(1);
    });

    // Path to the output TOML file
    let output_path: &Path = Path::new("lazymc.toml");

    // Write the TOML string to the file
    fs::write(output_path, &toml_string).unwrap_or_else(|err| {
        error!(target: "lazymc-docker-proxy::config", "Failed to write TOML file: {}", err);
        exit(1);
    });

    info!(target: "lazymc-docker-proxy::config", "Successfully generated lazymc.toml");
    debug!(target: "lazymc-docker-proxy::config", "lazymc.toml: {}", &toml_string);
}
