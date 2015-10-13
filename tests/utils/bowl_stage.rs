use fsm::stage::{Stage, StageThreadModel};
use fsm::{Fsm, Msg, MsgSender, Channel, Status, LocalFsm};
use super::bowl_fsm::{BowlHandler};

const QUEUE_SIZE: usize = 1024;

/// A stage that uses and tests a cat_fsm
pub struct BowlStage<T: Channel> {
    name: String,
    channel: T,
    thread_model: StageThreadModel,
    fsm: LocalFsm<BowlHandler>
}

impl<T: Channel> BowlStage<T> {
    pub fn new(name: &str, thread_model: StageThreadModel) -> BowlStage<T> {
        BowlStage {
            name: name.to_string(),
            channel: T::new(QUEUE_SIZE),
            fsm: LocalFsm::new(),
            thread_model: thread_model
        }
    }
}

impl<T: Channel> Stage for BowlStage<T> {

    fn get_sender(&self) -> Box<MsgSender> {
        self.channel.get_sender()
    }

    fn start(&mut self) {
        loop {
            let msg = self.channel.recv();
            self.fsm.send_msg(msg);
            let envelopes = self.fsm.get_output_envelopes();
            println!("Envelopes = {:?}", envelopes);
            return;
        }
    }

    fn get_thread_model(&self) -> StageThreadModel {
        return self.thread_model.clone();
    }

    fn handle_msg_internal(&mut self, msg: Msg) -> Status {
        self.fsm.send_msg(msg);
        let envelopes = self.fsm.get_output_envelopes();
        println!("Envelopes = {:?}", envelopes);
        return Status::Ok;
    }
}
