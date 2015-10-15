use std::sync::mpsc::{channel, Sender, Receiver};
use channel::{Msg, Channel, Status, MsgSender};

#[derive(Clone)]
pub struct HeuristicSender {
    predicate: fn() -> bool,
    tx: Sender<Msg>
}

impl HeuristicSender {
    pub fn new(tx: Sender<Msg>, predicate: fn() -> bool) -> HeuristicSender {
        HeuristicSender {
            predicate: predicate,
            tx: tx
        }
    }
}

impl MsgSender for HeuristicSender {
    fn send(&mut self, msg: Msg) -> Status {
        // It's possible that predicate is expensive. We should either ensure that this isn't the case or
        // not call it on every send using some other heuristic such as number of or time between
        // calls.
        let f = self.predicate;
        if f() {
            self.tx.send(msg).unwrap();
            Status::Ok
        } else {
            Status::Full
        }
    }

    fn send_ctl(&mut self, msg: Msg) {
        self.tx.send(msg).unwrap();
    }
}

pub struct HeuristicChannel {
    pub tx: HeuristicSender,
    pub rx: Receiver<Msg>
}

impl HeuristicChannel {
    pub fn new(predicate: fn() -> bool) -> HeuristicChannel {
        let (tx, rx) = channel();
        let sender = HeuristicSender::new(tx, predicate);
        HeuristicChannel {
            tx: sender,
            rx: rx
        }
    }
}

impl Channel for HeuristicChannel {
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
