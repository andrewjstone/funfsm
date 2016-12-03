use std::fmt::Debug;

#[macro_export]
macro_rules! next {
     ($state:ident) => {
         (StateFn(stringify!($state), $state), Vec::new())
     };
     ($state:ident, $output:expr) => {
         (StateFn(stringify!($state), $state), $output)
     };
}

#[macro_export]
macro_rules! state_fn {
     ($state:ident) => {
         StateFn(stringify!($state), $state)
     }
}

pub trait FsmTypes: Sized {
    // The application state of the fsm
    type Context: Send + Clone + Debug;
    type Msg: Send + Clone + Debug;
    type Output: Send + Clone + Debug;
}

// A recursive tuple struct indicating the name of current state and the function pointer that
// handles messages in that that state. Calling that function returns a pair containing the next
// state and any output.
pub struct StateFn<T: FsmTypes>(
    pub &'static str,
    pub fn(&mut T::Context, T::Msg) -> (StateFn<T>, Vec<T::Output>)
);

// Function pointers aren't `Clone` due to this [bug](https://github.com/rust-lang/rust/issues/24000)
// Since we can't derive clone, we just implement it manually, since function pointers are `Copy`
impl<T: FsmTypes> Clone for StateFn<T> {
    fn clone(&self) -> StateFn<T> {
        StateFn(self.0, self.1)
    }
}

#[derive(Clone)]
pub struct Fsm<T: FsmTypes> {
    pub state: StateFn<T>,
    pub ctx: T::Context
}

impl<T: FsmTypes> Fsm<T> {
    pub fn new(ctx: T::Context, state: StateFn<T>) -> Fsm<T> {
        Fsm {
            state: state,
            ctx: ctx
        }
    }

    pub fn get_state(&self) -> (&'static str, &T::Context) {
        (self.state.0, &self.ctx)
    }

    pub fn send(&mut self, msg: T::Msg) -> Vec<T::Output> {
        let StateFn(_name, f) = self.state;
        let (new_state, output) = f(&mut self.ctx, msg);
        self.state = new_state;
        output
    }
}
