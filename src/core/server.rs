use super::structs::HandlerSSH;
use super::structs::ServerSSH;
use russh::server::Server;
use std::io::Write;

impl ServerSSH {
    pub(crate) fn new() -> Self {
        ServerSSH {
            logger: std::io::stdout(),
        }
    }
}

impl Server for ServerSSH {
    type Handler = HandlerSSH;

    fn new_client(&mut self, address: Option<std::net::SocketAddr>) -> Self::Handler {
        if address.is_some() {
            self.logger
                .write(
                    format!(
                        "{}: [info] Client Connected from {}\n",
                        chrono::Local::now().to_string(),
                        address.unwrap().to_string()
                    )
                    .as_bytes(),
                )
                .unwrap();
            self.logger.flush().unwrap();
        }
        HandlerSSH::new()
    }

    fn handle_session_error(
        &mut self,
        error: <Self::Handler as russh::server::Handler>::Error,
    ) -> () {
        self.logger
            .write(
                format!(
                    "{}: [error] Session Error {}\n",
                    chrono::Local::now().to_string(),
                    error.to_string()
                )
                .as_bytes(),
            )
            .unwrap();
        self.logger.flush().unwrap();
        ()
    }
}
