use fsm::stage::Stage;
use fsm::{Fsm, Msg, MsgSender, Envelope, Channel, Status, LocalFsm};
use super::bowl_fsm::{BowlHandler, Context};

const QUEUE_SIZE: usize = 1024;

/// A stage that uses and tests a cat_fsm
pub struct BowlStage<T: Channel> {
    name: String,
    channel: T,
    fsm: LocalFsm<BowlHandler>
}

impl<T: Channel> Stage<T> for BowlStage<T> {
    fn new(name: &str) -> BowlStage<T> {
        BowlStage {
            name: name.to_string(),
            channel: T::new(QUEUE_SIZE),
            fsm: LocalFsm::new()
        }
    }

    fn get_sender(&self) -> Box<MsgSender> {
        self.channel.get_sender()
    }

    fn start(&mut self) {
        loop {
            let msg = self.channel.recv();
            self.fsm.send_msg(msg);
            let mut envelopes = self.fsm.get_output_envelopes();
            println!("Envelopes = {:?}", envelopes);
            return;
        }
    }
}
