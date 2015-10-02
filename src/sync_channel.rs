use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use channel::{Msg, Channel, Status, MsgSender};

impl MsgSender for SyncSender<Msg> {
    fn send(&mut self, msg: Msg) -> Status {
        match self.try_send(msg) {
            Ok(()) => Status::Ok,
            _ => Status::Full
        }
    }
}

pub struct SyncChannel {
    pub tx: SyncSender<Msg>,
    pub rx: Receiver<Msg>
}

impl Channel for SyncChannel {
    fn new(size: usize) -> SyncChannel {
        let (tx, rx) = sync_channel::<Msg>(size);
        SyncChannel {
            tx: tx,
            rx: rx
        }
    }

    fn get_sender(&self) -> Box<MsgSender> {
        Box::new(self.tx.clone()) as Box<MsgSender>
    }

    fn recv(&self) -> Msg {
        self.rx.recv().unwrap()
    }

    fn try_recv(&self) -> Option<Msg> {
        match self.rx.try_recv() {
            Ok(msg) => Some(msg),
            _ => None
        }
    }
}
