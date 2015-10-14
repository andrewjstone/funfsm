use std::fmt::Debug;
use channel::{Msg, Envelope};

#[macro_export]
macro_rules! next {
     ($x:ident) => {
         StateFn(stringify!($x), $x)
     }
}

pub trait Fsm<T: FsmHandler> {
    fn new(T::Context) -> Self;
    fn get_state(&self) -> (&'static str, T::Context);
    fn send_msg(&mut self, msg: Msg);
    fn trace_on(&mut self, _path: &str);
    fn trace_off(&mut self);
}

// A recursive tuple struct indicating the name of current state and the function pointer that
// handles messages in that that state. Calling that function returns the next state in a
// StateFn<T>.
pub struct StateFn<T: FsmHandler>(
    pub &'static str,
    pub fn(&mut T::Context, Msg, &mut Vec<Envelope>) -> StateFn<T>
);

pub trait FsmHandler: Sized {
    // The application state of the fsm
    type Context: Send + Clone + Debug;

    fn initial_state() -> StateFn<Self>;
}

