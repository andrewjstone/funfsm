#[macro_use]
extern crate fsm;

mod utils;

use std::thread::JoinHandle;
use std::thread;
use fsm::stage::Stage;
use fsm::Msg;
use fsm::sync_channel::SyncChannel;
use utils::bowl_stage::BowlStage;
use utils::bowl_fsm::CatMsg;

#[test]
fn basic_stage() {
    let mut stage = BowlStage::<SyncChannel>::new("cat_bowl_stage");
    let mut tx = stage.get_sender();
    let handle = thread::spawn(move || {
       stage.start()
    });

    tx.send(Box::new(CatMsg::Meow) as Msg);
    handle.join();
}
