use std::{
    fmt,
    thread,

    net::{
        SocketAddr,
        Shutdown
    },

    sync::mpsc::TryRecvError,

    io::{
        self,

        Read,
        Write
    },

    collections::VecDeque,
    time::Duration,
    convert::TryFrom
};

use serde;

use mio::{
    net::{
        TcpListener,
        TcpStream
    },

    Events,
    Poll,
    PollOpt,
    Ready,
    Token
};

use mio_extras::channel::{
    channel,

    Receiver,
    Sender
};

use bincode;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Disconnect,
    Network,

    InvalidMsg,
    Internal
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Network
    }
}

impl From<TryRecvError> for Error {
    fn from(err: TryRecvError) -> Self {
        Error::Internal
    }
}

impl From<bincode::ErrorKind> for Error {
    fn from(err: bincode::ErrorKind) -> Self {
        Error::InvalidMsg
    }
}

impl<T> From<mio_extras::channel::SendError<T>> for Error {
    fn from(err: mio_extras::channel::SendError<T>) -> Self {
        Error::Internal
    }
}

pub trait PostSend = 'static + serde::Serialize + Send + fmt::Debug;
pub trait PostRecv = 'static + serde::de::DeserializeOwned + Send + fmt::Debug;

const TCP_TOK:     Token = Token(0);
const CTRL_TOK:    Token = Token(1);
const POSTBOX_TOK: Token = Token(2);
const SEND_TOK:    Token = Token(3);
const RECV_TOK:    Token = Token(4);

const MAX_MSG_BYTES: usize = 1 << 20;

enum CtrlMsg {
    Shutdown
}

pub struct PostOffice<S: PostSend, R: PostRecv> {
    worker: Option<thread::JoinHandle<Result<(), Error>>>,
    ctrl_tx: Sender<CtrlMsg>,
    postbox_rx: Receiver<Result<PostBox<S, R>, Error>>,
    poll: Poll,
    err: Option<Error>
}

impl<S: PostSend, R: PostRecv> PostOffice<S, R> {
    pub fn bind<A: Into<SocketAddr>>(addr: A) -> Result<Self, Error> {
        let tcp_listener = TcpListener::bind(&addr.into())?;

        let (ctrl_tx, ctrl_rx) = channel();
        let (postbox_tx, postbox_rx) = channel();

        let worker_poll = Poll::new()?;

        worker_poll.register(&tcp_listener, TCP_TOK, Ready::readable(), PollOpt::edge())?;
        worker_poll.register(&ctrl_rx, CTRL_TOK, Ready::readable(), PollOpt::edge())?;

        let office_poll = Poll::new()?;
        office_poll.register(&postbox_rx, POSTBOX_TOK, Ready::readable(), PollOpt::edge())?;

        let worker = thread::spawn(move || office_worker(
            worker_poll,
            tcp_listener,

            ctrl_rx,
            postbox_tx
        ));

        Ok(Self {
            worker: Some(worker),

            ctrl_tx,
            postbox_rx,

            poll: office_poll,
            err: None
        })
    }

    pub fn error(&self) -> Option<Error> {
        self.err.clone()
    }

    pub fn new_connections(&mut self) -> impl ExactSizeIterator<Item=PostBox<S, R>> {
        let mut conns = VecDeque::new();

        if let Some(_) = self.err {
            return conns.into_iter();
        }

        let mut events = Events::with_capacity(64);

        if let Err(err) = self.poll.poll(
            &mut events,

            Some(Duration::new(0, 0))
        ) {
            self.err = Some(err.into());

            return conns.into_iter();
        }

        for event in events {
            match event.token() {
                // manter lendo novos postboxes do canal
                POSTBOX_TOKEN => loop {
                    match self.postbox_rx.try_recv() {
                        Ok(Ok(conn)) => conns.push_back(conn),

                        Err(TryRecvError::Empty) => break,

                        Err(err) => {
                            self.err = Some(err.into());

                            return conns.into_iter();
                        },

                        Ok(Err(err)) => {
                            self.err = Some(err.into());

                            return conns.into_iter();
                        }
                    }
                },

                tok => panic!("token de evento inesperado '{:?}'", tok)
            }
        }

        conns.into_iter()
    }
}

impl<S: PostSend, R: PostRecv> Drop for PostOffice<S, R> {
    fn drop(&mut self) {
        let _ = self.ctrl_tx.send(CtrlMsg::Shutdown);
        let _ = self.worker.take().map(|w| w.join());
    }
}

