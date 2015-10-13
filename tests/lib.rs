#[macro_use]
extern crate fsm;

mod utils;

use std::thread;
use std::sync::Arc;
use fsm::stage::{Stage, StageThreadModel};
use fsm::{Channel, Msg};
use fsm::sync_channel::SyncChannel;
use utils::bowl_stage::BowlStage;
use utils::bowl_fsm::CatMsg;
use std::collections::hash_map::HashMap;
use std::hash::Hash;

#[test]
fn basic_stage() {
    let mut stage_map: HashMap<String, Box<Stage>> = HashMap::new();
    let mut stage_caller_thread = BowlStage::<SyncChannel>::new("cat_bowl_stage",
                                                    StageThreadModel::CallerThread);
    stage_caller_thread.handle_msg(Box::new(CatMsg::Meow) as Msg);
    let mut stage   = BowlStage::<SyncChannel>::new("cat_bowl_stage",
                                                    StageThreadModel::DedicatedThread);
    let stage1str = "stage1".to_string();
    stage_map.insert(stage1str, Box::new(stage));
    let shared_map = Arc::new(stage_map);
    let handle = thread::spawn(move || {
        match shared_map.get(&"stage1".to_string()) {
            Some(stage) => {
                    stage.start();
                }
            None => ()
        }
    });
    match stage_map.get(&stage1str) {
        Some(stage) => {
            let mut sender  = stage.get_sender();
            sender.send(Box::new(CatMsg::Meow) as Msg);
            handle.join();
        }
        None => ()
    }
}
