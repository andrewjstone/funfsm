use std::io::{Read, Write};
use std::net::SocketAddr;
use std::collections::HashMap;
use mio;
use mio::{Token, Handler, PollOpt, EventSet};
use mio::tcp::{TcpListener, TcpStream};
use frame::{ReadState, WriteState};
use channel::{MsgSender, Msg, Status};
use error::StdError;

const ACCEPTOR: Token = Token(0);

#[derive(Debug, Clone)]
pub enum OutgoingMsg {
    NewSock(Token),
    Deregister(Token),
    TcpMsg(Token, Vec<u8>),
    Tick(usize)
}

#[derive(Debug, Clone)]
pub enum IncomingMsg {
    Connect(Token, SocketAddr),
    Deregister(Token, StdError),
    WireMsg(Token, Vec<u8>),
    // Timeout is (timeout id, timeout in ms)
    // TODO: Should either generate timeout ids in the event loop or use UUIDs to prevent clobbering
    SetTimeout(usize, u64),
    CancelTimeout(usize),
    Stop
}

pub struct EventLoop {
    addr: SocketAddr,
    event_loop: mio::EventLoop<Context>
}

impl EventLoop {
    pub fn new(listen_address: SocketAddr) -> EventLoop {
        EventLoop {
            addr: listen_address,
            event_loop: mio::EventLoop::new().unwrap()
        }
    }

    /// This call will block the current thread
    pub fn run(&mut self, tx: Box<MsgSender>) {
        let listener = TcpListener::bind(&self.addr).unwrap();
        self.event_loop.register(&listener, ACCEPTOR,
                                 EventSet::readable(),
                                 PollOpt::edge()).unwrap();
        let mut ctx = Context::new(listener, tx);
        self.event_loop.run(&mut ctx).unwrap();
    }

    pub fn sender(&self) -> mio::Sender<IncomingMsg> {
        self.event_loop.channel()
    }
}

struct Conn {
    sock: TcpStream,
    // We use an option simply so we can steal the inner value and own it elsewhere.
    // The value will never actually be `None` for either ReadState or WriteState
    read_state: Option<ReadState>,
    write_state: Option<WriteState>
}

impl Conn {
    fn new(sock: TcpStream) -> Conn {
        Conn {
            sock: sock,
            read_state: Some(ReadState::new()),
            write_state: Some(WriteState::new())
        }
    }
}

struct Context {
    timeouts: HashMap<usize, u64>,
    conns: HashMap<Token, Conn>,
    listener: TcpListener,
    token: usize,
    tx: Box<MsgSender>
}

impl Context {
    fn new(listener: TcpListener, sender: Box<MsgSender>) -> Context {
        Context {
            timeouts: HashMap::new(),
            conns: HashMap::new(),
            listener: listener,
            token: 0,
            tx: sender
        }
    }

    fn next_token(&mut self) -> Token {
        self.token += 1;
        Token(self.token)
    }

    fn accept(&mut self, event_loop: &mut mio::EventLoop<Context>) {
        match self.listener.accept() {
            Ok(None) => (), // EWOULDBLOCK
            Ok(Some((sock, _))) => {
                let token = self.next_token();
                self.register(event_loop, token, sock);
            },
            Err(err) => println!("Error accepting connection: {}", err)
        }
    }
}

impl Handler for Context {
    type Timeout = usize;
    type Message = IncomingMsg;

    fn ready(&mut self, event_loop: &mut mio::EventLoop<Context>, token: Token, event: EventSet)
    {
        if event.is_readable() {
            match token {
                ACCEPTOR => { self.accept(event_loop); }
                _ => {
                    match self.read(event_loop, token) {
                        Ok(()) => (),
                        Err(e) => {
                            println!("Error reading from socket with token: {:?}: {}", token, e);
                            self.deregister(event_loop, token, e);
                        }

                    }
                }
            }
        }

        if event.is_writable() {
            if let Err(err) = self.write(event_loop, token) {
                println!("Got a write error: {} for token {:?}", err, token);
                self.deregister(event_loop, token, err);
            }

        }
    }

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Context>, msg: IncomingMsg) {
        match msg {
            IncomingMsg::Deregister(token, err) => self.deregister(event_loop, token, err),
            IncomingMsg::WireMsg(token, msg) => self.push_outgoing(event_loop, token, msg),
            IncomingMsg::Connect(token, addr) => self.connect(event_loop, token, addr),
            IncomingMsg::Stop => event_loop.shutdown(),
            IncomingMsg::SetTimeout(id, timeout) => {
                event_loop.timeout_ms(id, timeout).unwrap();
                self.timeouts.insert(id, timeout);
            },
            IncomingMsg::CancelTimeout(id) => {
                // Note that it's still possible a tick will come after the removal
                self.timeouts.remove(&id);
            }
        }
    }

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Context>, timeout_id: usize) {
        if let Some(timeout) = self.timeouts.get(&timeout_id) {
            self.tx.send(Box::new(OutgoingMsg::Tick(timeout_id)) as Msg);
            event_loop.timeout_ms(timeout_id, *timeout).unwrap();
        }
    }
}

