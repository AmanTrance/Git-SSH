use crate::russh::server::Server;
use super::handler::HandlerSSH;

pub struct ServerSSH {}

impl Server for ServerSSH {
    type Handler = super::handler::HandlerSSH;

    fn new_client(&mut self, _address: Option<std::net::SocketAddr>) -> Self::Handler {
        HandlerSSH::new()
    }

    fn handle_session_error(&mut self, error: <Self::Handler as russh::server::Handler>::Error) -> () {
        println!("{}", error.to_string());
    }    
}