fn office_worker<S: PostSend, R: PostRecv>(
    poll: Poll,
    tcp_listener: TcpListener,
    ctrl_rx: Receiver<CtrlMsg>,
    postbox_tx: Sender<Result<PostBox<S, R>, Error>>
) -> Result<(), Error> {
    let mut events = Events::with_capacity(64);

    loop {
        if let Err(err) = poll.poll(&mut events, None) {
            postbox_tx.send(Err(err.into()))?;

            return Ok(());
        }

        for event in &events {
            match event.token() {
                CTRL_TOK => loop {
                    match ctrl_rx.try_recv() {
                        Ok(CtrlMsg::Shutdown) => return Ok(()),

                        Err(TryRecvError::Empty) => {},

                        Err(err) => {
                            postbox_tx.send(Err(err.into()))?;

                            return Ok(());
                        }
                    }
                },

                TCP_TOK => postbox_tx.send(
                    match tcp_listener.accept() {
                        Ok((stream, _)) => PostBox::from_tcpstream(stream),

                        Err(err) => Err(err.into())
                    }
                )?,

                tok => panic!("token de evento inesperado '{:?}'", tok)
            }
        }
    }
}

pub struct PostBox<S: PostSend, R: PostRecv> {
    worker: Option<thread::JoinHandle<Result<(), Error>>>,

    ctrl_tx: Sender<CtrlMsg>,
    send_tx: Sender<S>,
    recv_rx: Receiver<Result<R, Error>>,

    poll: Poll,
    err: Option<Error>
}

impl<S: PostSend, R: PostRecv> PostBox<S, R> {
    pub fn to_server<A: Into<SocketAddr>>(addr: A) -> Result<Self, Error> {
        Self::from_tcpstream(TcpStream::connect(&addr.into())?)
    }

    fn from_tcpstream(tcp_stream: TcpStream) -> Result<Self, Error> {
        let (ctrl_tx, ctrl_rx) = channel();
        let (send_tx, send_rx) = channel();
        let (recv_tx, recv_rx) = channel();

        let worker_poll = Poll::new()?;

        worker_poll.register(&tcp_stream, TCP_TOK, Ready::readable(), PollOpt::edge())?;
        worker_poll.register(&ctrl_rx, CTRL_TOK, Ready::readable(), PollOpt::edge())?;
        worker_poll.register(&send_rx, SEND_TOK, Ready::readable(), PollOpt::edge())?;

        let postbox_poll = Poll::new()?;
        postbox_poll.register(&recv_rx, RECV_TOK, Ready::readable(), PollOpt::edge())?;

        let worker = thread::spawn(move || postbox_worker(
            worker_poll,
            tcp_stream,

            ctrl_rx,
            send_rx,
            recv_tx
        ));

        Ok(Self {
            worker: Some(worker),

            ctrl_tx,
            send_tx,
            recv_rx,

            poll: postbox_poll,
            err: None
        })
    }

    pub fn error(&self) -> Option<Error> {
        self.err.clone()
    }

    pub fn send(&mut self, data: S) {
        let _ = self.send_tx.send(data);
    }

    pub fn new_messages(&mut self) -> impl ExactSizeIterator<Item = R> {
        let mut msgs = VecDeque::new();

        if let Some(_) = self.err {
            return msgs.into_iter();
        }

        let mut events = Events::with_capacity(64);

        if let Err(err) = self.poll.poll(
            &mut events,

            Some(Duration::new(0, 0))
        ) {
            self.err = Some(err.into());

            return msgs.into_iter();
        }

        for event in events {
            match event.token() {
                // mantém lendo novas mensagens do canal
                RECV_TOKEN => loop {
                    match self.recv_rx.try_recv() {
                        Ok(Ok(msg)) => msgs.push_back(msg),

                        Err(TryRecvError::Empty) => break,

                        Err(err) => {
                            self.err = Some(err.into());

                            return msgs.into_iter();
                        },

                        Ok(Err(err)) => {
                            self.err = Some(err.into());

                            return msgs.into_iter();
                        }
                    }
                },

                tok => panic!("token de evento inesperado '{:?}'", tok)
            }
        }

        msgs.into_iter()
    }
}

