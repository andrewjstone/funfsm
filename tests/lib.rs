#[macro_use]
extern crate fsm;

mod utils;

use std::thread;
use fsm::stage::{Stage, StageThreadModel};
use fsm::Msg;
use fsm::sync_channel::SyncChannel;
use utils::bowl_stage::BowlStage;
use utils::bowl_fsm::CatMsg;

#[test]
fn basic_stage() {
    let mut stage_caller_thread = BowlStage::<SyncChannel>::new("cat_bowl_stage",
                                                    StageThreadModel::CallerThread);
    stage_caller_thread.handle_msg(Box::new(CatMsg::Meow) as Msg);
    let mut stage   = BowlStage::<SyncChannel>::new("cat_bowl_stage",
                                                    StageThreadModel::DedicatedThread);
    let mut sender  = stage.get_sender();
    let handle = thread::spawn(move || {
       stage.start()
    });

    sender.send(Box::new(CatMsg::Meow) as Msg);
    handle.join();
}
