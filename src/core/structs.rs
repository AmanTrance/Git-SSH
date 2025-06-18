#[derive(Debug)]
pub(crate) struct HandlerSSH {
    pub(crate) mode: Option<Mode>,
    pub(crate) buffer: Option<Vec<u8>>,
    pub(crate) fd_in: Option<tokio::process::ChildStdin>,
    pub(crate) fd_out: Option<tokio::process::ChildStdout>,
    pub(crate) child: Option<tokio::process::Child>,
}

#[derive(Debug)]
pub(crate) struct ServerSSH {
    pub(crate) logger: std::io::Stdout,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Mode {
    UploadPack(String),
    ReceivePack,
}
