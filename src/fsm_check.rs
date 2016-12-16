use fsm::{Fsm, StateFn, FsmTypes};
use constraints::Constraints;

pub struct Checker<T: FsmTypes> {
    fsm: Fsm<T>,
    constraints: Constraints<T::Context>
}

impl<T: FsmTypes> Checker<T> {
    pub fn new(ctx: T::Context, state: StateFn<T>, constraints: Constraints<T::Context>) -> Checker<T> {
        Checker {
            fsm: Fsm::<T>::new(ctx, state),
            constraints: constraints
        }
    }

    // TODO: Use quickcheck and a generator for messages here
    pub fn check(&mut self, msgs: Vec<T::Msg>) -> Result<(), String> {
        for msg in msgs {
            let from = try!(self.check_preconditions());
            self.fsm.send(msg);
            try!(self.check_postconditions(from));
        }
        Ok(())
    }

    pub fn check_preconditions(&self) -> Result<&'static str, String> {
        let (from, ctx) = self.fsm.get_state();
        try!(self.constraints.check_preconditions(from, &ctx));
        Ok(from)
    }

    pub fn check_postconditions(&self, from: &'static str) -> Result<(), String> {
        let (to, ctx) = self.fsm.get_state();
        try!(self.constraints.check_postconditions(from, &ctx));
        try!(self.constraints.check_invariants(&ctx));
        self.constraints.check_transitions(from, to, &ctx)
    }
}
