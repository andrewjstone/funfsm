use channel::{Channel, Msg, Status, MsgSender};

#[derive(Clone)]
pub enum StageThreadModel {
    DedicatedThread,
    CallerThread,
}

pub trait Stage {
    fn get_sender(&self) -> Box<MsgSender>;
    fn get_thread_model(&self) -> StageThreadModel;

    fn handle_msg(&mut self, msg:Msg) -> Status {
        let thread_model = self.get_thread_model();
        match thread_model {
            StageThreadModel::DedicatedThread =>
                    self.get_sender().send(Box::new(msg) as Msg),
            StageThreadModel::CallerThread => self.handle_msg_internal(msg)
        }
    }

    fn start(&mut self);
    fn handle_msg_internal(&mut self, msg:Msg) -> Status;
}