impl<S: PostSend, R: PostRecv> Drop for PostBox<S, R> {
    fn drop(&mut self) {
        let _ = self.ctrl_tx.send(CtrlMsg::Shutdown);
        let _ = self.worker.take().map(|w| w.join());
    }
}

fn postbox_worker<S: PostSend, R: PostRecv>(
    poll: Poll,
    
    mut tcp_stream: TcpStream,

    ctrl_rx: Receiver<CtrlMsg>,
    send_rx: Receiver<S>,
    recv_tx: Sender<Result<R, Error>>
) -> Result<(), Error> {
    enum RecvState {
        ReadHead(Vec<u8>),
        ReadBody(usize, Vec<u8>)
    }

    let mut recv_state = RecvState::ReadHead(Vec::with_capacity(8));
    let mut events = Events::with_capacity(64);

    'work: loop {
        if let Err(err) = poll.poll(&mut events, None) {
            recv_tx.send(Err(err.into()))?;

            break 'work;
        }

        for event in &events {
            match event.token() {
                CTRL_TOK => loop {
                    match ctrl_rx.try_recv() {
                        Ok(CtrlMsg::Shutdown) => {
                            break 'work;
                        },

                        Err(TryRecvError::Empty) => break,

                        Err(err) => {
                            recv_tx.send(Err(err.into()))?;

                            break 'work;
                        }
                    }
                },

                SEND_TOK => loop {
                    match send_rx.try_recv() {
                        Ok(outgoing_msg) => {
                            let mut msg_bytes = match bincode::serialize(&outgoing_msg) {
                                Ok(bytes) => bytes,

                                Err(err) => {
                                    recv_tx.send(Err((*err).into()));

                                    break 'work;
                                }
                            };

                            let mut packet = msg_bytes
                                .len()
                                .to_le_bytes()
                                .as_ref()
                                .to_vec();

                            packet.append(&mut msg_bytes);

                            match tcp_stream.write_all(&packet) {
                                Ok(()) => {},

                                Err(err) => {
                                    recv_tx.send(Err(err.into()));

                                    break 'work;
                                }
                            }
                        },

                        Err(TryRecvError::Empty) => break,
                        
                        Err(err) => Err(err)?
                    }
                },

                TCP_TOK => loop {
                    match tcp_stream.take_error() {
                        Ok(None) => {},

                        Ok(Some(err)) => {
                            recv_tx.send(Err(err.into()));

                            break 'work;
                        },
                        
                        Err(err) => {
                            recv_tx.send(Err(err.into()));

                            break 'work;
                        }
                    }

                    match &mut recv_state {
                        RecvState::ReadHead(head) => if head.len() == 8 {
                            let len = usize::from_le_bytes(<[u8; 8]>::try_from(head.as_slice()).unwrap());

                            if len > MAX_MSG_BYTES {
                                recv_tx.send(Err(Error::InvalidMsg));

                                break 'work;
                            } else if len == 0 {
                                recv_state = RecvState::ReadHead(Vec::with_capacity(8));

                                break;
                            } else {
                                recv_state = RecvState::ReadBody(
                                    len,

                                    Vec::new()
                                );
                            }
                        } else {
                            let mut b = [0; 1];

                            match tcp_stream.read(&mut b) {
                                Ok(_) => head.push(b[0]),

                                Err(_) => break
                            }
                        },

                        RecvState::ReadBody(len, body) => if body.len() == *len {
                            match bincode::deserialize(&body) {
                                Ok(msg) => {
                                    recv_tx.send(Ok(msg))?;
                                    
                                    recv_state = RecvState::ReadHead(Vec::with_capacity(8));
                                },
                                
                                Err(err) => {
                                    recv_tx.send(Err((*err).into()))?;

                                    break 'work;
                                }
                            }
                        } else {
                            let left = *len - body.len();
                            let mut buf = vec![0; left];

                            match tcp_stream.read(&mut buf) {
                                Ok(_) => body.append(&mut buf),

                                Err(err) => {
                                    recv_tx.send(Err(err.into()))?;

                                    break 'work;
                                }
                            }
                        }
                    }
                },

                tok => panic!("token de evento inesperado '{:?}'", tok)
            }
        }
    }

    tcp_stream.shutdown(Shutdown::Both);

    Ok(())
}

// testes

