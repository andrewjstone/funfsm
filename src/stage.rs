use channel::{Channel, MsgSender};

pub trait Stage<T: Channel> {
    fn get_sender(&self) -> Box<MsgSender>;
    fn start(&mut self);
}
