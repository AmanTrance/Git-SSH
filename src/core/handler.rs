use super::structs::HandlerSSH;
use russh::server::Handler;
use russh::{
    server::{Auth, Msg, Session},
    Channel, ChannelId, CryptoVec, Disconnect,
};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

impl HandlerSSH {
    pub(crate) fn new() -> Self {
        HandlerSSH {
            mode: None,
            buffer: None,
            fd_in: None,
            fd_out: None,
            child: None,
        }
    }
}

impl Handler for HandlerSSH {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        let mut auth_methods: russh::MethodSet = russh::MethodSet::empty();
        auth_methods.push(russh::MethodKind::PublicKey);
        Ok(Auth::Reject {
            proceed_with_methods: Some(auth_methods),
            partial_success: false,
        })
    }

    async fn auth_publickey_offered(
        &mut self,
        _user: &str,
        _public_key: &russh::keys::PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _public_key: &russh::keys::PublicKey,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        session.channel_success(channel.id())?;
        Ok(true)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        match self.mode {
            None => Err(russh::Error::RequestDenied),
            Some(ref a) => match a {
                super::structs::Mode::ReceivePack => {
                    self.fd_in.as_mut().unwrap().write(data).await?;
                    self.fd_in.as_mut().unwrap().flush().await?;

                    let () = 'outer: loop {
                        tokio::select! {
                            result = self.fd_out
                                .as_mut()
                                .unwrap()
                                .read_buf(self.buffer.as_mut().unwrap()) => {
                                    match result {
                                        Ok(bytes) => {
                                            if bytes == 0 {
                                                session.data(
                                                    channel,
                                                    CryptoVec::from(self.buffer.as_ref().unwrap() as &[u8]),
                                                )?;
                                                break 'outer ()
                                            }
                                        }

                                        Err(_) => break 'outer ()
                                    }
                                },

                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => break 'outer ()
                        };
                    };

                    Ok(())
                }

                super::structs::Mode::UploadPack(path) => {
                    let mut child: tokio::process::Child =
                        tokio::process::Command::new("git-upload-pack")
                            .args(["--stateless-rpc", path])
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()?;
                    let mut fd_out: tokio::process::ChildStdout = child.stdout.take().unwrap();
                    let mut fd_in: tokio::process::ChildStdin = child.stdin.take().unwrap();
                    let mut buffer: Vec<u8> = vec![];

                    fd_in.write(data).await?;
                    fd_out.read_to_end(&mut buffer).await?;

                    session.data(channel, CryptoVec::from_slice(buffer.as_slice()))
                }
            },
        }
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let cmd: String = String::from_utf8_lossy(&data).into();
        if cmd.len() < 18 {
            session.disconnect(
                Disconnect::HostNotAllowedToConnect,
                "wrong operation",
                "en-US",
            )
        } else {
            let matcher: regex::Regex =
                regex::Regex::new("^(git-receive-pack)|^(git-upload-pack)").unwrap();
            if matcher.is_match(&cmd) {
                if cmd[..=15].eq("git-receive-pack") {
                    self.mode = Some(crate::core::structs::Mode::ReceivePack);
                    let mut child: tokio::process::Child =
                        tokio::process::Command::new("git-receive-pack")
                            .args([&cmd[18..(cmd.len() - 1)]])
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()?;
                    self.buffer = Some(Vec::with_capacity(1024));

                    self.fd_in = Some(child.stdin.take().unwrap());
                    self.fd_out = Some(child.stdout.take().unwrap());
                    self.child = Some(child);

                    self.fd_in.as_mut().unwrap().flush().await?;

                    self.fd_out
                        .as_mut()
                        .unwrap()
                        .read_buf(self.buffer.as_mut().unwrap())
                        .await?;
                    session.data(
                        channel,
                        CryptoVec::from(self.buffer.as_ref().unwrap() as &[u8]),
                    )?;
                    self.buffer.as_mut().unwrap().clear();
                    Ok(())
                } else {
                    self.mode = Some(super::structs::Mode::UploadPack(format!(
                        "{}",
                        &cmd[17..(cmd.len() - 1)]
                    )));
                    let output: std::process::Output =
                        tokio::process::Command::new("git-upload-pack")
                            .args(["--timeout=2", &cmd[17..(cmd.len() - 1)]])
                            .output()
                            .await?;
                    session.data(channel, CryptoVec::from_slice(output.stdout.as_slice()))
                }
            } else {
                session.disconnect(Disconnect::ServiceNotAvailable, "wrong operation", "en-US")
            }
        }
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if !self.fd_in.is_none() {
            drop(self.fd_in.take().unwrap());
        }

        if !self.fd_out.is_none() {
            drop(self.fd_out.take().unwrap());
        }

        if !self.child.is_none() {
            let mut child_process: tokio::process::Child = self.child.take().unwrap();
            let () = tokio::select! {
                _ = child_process.wait() => (),

                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                    let _ = child_process.kill();
                }
            };
        }

        session.exit_status_request(channel, 0)?;
        session.eof(channel)?;

        session.close(channel)
    }
}
