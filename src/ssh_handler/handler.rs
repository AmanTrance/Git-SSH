use std::process::Stdio;

use crate::russh::server::Handler;
use async_trait::async_trait;
use russh::{server::{Auth, Msg, Session}, Channel, ChannelId, CryptoVec, MethodSet};
use russh_keys::ssh_key;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub struct HandlerSSH {}

impl HandlerSSH {
    pub fn new() -> Self {
        HandlerSSH {}
    }
}

#[async_trait]
impl Handler for HandlerSSH {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Reject { proceed_with_methods: Some (MethodSet::HOSTBASED) })
    }

    async fn auth_publickey_offered(&mut self, user: &str, public_key: &ssh_key::PublicKey) -> Result<Auth, Self::Error> {
        if user != "asvatthi" {
            Ok(Auth::Reject { proceed_with_methods: None })
        } else {
            let _ = String::from_utf8_lossy(public_key.to_bytes().unwrap().as_slice());
            Ok(Auth::Accept)    
        }
    }

    async fn channel_open_session(&mut self, channel: Channel<Msg>, session: &mut Session) -> Result<bool, Self::Error> {
        session.channel_success(channel.id())?;
        Ok(true)
    }

    async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error> {
        let mut child: tokio::process::Child = tokio::process::Command::new("git-upload-pack").args(["--stateless-rpc", "/home/amanfreecs/hello.git"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let mut fd_out: tokio::process::ChildStdout = child.stdout.take().unwrap();
        let mut fd_in: tokio::process::ChildStdin = child.stdin.take().unwrap();
        fd_in.write(data).await.unwrap();
        let mut buffer: Vec<u8> = vec![];
        fd_out.read_to_end(&mut buffer).await.unwrap();
        session.data(channel, CryptoVec::from_slice(buffer.as_slice())).unwrap();
        Ok(())
    }

    async fn exec_request(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error> {
        let cmd: String = String::from_utf8_lossy(data).into();
        if cmd.len() < 16 {
            Err(russh::Error::RequestDenied)
        } else {
            if cmd[0..16].eq("git-receive-pack")  {
                let output: std::process::Output = tokio::process::Command::new("git-upload-pack").args(["--timeout=2", "/home/amanfreecs/hello.git"]).output().await.unwrap();
                session.data(channel, CryptoVec::from(output.stdout)).unwrap();
                Ok(())
            } else if cmd[0..15].eq("git-upload-pack") {
                let output: std::process::Output = tokio::process::Command::new("git-upload-pack").args(["--timeout=2", "/home/amanfreecs/hello.git"]).output().await.unwrap();
                session.data(channel, CryptoVec::from(output.stdout)).unwrap();
                Ok(())
            } else {
                Err(russh::Error::RequestDenied)
            }
        }
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.exit_status_request(channel, 0)?;    
        session.close(channel)?;    
        Ok(())
    }
}
