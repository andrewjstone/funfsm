use std::fmt::Debug;

#[macro_export]
macro_rules! next {
     ($x:ident) => {
         StateFn(stringify!($x), $x)
     }
}

pub trait Fsm<T: FsmHandler> {
    fn new() -> Self;
    fn get_state(&self) -> (&'static str, T::Context);
    fn send_msg(&mut self, msg: T::Msg);
    fn trace_on(&mut self, _path: &str);
    fn trace_off(&mut self);
}

pub trait FsmContext {
    fn new() -> Self;
}

pub struct StateFn<T: FsmHandler>(pub &'static str, pub fn(&mut T::Context, T::Msg) -> StateFn<T>);

pub trait FsmHandler: Sized {
    // The application state of the fsm
    type Context: FsmContext + Send + Clone + Debug;

    // A message handled by the fsm
    type Msg: Send + Debug;

    fn initial_state() -> StateFn<Self>;
}

