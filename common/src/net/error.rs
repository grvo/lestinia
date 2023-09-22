#[derive(Debug)]
pub enum PostError {
    Io(std::io::Error),
    Serde(bincode::Error),

    ChannelRecv(std::sync::mpsc::TryRecvError),
    ChannelSend, // vazio porque não foi possível figurar como gerenciar tipo genérico em mpsc::trysenderror

    MsgSizeLimitExceeded
}

impl From<std::io::Error> for PostError {
    fn from(err: std::io::Error) -> Self {
        PostError::Io(err)
    }
}

impl From<bincode::Error> for PostError {
    fn from(err: bincode::Error) -> Self {
        PostError::Serde(err)
    }
}

impl From<std::sync::mpsc::TryRecvError> for PostError {
    fn from(err: std::sync::mpsc::TryRecvError) -> Self {
        PostError::ChannelRecv(err)
    }
}
