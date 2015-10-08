#[macro_use]
extern crate fsm;

mod utils;

use std::thread;
use fsm::stage::Stage;
use fsm::Msg;
use fsm::heuristic_channel::HeuristicChannel;
use utils::bowl_stage::BowlStage;
use utils::bowl_fsm::CatMsg;

fn always_true() -> bool {
    true
}

#[test]
fn basic_stage() {
    let channel = HeuristicChannel::new(always_true);
    let mut stage = BowlStage::new("cat_bowl_stage", channel);
    let mut tx = stage.get_sender();
    let handle = thread::spawn(move || {
       stage.start()
    });

    tx.send(Box::new(CatMsg::Meow) as Msg);
    handle.join().unwrap();
}