impl Context {
    fn connect(&mut self, event_loop: &mut mio::EventLoop<Context>, token: Token, addr: SocketAddr) {
        match TcpStream::connect(&addr) {
            Ok(sock) => self.register(event_loop, token, sock),
            Err(_e) => self.tx.send_ctl(Box::new(OutgoingMsg::Deregister(token)) as Msg)
        }
    }

    fn read(&mut self, event_loop: &mut mio::EventLoop<Context>, token: Token) -> Result<(), StdError> {
        let res = match self.conns.get_mut(&token) {
            Some(ref mut conn) => {
                let read_state = conn.read_state.take().unwrap();
                match read_state.read(&mut conn.sock) {
                    (new_read_state, Ok(r)) => {
                        event_loop.reregister(&conn.sock,
                                              token,
                                              EventSet::readable(),
                                              PollOpt::edge() | PollOpt::oneshot()).unwrap();
                        conn.read_state = Some(new_read_state);
                        r
                    },
                    (_, Err(e)) => return Err(e)
                }
            },
            None => None
        };
        if let Some(buf) = res {
            self.tx.send(Box::new(OutgoingMsg::TcpMsg(token, buf)) as Msg);
        }
        Ok(())
    }

    fn push_outgoing(&mut self, event_loop: &mut mio::EventLoop<Context>, token: Token, msg: Vec<u8>) {

        if let Some(ref mut conn) = self.conns.get_mut(&token) {
            let write_state = conn.write_state.take().unwrap();
            let new_write_state = write_state.push(msg);
            conn.write_state = Some(new_write_state);
            event_loop.reregister(&conn.sock,
                                  token,
                                  EventSet::writable(),
                                  PollOpt::edge() | PollOpt::oneshot()).unwrap();
        }
    }

    fn write(&mut self, event_loop: &mut mio::EventLoop<Context>, token: Token)
        -> Result<(), StdError> {
        // It's possible we already deregistered and one last write gets triggered
        self.conns.get_mut(&token).map(|ref mut conn| {
            let write_state = conn.write_state.take().unwrap();
            match write_state.write(&mut conn.sock) {
                Ok((true, new_write_state)) => {
                    event_loop.reregister(&conn.sock,
                                          token,
                                          EventSet::writable(),
                                          PollOpt::edge() | PollOpt::oneshot()).unwrap();
                    conn.write_state = Some(new_write_state);
                    return Ok(())
                },
                Ok((false, new_write_state)) => {
                    conn.write_state = Some(new_write_state);
                    return Ok(());
                },
                Err(e) => return Err(e)
            }
        });
        Ok(())
    }

    fn register(&mut self, event_loop: &mut mio::EventLoop<Context>, token: Token, sock: TcpStream) {
        match self.tx.send(Box::new(OutgoingMsg::NewSock(token)) as Msg) {
            Status::Ok => {
                event_loop.register(&sock, token,
                            EventSet::readable() | EventSet::writable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();
                let conn = Conn::new(sock);
                self.conns.insert(token, conn);
            },
            Status::Full => () // just drop the sock, forcing a disconnect since we are full
        }
    }

    fn deregister(&mut self, event_loop: &mut mio::EventLoop<Context>, token: Token, err: StdError) {
        if let Some(conn) = self.conns.remove(&token) {
            event_loop.deregister(&conn.sock).unwrap();
            println!("Deregistered cluster socket for token {:?} with error: {}", token, err);
            self.tx.send_ctl(Box::new(OutgoingMsg::Deregister(token)) as Msg);
        } else {
            println!("Error: Tried to deregister a token with no corresponding socket");
        }
    }
}
