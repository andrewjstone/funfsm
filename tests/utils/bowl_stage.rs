use fsm::stage::Stage;
use fsm::{Fsm, MsgSender, Channel, LocalFsm};
use super::bowl_fsm::BowlHandler;

/// A stage that uses and tests a cat_fsm
pub struct BowlStage<T: Channel> {
    _name: String,
    channel: T,
    fsm: LocalFsm<BowlHandler>
}

impl<T: Channel> BowlStage<T> {
    pub fn new(name: &str, channel: T) -> BowlStage<T> {
        BowlStage {
            _name: name.to_string(),
            channel: channel,
            fsm: LocalFsm::new()
        }
    }
}

impl<T: Channel> Stage<T> for BowlStage<T> {
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
}
