use fsm::{Fsm, StateFn, FsmTypes};
use constraints::Constraints;

pub struct Checker<T: FsmTypes> {
    pub fsm: Fsm<T>,
    constraints: Constraints<T>
}

impl<T: FsmTypes> Checker<T> {
    pub fn new(ctx: T::Context, state: StateFn<T>, constraints: Constraints<T>) -> Checker<T> {
        Checker {
            fsm: Fsm::<T>::new(ctx, state),
            constraints: constraints
        }
    }

    pub fn check(&mut self, msg: T::Msg) -> Result<Vec<T::Output>, String> {
        let (from, init_ctx) = try!(self.check_preconditions());
        let output = self.fsm.send(msg.clone());
        self.check_postconditions(from, &init_ctx, &msg, &output).map(|_| output)
    }

    pub fn check_preconditions(&self) -> Result<(&'static str, T::Context), String> {
        let (from, ctx) = self.fsm.get_state();
        try!(self.constraints.check_preconditions(from, &ctx));
        try!(self.constraints.check_invariants(&ctx));
        Ok((from, ctx.clone()))
    }

    pub fn check_postconditions(&self,
                                from: &'static str,
                                init_ctx: &T::Context,
                                msg: &T::Msg,
                                output: &Vec<T::Output>) -> Result<(), String> {
        let (to, final_ctx) = self.fsm.get_state();
        try!(self.constraints.check_invariants(&final_ctx));
        self.constraints.check_transition(from, to, init_ctx, final_ctx, msg, output)
    }
}
