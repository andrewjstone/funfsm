//! This is a simple example of a fsm for feeding a cat. The states of the fsm are the states of
//! the cat food bowl. Our cat is very whiny and will always be fed when her bowl is empty and she
//! meows. If there is already food in the bowl, she will have to eat it before we give her more.

#[macro_use]
extern crate fsm;

use fsm::{Fsm, StateFn, FsmHandler};
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

impl Context {
    pub fn new() -> Context {
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

#[derive(Debug, Clone)]
pub enum BowlMsg {
    CatMsg(CatMsg),
    StoreReq(StoreReq),
    StoreRpy(StoreRpy)
}

#[derive(Debug)]
pub struct BowlHandler;

impl FsmHandler for BowlHandler {
    type Context = Context;
    type Msg = BowlMsg;

    fn initial_state() -> StateFn<BowlHandler> {
        next!(empty)
    }
}

pub fn empty(ctx: &mut Context, msg: BowlMsg) -> StateFn<BowlHandler> {

    if let BowlMsg::CatMsg(CatMsg::Meow) = msg {
        if ctx.reserves > 0 {
            // Fill the bowl
            ctx.contents = 100;
            ctx.reserves -= 1;
            if ctx.reserves <= REFILL_THRESHOLD {
                // We'd send a refill request here in a real system
            }
            return next!(full)
        } else {
            return next!(empty)
        }
    }

    if let BowlMsg::StoreRpy(StoreRpy::Bowls(num)) = msg {
        ctx.reserves += num-1;
        ctx.contents = 100;
        return next!(full)
    }

    next!(empty)
}

pub fn full(ctx: &mut Context, msg: BowlMsg) -> StateFn<BowlHandler> {
    if let BowlMsg::CatMsg(CatMsg::Eat(pct)) = msg {
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

#[test]
fn test_state_transitions() {
    let mut fsm = Fsm::<BowlHandler>::new(Context::new());
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "empty");
    assert_eq!(ctx.contents, 0);
    fsm.send_msg(BowlMsg::CatMsg(CatMsg::Meow));
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 100);
    fsm.send_msg(BowlMsg::CatMsg(CatMsg::Eat(30)));
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 70);
    fsm.send_msg(BowlMsg::CatMsg(CatMsg::Meow));
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "full");
    assert_eq!(ctx.contents, 70);
    fsm.send_msg(BowlMsg::CatMsg(CatMsg::Eat(75)));
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "empty");
    assert_eq!(ctx.contents, 0);
}

#[test]
fn test_check() {
    let msgs = vec![BowlMsg::CatMsg(CatMsg::Meow),
                 BowlMsg::CatMsg(CatMsg::Eat(30)),
                 BowlMsg::CatMsg(CatMsg::Eat(70)),
                 BowlMsg::CatMsg(CatMsg::Meow),
                 BowlMsg::CatMsg(CatMsg::Eat(50)),
                 BowlMsg::CatMsg(CatMsg::Meow)];
    check_constraints(msgs);
}

fn check_constraints(msgs: Vec<BowlMsg>) {
    let mut c = Constraints::new();
    precondition!(c, "empty", |ctx: &Context| ctx.contents == 0);
    precondition!(c, "full", |ctx: &Context| ctx.contents > 0 && ctx.contents <= 100);
    postcondition!(c, "empty", |ctx: &Context| ctx.contents == 0 || ctx.contents == 100);
    invariant!(c, |ctx: &Context| ctx.contents <= 100);
    transition!(c, "empty", "full", |ctx: &Context| ctx.contents == 100);
    transition!(c, "full", "empty", |ctx: &Context| ctx.contents == 0);
    let mut checker = Checker::<BowlHandler>::new(Context::new(), c);
    assert_eq!(Ok(()), checker.check(msgs));
}
