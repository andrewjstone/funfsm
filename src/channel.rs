use std::any::Any;

pub type Msg = Box<Any +'static + Send>;

#[derive(Debug)]
pub struct Envelope(pub String, pub Msg);

pub trait Channel {
    fn get_sender(&self) -> Box<MsgSender>;
    fn recv(&self) -> Msg;
    fn try_recv(&self) -> Option<Msg>;
}

pub enum Status {
    Ok,
    Full
}

pub trait MsgSender: Send {
    // Never blocks
    fn send(&mut self, msg: Msg) -> Status;
    fn send_ctl(&mut self, msg: Msg);
}
