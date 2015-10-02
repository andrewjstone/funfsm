//! This is a simple example of a fsm for feeding a cat. The states of the fsm are the states of
//! the cat food bowl. Our cat is very whiny and will always be fed when her bowl is empty and she
//! meows. If there is already food in the bowl, she will have to eat it before we give her more.

use fsm::{Msg, ThreadedFsm, LocalFsm, Fsm, FsmContext, StateFn, FsmHandler, Envelope};
use fsm::constraints::Constraints;
use fsm::constraints;
use fsm::fsm_check::Checker;

const MAX_RESERVES: u8 = 10;
const REFILL_THRESHOLD: u8 = 9;

// Currently the pub members exist because constraint checking happens outside the impl
// TODO: Do we move the constraints in?
#[derive(Debug, Clone)]
pub struct Context {
    pub contents: u8, // % of the bowl that is full
    pub reserves: u8, // The amount of bowls of food left in the bag
}

impl FsmContext for Context {
    fn new() -> Context {
        Context {
            contents: 0, // The bowl starts off empty
            reserves: MAX_RESERVES,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CatMsg {
    Meow,
    Eat(u8) // % of food to eat
}

#[derive(Debug, Clone)]
pub enum StoreReq {
    Buy(u8)
}

#[derive(Debug, Clone)]
pub enum StoreRpy {
    Bowls(u8)
}

#[derive(Debug)]
pub struct BowlHandler;

impl FsmHandler for BowlHandler {
    type Context = Context;

    fn initial_state() -> StateFn<BowlHandler> {
        next!(empty)
    }
}

pub fn empty(ctx: &mut Context, msg: Msg, out: &mut Vec<Envelope>) -> StateFn<BowlHandler> {

    if let Some(&CatMsg::Meow) = msg.downcast_ref::<CatMsg>() {
        if ctx.reserves > 0 {
            // Fill the bowl
            ctx.contents = 100;
            ctx.reserves -= 1;
            if ctx.reserves <= REFILL_THRESHOLD {
                let refill = Box::new(StoreReq::Buy(MAX_RESERVES - ctx.reserves)) as Msg;
                out.push(Envelope("cat_food_store".to_string(), refill));
            }
            return next!(full)
        } else {
            return next!(empty)
        }
    }

    if let Some(&StoreRpy::Bowls(num)) = msg.downcast_ref::<StoreRpy>() {
        ctx.reserves += num-1;
        ctx.contents = 100;
        return next!(full)
    }

    next!(empty)
}

pub fn full(ctx: &mut Context, msg: Msg, _out: &mut Vec<Envelope>) -> StateFn<BowlHandler> {
    if let Some(&CatMsg::Eat(pct)) = msg.downcast_ref::<CatMsg>() {
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
    fsm.send_msg(Box::new(CatMsg::Meow) as Msg);
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 100);
    fsm.send_msg(Box::new(CatMsg::Eat(30)) as Msg);
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 70);
    fsm.send_msg(Box::new(CatMsg::Meow) as Msg);
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 70);
    fsm.send_msg(Box::new(CatMsg::Eat(75)) as Msg);
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
    let v = vec![CatMsg::Meow, CatMsg::Eat(30), CatMsg::Eat(70), CatMsg::Meow, CatMsg::Eat(50), CatMsg::Meow];
    let msgs = v.iter().cloned().map(|msg| Box::new(msg) as Msg).collect();
    check_constraints(msgs);
}

fn check_constraints(msgs: Vec<Msg>) {
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
