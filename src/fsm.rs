use std::fmt::Debug;
use channel::Envelope;

#[macro_export]
macro_rules! next {
     ($x:ident) => {
         StateFn(stringify!($x), $x)
     }
}

pub enum FsmType {
    Local,
    Threaded
}

pub trait Fsm<T: FsmHandler> {
    fn get_state(&self) -> (&'static str, T::Context);
    fn send_msg(&mut self, msg: T::Msg);
    fn trace_on(&mut self, _path: &str);
    fn trace_off(&mut self);
}

// A recursive tuple struct indicating the name of current state and the function pointer that
// handles messages in that that state. Calling that function returns the next state in a
// StateFn<T>.
pub struct StateFn<T: FsmHandler>(
    pub &'static str,
    pub fn(&mut T::Context, T::Msg, &mut Vec<Envelope>) -> StateFn<T>
);

pub trait FsmHandler: Sized {
    // The application state of the fsm
    type Context: Send + Clone + Debug;
    type Msg: Send + Clone + Debug;

    fn initial_state() -> StateFn<Self>;
}

