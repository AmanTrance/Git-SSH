use std::sync::Arc;
use russh::*;
use russh::server::Server;
use russh_keys::ssh_key::rand_core::OsRng;

extern crate tokio;
extern crate russh;
extern crate async_trait;
extern crate uuid;
extern crate russh_keys;
mod ssh_handler;

#[tokio::main]
async fn main() {
    let config: server::Config = russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![
            russh_keys::PrivateKey::random(&mut OsRng, russh_keys::Algorithm::Ed25519).unwrap(),
        ],
        preferred: Preferred {
            ..Preferred::default()
        },
        ..Default::default()
    };

    let mut server: ssh_handler::server::ServerSSH = ssh_handler::server::ServerSSH {};

    server.run_on_address(Arc::new(config), ("0.0.0.0", 2222)).await.unwrap();
}
