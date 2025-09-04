mod core;

use russh::server::{self, Server};
use russh::Preferred;
use std::path::Path;
use std::sync::Arc;
use std::env;
use serde::Deserialize;

use std::time::Duration;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file_path = if args.len() == 2 {
        Some(args.get(1).unwrap())
    }else {
        None
    };
    let config = server_config(config_file_path);
    let mut server: core::structs::ServerSSH = core::structs::ServerSSH::new();

    server
        .run_on_address(Arc::new(config), ("0.0.0.0", 2222))
        .await
        .unwrap();
}

fn server_config(path: Option<&String>) -> russh::server::Config {
    if path.is_none() {
        let config: server::Config = russh::server::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
            auth_rejection_time: std::time::Duration::from_secs(3),
            auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
            keepalive_interval: Some(std::time::Duration::from_secs(30)),
            keepalive_max: 3,
            keys: vec![russh::keys::PrivateKey::read_openssh_file(Path::new(
                "$HOME/.ssh/id_ed25519",
            ))
            .unwrap()],
            preferred: Preferred {
                ..Preferred::default()
            },
            ..Default::default()
        };
        return config;
    }else {
        let path = path.unwrap();
        let config_file = std::fs::read_to_string(path).expect("Config file to exist");
        let config: Config = serde_yaml::from_str(&config_file).unwrap();
        let mut keys = vec![];
        for p in config.keys_path {
            let key = russh::keys::PrivateKey::read_openssh_file(Path::new(&p)).unwrap();
            keys.push(key);
        }
        let server_config: server::Config = russh::server::Config {
            inactivity_timeout: Some(Duration::from_secs(config.inactivity_timeout)),
            auth_rejection_time: Duration::from_secs(config.auth_rejection_time),
            auth_rejection_time_initial: Some(Duration::from_secs(config.auth_rejection_time_initial)),
            keepalive_interval: Some(Duration::from_secs(config.keepalive_interval)),
            keepalive_max: config.keepalive_max,
            keys,
            preferred: russh::Preferred {
                ..russh::Preferred::default()
            },
            ..Default::default()
        };
        return server_config;
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    inactivity_timeout: u64,
    auth_rejection_time: u64,
    auth_rejection_time_initial: u64,
    keepalive_interval: u64,
    keepalive_max: usize,
    keys_path: Vec<String>,
}