#[test]
fn connect() {
    let srv_addr = ([127, 0, 0, 1], 12345);
    
    let mut postoffice = PostOffice::<u32, f32>::bind(srv_addr).unwrap();

    // deve-se iniciar com 0 conexões pendentes
    thread::sleep(Duration::from_millis(250));

    assert_eq!(postoffice.new_connections().len(), 0);
    assert_eq!(postoffice.error(), None);

    let postbox = PostBox::<f32, u32>::to_server(srv_addr).unwrap();

    // agora com um postbox criado, deve-se ter 1 novo
    thread::sleep(Duration::from_millis(250));

    let incoming = postoffice.new_connections();

    assert_eq!(incoming.len(), 1);
    assert_eq!(postoffice.error(), None);
}

#[test]
fn connection_count() {
    let srv_addr = ([127, 0, 0, 1], 12346);

    let mut postoffice = PostOffice::<u32, f32>::bind(srv_addr).unwrap();
    let mut postboxes = Vec::new();

    // deve-se iniciar com 0 conexões pendentes
    thread::sleep(Duration::from_millis(250));

    assert_eq!(postoffice.new_connections().len(), 0);
    assert_eq!(postoffice.error(), None);

    for _ in 0..5 {
        postboxes.push(PostBox::<f32, u32>::to_server(srv_addr).unwrap());
    }

    // 10 postboxes criados, deve-se ter 10
    thread::sleep(Duration::from_millis(3500));

    let incoming = postoffice.new_connections();

    assert_eq!(incoming.len(), 5);
    assert_eq!(postoffice.error(), None);
}

#[test]
fn disconnect() {
    let srv_addr = ([127, 0, 0, 1], 12347);
    
    let mut postoffice = PostOffice::<u32, f32>::bind(srv_addr).unwrap();
    
    let mut server_postbox = {
        let mut client_postbox = PostBox::<f32, u32>::to_server(srv_addr).unwrap();

        thread::sleep(Duration::from_millis(250));

        let mut incoming = postoffice.new_connections();

        assert_eq!(incoming.len(), 1);
        assert_eq!(postoffice.error(), None);

        incoming.next().unwrap();
    };

    // o postbox do client foi desconectado
    thread::sleep(Duration::from_millis(2050));

    let incoming_msgs = server_postbox.new_messages();
    
    assert_eq!(incoming_msgs.len(), 0);

    // todo
    // assert_eq!(server_postbox.error(), Some(Error::Disconnect));
}

#[test]
fn client_to_server() {
    let srv_addr = ([127, 0, 0, 1], 12348);

    let mut po = PostOffice::<u32, f32>::bind(srv_addr).unwrap();
    let mut client_pb = PostBox::<f32, u32>::to_server(srv_addr).unwrap();

    thread::sleep(Duration::from_millis(250));

    let mut server_pb = po.new_connections().next().unwrap();

    client_pb.send(1337.0);
    client_pb.send(9821.0);
    client_pb.send(-3.2);
    client_pb.send(17.0);

    thread::sleep(Duration::from_millis(250));

    let mut incoming_msgs = server_pb.new_messages();

    assert_eq!(incoming_msgs.len(), 4);
    assert_eq!(incoming_msgs.next().unwrap(), 1337.0);
    assert_eq!(incoming_msgs.next().unwrap(), 9821.0);
    assert_eq!(incoming_msgs.next().unwrap(), -3.2);
    assert_eq!(incoming_msgs.next().unwrap(), 17.0);
}

#[test]
fn server_to_client() {
    let srv_addr = ([127, 0, 0, 1], 12349);

    let mut po = PostOffice::<u32, f32>::bind(srv_addr).unwrap();
    let mut client_pb = PostBox::<f32, u32>::to_server(srv_addr).unwrap();

    thread::sleep(Duration::from_millis(250));

    let mut server_pb = po.new_connections().next().unwrap();

    server_pb.send(1337);
    server_pb.send(9821);
    server_pb.send(39999999);
    server_pb.send(17);

    thread::sleep(Duration::from_millis(250));

    let mut incoming_msgs = client_pb.new_messages();
    
    assert_eq!(incoming_msgs.len(), 4);
    assert_eq!(incoming_msgs.next().unwrap(), 1337);
    assert_eq!(incoming_msgs.next().unwrap(), 9821);
    assert_eq!(incoming_msgs.next().unwrap(), 39999999);
    assert_eq!(incoming_msgs.next().unwrap(), 17);
}
