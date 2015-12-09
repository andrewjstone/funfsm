use std::fmt::Debug;

#[macro_export]
macro_rules! next {
     ($x:ident) => {
         StateFn(stringify!($x), $x)
     }
}

// A recursive tuple struct indicating the name of current state and the function pointer that
// handles messages in that that state. Calling that function returns the next state in a
// StateFn<T>.
pub struct StateFn<T: FsmHandler>(
    pub &'static str,
    pub fn(&mut T::Context, T::Msg) -> StateFn<T>
);

// Function pointers aren't `Clone` due to this [bug](https://github.com/rust-lang/rust/issues/24000)
// Since we can't derive clone, we just implement it manually, since function pointers are `Copy`
impl<T: FsmHandler> Clone for StateFn<T> {
    fn clone(&self) -> StateFn<T> {
        StateFn(self.0, self.1)
    }
}

#[derive(Clone)]
pub struct Fsm<T: FsmHandler> {
    pub state: StateFn<T>,
    pub ctx: T::Context
}

impl<T: FsmHandler> Fsm<T> {
    pub fn new(ctx: T::Context) -> Fsm<T> {
        Fsm {
            state: T::initial_state(),
            ctx: ctx,
        }
    }

    pub fn get_state(&self) -> (&'static str, T::Context) {
        (self.state.0, self.ctx.clone())
    }

    pub fn send_msg(&mut self, msg: T::Msg) {
        let StateFn(_name, f) = self.state;
        self.state = f(&mut self.ctx, msg);
    }
}

pub trait FsmHandler: Sized {
    // The application state of the fsm
    type Context: Send + Clone + Debug;
    type Msg: Send + Clone + Debug;

    fn initial_state() -> StateFn<Self>;
}

