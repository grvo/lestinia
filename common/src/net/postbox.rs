// padrão
use std::{
    collections::VecDeque,
    convert::TryFrom,

    io::{
        ErrorKind,
        Read
    },

    net::SocketAddr,
    time::Duration,

    thread,

    sync::mpsc::TryRecvError
};

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
use super::{
    data::ControlMsg,

    error::{
        PostError,
        PostErrorInternal
    },
    
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
    recv: Receiver<Result<R>, PostErrorInternal>,
    send: Sender<S>,
    poll: Poll,
    err: Option<PostErrorInternal>
}

impl<S, R> PostBox<S, R>
where
    S: PostSend,
    R: PostRecv
{
    /// cria um novo [`postbox`] conectado para endereços específicos, podendo ser utilizado pelo cleint
    pub fn to_server<A: Into<SocketAddr>>(addr: A) -> Result<PostBox<S, R>, PostError> {
        let connection = TcpStream::connect(&addr.into())?;

        Self::from_tcpstream(connection)
    }

    /// cria um novo [`postbox`] por meio de uma conexão existente, podendo ser utilizado por [`postoffice`] no servidor
    pub fn from_tcpstream(connection: TcpStream) -> Result<PostBox<S, R>, PostError> {
        let (ctrl_tx, ctrl_rx) = channel(); // mensagens de controle
        let (send_tx, send_rx) = channel(); // thread principal
        let (recv_tx, recv_rx) = channel(); // thread principal - thread trabalhadora

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

            poll: postbox_poll,
    
            err: None
        })
    }

    /// retorna uma `option<posterror>` indicando o status atual de `postbox`
    pub fn status(&self) -> Option<PostError> {
        self.err.as_ref().map(|err| err.into())
    }

    /// método sender não-bloqueável
    pub fn send(&mut self, data: S) -> Result<(), PostError> {
        match &mut self.err {
            err @ None => if let Err(_) = self.send.send(data) {
                *err = Some(PostErrorInternal::MioError);

                Err(err.as_ref().unwrap().into())
            } else {
                Ok(())
            },

            err => Err(err.as_ref().unwrap().into())
        }
    }

    /// método receptor não-bloqueável retornando um iterator após recber objetos deserializados
    /// # erros
    /// se o outro lado se desconectar do postbox, tentar algo novo para enviar
    pub fn new_messages(&mut self) -> impl ExacSizeIterator<Item = R> {
        let mut events = Events::with_capacity(4096);

        let mut items = VecDeque::new();

        // se ocorrer um erro, ou caso tenha ocorrido antes, deixar pra lá
        if let Some(_) = self.err {
            return items.into_iter();
        } else if let Err(err) = self.poll.poll(&mut events, Some(Duration::new(0, 0))) {
            self.err = Some(err.into());

            return items.into_iter();
        }

        for event in events {
            match event.token() {
                DATA_TOKEN => loop {
                    match self.recv.try_recv() {
                        Ok(Ok(item)) => items.push_back(item),

                        Err(TryRecvError::Empty) => break,

                        Err(err) => self.err = Some(err.into()),
                        Ok(Err(err)) => self.err = Some(err.into())
                    }
                },

                _ => ()
            }
        }

        items.into_iter()
    }
}

fn postbox_thread<S, R>(
    mut connection: TcpStream,

    ctrl_rx: Receiver<ControlMsg>,
    send_rx: Receiver<S>,
    recv_tx: Sender<Result<R, PostErrorInternal>>,

    poll: Poll
) where
    S: PostSend,
    R: PostRecv
{
    // recebendo variáveis relacionadas
    let mut events = Events::with_capacity(64);
    let mut recv_buff = Vec::new();
    let mut recv_nextlen: u64 = 0;

    loop {
        let mut disconnected = false;

        poll.poll(&mut events, Some(Duration::from_millis(20)))
            .expect("falha ao executar poll(), aparenta ser algo do sistema operacional");

        println!("poll finalizado!")

        for event in events.iter() {
            println!("evento!");
            
            match event.token() {
                CTRL_TOKEN => match ctrl_rx.try_recv().unwrap() {
                    ControlMsg::Shutdown => return
                },

                CONN_TOKEN => match connection.read_to_end(&mut recv_buff) {
                    Ok(_) => {}

                    // retornado quando todo o dado for lido
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}

                    Err(e) => recv_tx.send(Err(e.into())).unwrap();
                },

                DATA_TOKEN => {
                    let msg = send_rx.try_recv().unwrap();
                    println!("enviar: {:?}", msg);
                    let mut packet = bincode::serialize(&msg).unwrap();

                    packet.splice(0..0, (packet.len() as u64).to_be_bytes().iter().cloned());

                    match connection.write_bufs(& [packet.as_slice().into()]) {
                        Ok(_) => {
                            println("enviado!");
                        }

                        Err(e) => {
                            println!("enviar erro!");
                            
                            recv_tx.send(Err(e.into())).unwrap();
                        }
                    };
                }

                _ => {}
            }
        }

        loop {
            if recv_nextlen == 0 && recv_buff.len() >= 8 {
                println!("ler nextlen");
                
                recv_nextlen = u64::from_be_bytes(
                    <[u8; 8]>::try_from(recv_buff.drain(0..8).collect::<Vec<u8>>().as_slice()).unwrap()
                );

                if recv_nextlen > MESSAGE_SIZE_CAP {
                    recv_tx.send(Err(PostErrorInternal::MsgSizeLimitExceeded)).unwrap();

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
                    Ok(msg) => {
                        println!("recebido: {:?}", msg);
                        
                        recv_tx
                            .send(Ok(msg))
                            .unwrap();

                        recv_nextlen = 0;
                    }

                    Err(e) => {
                        println!("erro recebido: {:?}", e);
                        
                        recv_tx.send(Err(e.into())).unwrap();
                        recv_nextlen = 0;
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
