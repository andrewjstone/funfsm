//! This is a simple example of a fsm for feeding a cat. The states of the fsm are the states of
//! the cat food bowl. Our cat is very whiny and will always be fed when her bowl is empty and she
//! meows. If there is already food in the bowl, she will have to eat it before we give her more.

#[macro_use]
extern crate fsm;

use fsm::{ThreadedFsm, LocalFsm, Fsm, FsmContext, StateFn, FsmHandler};
use fsm::constraints::Constraints;
use fsm::constraints;
use fsm::fsm_check::Checker;

#[derive(Debug, Clone)]
pub struct Context {
    pub contents: u8
}

impl FsmContext for Context {
    fn new() -> Context {
        Context {
            contents: 0 // The bowl starts off empty
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    Meow,
    Eat(u8) // % of food to eat
}

#[derive(Debug)]
pub struct BowlHandler;

impl FsmHandler for BowlHandler {
    type Context = Context;
    type Msg = Msg;

    fn initial_state() -> StateFn<BowlHandler> {
        next!(empty)
    }
}

pub fn empty(ctx: &mut Context, msg: Msg) -> StateFn<BowlHandler> {
    if let Msg::Meow = msg {
        // Fill the bowl
        ctx.contents = 100;
        next!(full)
    } else {
        // Can't eat out of an empty bowl
        next!(empty)
    }
}

pub fn full(ctx: &mut Context, msg: Msg) -> StateFn<BowlHandler> {
    if let Msg::Eat(pct) = msg {
        if pct >= ctx.contents {
            ctx.contents = 0;
            next!(empty)
        } else {
            ctx.contents -= pct;
            next!(full)
        }
    } else {
        next!(full)
    }
}

fn assert_state_transitions<T: Fsm<BowlHandler>>(mut fsm: T) {
    fsm.trace_on("/tmp/fsm_trace.txt");
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "empty");
    assert_eq!(ctx.contents, 0);
    fsm.send_msg(Msg::Meow);
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 100);
    fsm.send_msg(Msg::Eat(30));
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 70);
    fsm.send_msg(Msg::Meow);
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 70);
    fsm.send_msg(Msg::Eat(75));
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "empty");
    assert_eq!(ctx.contents, 0);
}

#[test]
fn test_threaded() {
    let fsm = ThreadedFsm::new();
    assert_state_transitions(fsm);
}

#[test]
fn test_local() {
    let fsm = LocalFsm::new();
    assert_state_transitions(fsm);
}

#[test]
fn test_check() {
    let msgs = vec![Msg::Meow, Msg::Eat(30), Msg::Eat(70), Msg::Meow, Msg::Eat(50), Msg::Meow];
    let mut c = Constraints::new();
    precondition!(c, "empty", |ctx: &Context| ctx.contents == 0);
    precondition!(c, "full", |ctx: &Context| ctx.contents > 0 && ctx.contents <= 100);
    postcondition!(c, "empty", |ctx: &Context| ctx.contents == 0 || ctx.contents == 100);
    invariant!(c, |ctx: &Context| ctx.contents <= 100);
    transition!(c, "empty", "full", |ctx: &Context| ctx.contents == 100);
    transition!(c, "full", "empty", |ctx: &Context| ctx.contents == 0);
    let mut checker = Checker::<BowlHandler>::new(c);
    assert_eq!(Ok(()), checker.check(msgs));
}
