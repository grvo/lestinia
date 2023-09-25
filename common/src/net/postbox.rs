// padrão
use std::collections::VecDeque;
use std::convert::TryForm;
use std::io::ErrorKind;
use std::io::Read;
use std::net::SocketAddr;
use std::thread;

// exeterno
use bincode;

use mio::{
    net::TcpStream,
    
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

// caixote
use super::data::ControlMsg;
use super::error::PostError;

use super::{
    PostRecv,
    PostSend
};

// constantes
const CTRL_TOKEN: Token = Token(0) // token para mensagens de controle de threads
const DATA_TOKEN: Token = Token(1) // token para exchange de dados de threads
const CONN_TOKEN: Token = Token(2) // token para tcpstream

const MESSAGE_SIZE_CAP: u64 = 1 << 20; // tamanho máximo aceito de um packet

/// um wrapper de alto-nível para [`tcpstream`](mio::net::tcpstream).
/// [`postbox`] cuida de packets enviados serializados no background, fornecendo uma api simples para enviar e receber objetos
pub struct PostBox<S, R>
where
    S: PostSend,
    R: PostRecv
{
    handle: Option<thread::JoinHandle<()>>,
    ctrl: Sender<ControlMsg>,
    recv: Receiver<Result<R>, PostError>,
    send: Sender<S>,
    poll: Poll
}

impl<S, R> PostBox<S, R>
where
    S: PostSend,
    R: PostRecv
{
    /// cria um novo [`postbox`] conectado para endereços específicos, podendo ser utilizado pelo cleint
    pub fn to_server(addr: &SocketAddr) -> Result<PostBox<S, R>, PostError> {
        let connection = TcpStream::connect(addr)?;

        Self::from_tcpstream(connection)
    }

    /// cria um novo [`postbox`] por meio de uma conexão existente, podendo ser utilizado por [`postoffice`] no servidor
    pub fn from_tcpstream(connection: TcpStream) -> Result<PostBox<S, R>, PostError> {
        let (ctrl_tx, ctrl_rx) = channel::<ControlMsg>(); // mensagens de controle
        let (send_tx, send_rx) = channel::<S>(); // thread principal
        let (recv_tx, recv_rx) = channel::<Result<R, PostError>>(); // thread principal - thread trabalhadora

        let thread_poll = Poll::new().unwrap();
        let postbox_poll = Poll::new().unwrap();

        thread_poll
            .register(&connection, CONN_TOKEN, Ready::readable(), PollOpt::edge())
            .unwrap();

        thread_poll
            .register(&ctrl_rx, CTRL_TOKEN, Ready::readable(), PollOpt::edge())
            .unwrap()

        thread_poll
            .register(&send_rx, DATA_TOKEN, Ready::readable(), PollOpt::edge())
            .unwrap()

        thread_poll
            .register(&recv_rx, DATA_TOKEN, Ready::readable(), PollOpt::edge())
            .unwrap()

        let handle = thread::Builder::new()
            .name("postbox_worker".into())
            .spawn(move || postbox_thread(connection, ctrl_rx, send_rx, recv_tx, thread_poll))?;

        Ok(PostBox {
            handle: Some(handle),

            ctrl: ctrl_tx,
            recv: recv_rx,
            send: send_tx,

            poll: postbox_poll
        })
    }

    /// método sender não-bloqueável
    pub fn send(&self, data: S) {
        self.send.send(data).unwrap_or(());
    }

    /// método receptor não-bloqueável retornando um iterator após recber objetos deserializados
    /// # erros
    /// se o outro lado se desconectar do postbox, tentar algo novo para enviar
    pub fn recv_iter(&self) -> Result<impl Iterator<Item = Result<R, PostError>>, PostError> {
        let mut events = Events::with_capacity(4096);

        self.poll
            .poll(&mut events, Some(core::time::Duration::new(0, 0)))?;

        let mut data: VecDeque<Result<R, PostError>> = VecDeque::new();

        for event in events {
            match event.token() {
                DATA_TOKEN => {
                    data.push_back(self.recv.try_recv()?);
                }

                _ => ()
            }
        }

        Ok(data.into_iter())
    }
}

fn postbox_thread<S, R>(
    mut connection: TcpStream,

    ctrl_rx: Receiver<ControlMsg>,
    send_rx: Receiver<S>,
    recv_tx: Sender<Result<R, PostError>>,

    poll: Poll
) where
    S: PostSend,
    R: PostRecv
{
    let mut events = Events::with_capacity(64);

    // recebendo variáveis relacionadas
    let mut recv_buff = Vec::new();
    let mut recv_nextlen: u64 = 0;

    loop {
        let mut disconnected = false;

        poll.poll(&mut events, None)
            .expect("falha ao executar poll(), aparenta ser algo do sistema operacional");

        for event in events.iter() {
            match event.token() {
                CTRL_TOKEN => match ctrl_rx.try_recv().unwrap() {
                    ControlMsg::Shutdown => return
                },

                CONN_TOKEN => match connection.read_to_end(&mut recv_buff) {
                    Ok(_) => {}

                    // retornado quando todo o dado for lido
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}

                    Err(e) => {
                        recv_tx.send(Err(e.into())).unwrap();
                    }
                },

                DATA_TOKEN => {
                    let mut packet = bincode::serialize(&send_rx.try_recv().unwrap()).unwrap();

                    packet.splice(0..0, (packet.len() as u64).to_be_bytes().iter().cloned());

                    match connection.write_bufs(& [packet.as_slice().into()]) {
                        Ok(_) => {}

                        Err(e) => {
                            recv_tx.send(Err(e.into())).unwrap();
                        }
                    };
                }

                _ => {}
            }
        }

        loop {
            if recv_nextlen == 0 && recv_buff.len() >= 8 {
                recv_nextlen = u64::from_be_bytes(
                    <[u8; 8]>::try_from(recv_buff.drain(0..8).collect::<Vec<u8>>().as_slice())
                        .unwrap()
                );

                if recv_nextlen > MESSAGE_SIZE_CAP {
                    recv_tx.send(Err(PostError::MsgSizeLimitExceeded)).unwrap();

                    connection.shutdown(std::net::Shutdown::Both).unwrap();

                    recv_buff.drain(..);
                    recv_nextlen = 0;

                    break;
                }
            }

            if recv_buff.len() as u64 >= recv_nextlen && recv_nextlen != 0 {
                match bincode::deserialize(recv_buff
                    .drain(
                        0..usize::try_from(recv_nextlen)
                            .expect("tamanho de mensagem é maior que o usize (tamanho de mensagem insano e sistema operacional 32 bit)")
                    )
                    .collect::<Vec<u8>>()
                    .as_slice()
                ) {
                    Ok(ok) => {
                        recv_tx
                            .send(Ok(ok))
                            .unwrap();

                        recv_nextlen = 0;
                    }

                    Err(e) => {
                        recv_tx.send(Err(e.into())).unwrap();
                        recv_nextlen = 0;

                        continue
                    }
                }
            } else {
                break;
            }
        }

        match connection.take_error().unwrap() {
            Some(e) => {
                if e.kind() == ErrorKind::BrokenPipe {
                    disconnected = true;
                }

                recv_tx.send(Err(e.into())).unwrap();
            }

            None => {}
        }

        if disconnected == true {
            break;
        }
    }

    // loop depois de desconectado
    loop {
        poll.poll(&mut events, None)
            .expect("falha ao executar poll(), aparenta ser um problema de sistema operacional");

        for event in events.iter() {
            match event.token() {
                CTRL_TOKEN => match ctrl_rx.try_recv().unwrap() {
                    ControlMsg::Shutdown => return
                },

                _ => {}
            }
        }
    }
}

impl<S, R> Drop for PostBox<S, R>
where
    S: PostSend,
    R: PostRecv
{
    fn drop(&mut self) {
        self.ctrl.send(ControlMsg::Shutdown).unwrap_or(());

        self.handle.take().map(|handle| handle.join());
    }
}
