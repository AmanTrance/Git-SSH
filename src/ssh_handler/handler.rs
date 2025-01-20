use std::process::Stdio;
use crate::russh::server::Handler;
use async_trait::async_trait;
use russh::{server::{Auth, Msg, Session}, Channel, ChannelId, CryptoVec, Disconnect, MethodSet};
use russh_keys::ssh_key;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use super::structs::HandlerSSH;

impl HandlerSSH {
    pub fn new() -> Self {
        HandlerSSH { mode: None, fd_in: None, child: None, remove_packet_length: 0 }
    }
}

#[async_trait]
impl Handler for HandlerSSH {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Reject { proceed_with_methods: Some (MethodSet::PUBLICKEY) })
    }

    async fn auth_publickey_offered(&mut self, _user: &str, public_key: &ssh_key::PublicKey) -> Result<Auth, Self::Error> {
        // let key_slice: Vec<u8> = public_key.to_bytes().unwrap();
        // let key: std::borrow::Cow<'_, str> = String::from_utf8_lossy(key_slice.as_slice()).to_owned();
        self.mode = Some ( super::structs::Mode::ReceivePack (String::from("/home/amanfreecs/hello.git")) );
        Ok(Auth::Accept)
    }

    async fn auth_publickey(&mut self, _user: &str, _public_key: &ssh_key::PublicKey) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(&mut self, channel: Channel<Msg>, session: &mut Session) -> Result<bool, Self::Error> {
        session.channel_success(channel.id())?;
        Ok(true)
    }

    async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error> {
        match self.mode {
            None => Err(russh::Error::RequestDenied),
            Some (ref a) => {
                match a {
                    
                    super::structs::Mode::ReceivePack (path) => {
                        
                        if !self.fd_in.is_none() && !self.child.is_none() {
                            self.fd_in.as_mut().unwrap().write(data).await?;

                        } else {
                            let mut child: tokio::process::Child = tokio::process::Command::new("git-receive-pack").args([path]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
                            let mut fd_out: tokio::process::ChildStdout = child.stdout.take().unwrap();
                            let mut fd_in: tokio::process::ChildStdin = child.stdin.take().unwrap();
                            
                            fd_in.write(data).await?;
                            
                            self.child = Some (child);
                            self.fd_in = Some (fd_in);
                            
                            let handler: russh::server::Handle = session.handle();
                            let remove_len = self.remove_packet_length;

                            tokio::spawn(async move {
                                let mut buffer: Vec<u8> = vec![];
                                fd_out.read_to_end(&mut buffer).await.unwrap();
                                handler.data(channel, CryptoVec::from_slice(&buffer[remove_len..])).await.unwrap();
                            });
                        }

                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        Ok(()) 
                    },

                    super::structs::Mode::UploadPack (path) => {
                        let mut child: tokio::process::Child = tokio::process::Command::new("git-upload-pack").args(["--stateless-rpc", path]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
                        let mut fd_out: tokio::process::ChildStdout = child.stdout.take().unwrap();
                        let mut fd_in: tokio::process::ChildStdin = child.stdin.take().unwrap();
                        let mut buffer: Vec<u8> = vec![];

                        fd_in.write(data).await?;
                        fd_out.read_to_end(&mut buffer).await?;
                        
                        session.data(channel, CryptoVec::from_slice(buffer.as_slice()))
                    }
                }
            }
        }
    }

    async fn exec_request(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error> {
        let cmd: String = String::from_utf8_lossy(data).into();
        println!("{cmd}");
        if cmd.len() < 18 {
            Err(russh::Error::RequestDenied)
        } else {
            match self.mode {
                None => Err(russh::Error::NotAuthenticated),
                Some (ref a) => {
                    match a {
                        
                        super::structs::Mode::ReceivePack (path) => {
                            
                            if !cmd[..=15].eq("git-receive-pack") {
                                session.disconnect(Disconnect::ServiceNotAvailable, "wrong operation", "en-US")
                            } else if !cmd[18..(cmd.len()-1)].eq(path) {
                                session.disconnect(Disconnect::ServiceNotAvailable, "wrong repo path", "en-US")
                            } else {
                                let output = tokio::process::Command::new("git-receive-pack").args(["--advertise-refs", path]).output().await?;
                                self.remove_packet_length = output.stdout.len();
                                session.data(channel, CryptoVec::from_slice(output.stdout.as_slice()))
                            }

                        },

                        super::structs::Mode::UploadPack (path) => {
                            
                            if !cmd[..=14].eq("git-upload-pack") {
                                session.disconnect(Disconnect::ServiceNotAvailable, "wrong operation", "en-US")
                            } else if !cmd[17..(cmd.len()-1)].eq(path) {
                                session.disconnect(Disconnect::ServiceNotAvailable, "wrong repo path", "en-US")
                            } else {
                                let output = tokio::process::Command::new("git-upload-pack").args(["--timeout=5", path]).output().await?;
                                session.data(channel, CryptoVec::from_slice(output.stdout.as_slice()))
                            }

                        }
                    }
                }
            }
        }
    }

    async fn channel_eof(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), Self::Error> {
        session.exit_status_request(channel, 0)?;
        session.eof(channel)?;
        
        let mut child_process = self.child.take().unwrap();
        
        if !self.fd_in.is_none() {
            drop(self.fd_in.take().unwrap());
        }
        
        tokio::spawn(async move {
            child_process.wait().await.unwrap();
        });    
        
        session.close(channel)
    }
}
