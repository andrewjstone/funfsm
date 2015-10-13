use std::any::Any;

pub type Msg = Box<Any +'static + Send>;

#[derive(Debug)]
pub struct Envelope(pub String, pub Msg);

/// A Channel represents a dynamically typed channel as opposed to the typical static channels
/// provided by the Rust standard library. It sends and receives only `Any` trait objects typed as
/// `Msg`. This allows storing channels and their corresponding senders inside collections since
/// they are all of the same type. It also allows messages to be forwarded among channels without
/// declaring a global enum or struct, by allowing open extension. The one downside is
/// performance as messages must be allocated on the heap and downcast using reflection when
/// received.
pub trait Channel {
    fn get_sender(&self) -> Box<MsgSender>;
    fn recv(&self) -> Msg;
    fn try_recv(&self) -> Option<Msg>;
}

pub enum Status {
    Ok,
    Full
}

/// send() is used for messages that can be dropped. It provides backpressure by returning
/// Status::Full to the caller if the message cannot be sent at this time.
pub trait MsgSender: Send {
    fn send(&mut self, msg: Msg) -> Status;

    /// Some messages cannot be dropped. These are called 'control' messages. They should be sent by
    /// calling send_ctl() which always succeeds.
    fn send_ctl(&mut self, msg: Msg);
}
