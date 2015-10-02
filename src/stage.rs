use channel::{Channel, MsgSender};

pub trait Stage<T: Channel> {
    fn new(name: &str) -> Self;
    fn get_sender(&self) -> Box<MsgSender>;
    fn start(&mut self);
}
