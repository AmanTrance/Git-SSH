#[derive(Debug)]
pub(crate) struct HandlerSSH {
    pub(crate) mode: Option<Mode>,
    pub(crate) fd_in: Option<tokio::process::ChildStdin>,
    pub(crate) child: Option<tokio::process::Child>,
    pub(crate) remove_packet_length: usize
}

#[derive(Debug)]
pub(crate) struct ServerSSH {}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Mode {
    UploadPack (String),
    ReceivePack (String)
}