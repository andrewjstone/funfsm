#![feature(thread_sleep)]

extern crate fsm;
extern crate mio;

use mio::{Token};
use std::net::TcpStream;
use std::str;
use std::thread;
use std::time::Duration;
use std::io::Write;
use fsm::event_loop::{EventLoop, OutgoingMsg, IncomingMsg};
use fsm::channel::{Channel, Msg};
use fsm::heuristic_channel::HeuristicChannel;
use fsm::frame;

fn always_true() -> bool {
    true
}

#[test]
fn tcp_with_4_byte_framed_messages() {
    let addr = "127.0.0.1:9494";
    let mut event_loop = EventLoop::new(addr.parse().unwrap());
    // mio channel to send to event loop
    let event_loop_sender = event_loop.sender();
    // Channel that event loop sends on
    let channel = HeuristicChannel::new(always_true);
    let msg_sender = channel.get_sender();

    let h1 = thread::spawn(move || {
        event_loop.run(msg_sender);
    });

    // This is ugly, but we need to wait for the server to start listening
    thread::sleep(Duration::from_secs(1));

    let mut sock = TcpStream::connect(addr).unwrap();
    let msg = channel.recv();
    let token = assert_connection(msg);

    let tcp_msg = b"hello world";
    send_tcp_msg(&mut sock, tcp_msg);
    let msg = channel.recv();
    assert_tcp_msg_received(token, msg, tcp_msg);

    assert_timer_tick(&channel, &event_loop_sender);

    // Shutdown the event loop
    event_loop_sender.send(IncomingMsg::Stop).unwrap();

    h1.join().unwrap();
}

fn assert_timer_tick(channel: &HeuristicChannel, sender: &mio::Sender<IncomingMsg>) {
    sender.send(IncomingMsg::SetTimeout(9999, 100)).unwrap();
    let msg = channel.recv();
    if let Some(&OutgoingMsg::Tick(id)) = msg.downcast_ref::<OutgoingMsg>() {
        assert_eq!(id, 9999);
    } else {
        assert!(false);
    }
}

fn assert_tcp_msg_received(token: Token, received_msg: Msg, sent: &[u8]) {
    if let Some(&OutgoingMsg::TcpMsg(token2, ref vec)) = received_msg.downcast_ref::<OutgoingMsg>() {
        assert_eq!(token, token2);
        assert_eq!(&vec[..], sent);
        println!("Received: {}", str::from_utf8(vec).unwrap());
    } else {
        assert!(false);
    }
}

fn send_tcp_msg(sock: &mut TcpStream, msg: &[u8]) {
    let header = frame::u32_to_vec(msg.len() as u32);
    sock.write_all(&header[..]).unwrap();
    sock.write_all(msg).unwrap();
}

fn assert_connection(msg: Msg) -> Token {
    if let Some(&OutgoingMsg::NewSock(token)) = msg.downcast_ref::<OutgoingMsg>() {
        token
    } else {
        assert!(false);
        Token(0)
    }
}
