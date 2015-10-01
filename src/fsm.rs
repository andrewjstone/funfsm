use std::fmt::Debug;
use std::any::Any;

#[macro_export]
macro_rules! next {
     ($x:ident) => {
         StateFn(stringify!($x), $x)
     }
}

pub type Msg = Box<Any +'static + Send>;

#[derive(Debug)]
pub struct Envelope(pub String, pub Msg);

pub trait Fsm<T: FsmHandler> {
    fn new() -> Self;
    fn get_state(&self) -> (&'static str, T::Context);
    fn send_msg(&mut self, msg: Msg);
    fn trace_on(&mut self, _path: &str);
    fn trace_off(&mut self);
}

pub trait FsmContext {
    fn new() -> Self;
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
    type Context: FsmContext + Send + Clone + Debug;

    fn initial_state() -> StateFn<Self>;
}

