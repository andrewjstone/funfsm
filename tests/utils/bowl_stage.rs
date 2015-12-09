use fsm::stage::Stage;
use fsm::{Fsm, MsgSender, Channel};
use super::bowl_fsm::{BowlHandler, Context, BowlMsg};

/// A stage that uses and tests a cat_fsm
pub struct BowlStage<T: Channel> {
    _name: String,
    channel: T,
    fsm: Fsm<BowlHandler>
}

impl<T: Channel> BowlStage<T> {
    pub fn new(name: &str, channel: T) -> BowlStage<T> {
        BowlStage {
            _name: name.to_string(),
            channel: channel,
            fsm: Fsm::new(Context::new())
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
            if let Some(bowl_msg) = msg.downcast_ref::<BowlMsg>() {
                // Must clone, because we can't cast a trait object to a concrete type safely and
                // stably
                self.fsm.send_msg(bowl_msg.clone());
            } else {
                println!("Message received by bowl_stage is of incorrect type");
            }
            return;
        }
    }
}
