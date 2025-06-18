mod core;

use russh::server::Server;
use russh::*;
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config: server::Config = russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keepalive_interval: Some(std::time::Duration::from_secs(30)),
        keepalive_max: 3,
        keys: vec![russh::keys::PrivateKey::read_openssh_file(Path::new(
            "/home/amanfreecs/.ssh/id_ed25519",
        ))
        .unwrap()],
        preferred: Preferred {
            ..Preferred::default()
        },
        ..Default::default()
    };

    let mut server: core::structs::ServerSSH = core::structs::ServerSSH::new();

    server
        .run_on_address(Arc::new(config), ("0.0.0.0", 2222))
        .await
        .unwrap();
}
