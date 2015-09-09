#[macro_use]
extern crate fsm;

use fsm::{ThreadedFsm, LocalFsm, Fsm, FsmContext, StateFn, FsmHandler};

#[derive(Debug, Clone)]
pub struct Context {
    pub view_number: u64,
    pub op_number: u64
}

impl FsmContext for Context {
    fn new() -> Context {
        Context {
            view_number: 0,
            op_number: 0
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    Prepare,
    PrepareOk
}

#[derive(Debug)]
pub struct VrHandler;

impl FsmHandler for VrHandler {
    type Context = Context;
    type Msg = Msg;

    fn initial_state() -> StateFn<VrHandler> {
        next!(normal)
    }
}

pub fn normal(ctx: &mut Context, _msg: Msg) -> StateFn<VrHandler> {
    // Mutate ctx as needed
    ctx.view_number += 1;
    // Transition to recovering state
    next!(recovering)
}

pub fn recovering(ctx: &mut Context, _msg: Msg) -> StateFn<VrHandler> {
    // Mutate ctx as needed
    ctx.op_number += 1;
    // Transition to normal state
    next!(normal)
}

fn assert_state_transitions<T: Fsm<VrHandler>>(mut fsm: T) {
    fsm.trace_on("/tmp/fsm_trace.txt");
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "normal");
    assert_eq!(ctx.view_number, 0);
    fsm.send_msg(Msg::Prepare);
    let (name, ctx) = fsm.get_state();
    assert_eq!(name, "recovering");
    assert_eq!(ctx.view_number, 1);
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
