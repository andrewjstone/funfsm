use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::JoinHandle;
use std::thread;
use fsm::{Msg, Fsm, FsmContext, FsmHandler};
use local_fsm::LocalFsm;

enum Req {
    GetState,
    TraceOn(String),
    TraceOff,
    FsmMsg(Msg)
}

enum Rpy<T: FsmHandler> {
    State(&'static str, T::Context)
}

pub struct ThreadedFsm<T: 'static + FsmHandler> {
    sender: Sender<Req>,
    receiver: Receiver<Rpy<T>>,
    pub thread: JoinHandle<()>
}

/// Use the local fsm to share the logic
impl<T: FsmHandler> Fsm<T> for ThreadedFsm<T> {
    fn new() -> ThreadedFsm<T> {
        let (client_tx, fsm_rx) = channel();
        let (fsm_tx, client_rx) = channel();

        let handle = thread::spawn(move || {
            let mut local_fsm = LocalFsm::<T>::new();
            loop {
                // Handle debug, config messages
                match fsm_rx.recv() {
                    Ok(Req::GetState) => {
                        let state = local_fsm.get_state();
                        fsm_tx.send(Rpy::State(state.0, state.1)).unwrap();
                    },
                    Ok(Req::TraceOn(path)) => {
                        local_fsm.trace_on(&path);
                    },
                    Ok(Req::TraceOff) => {
                        local_fsm.trace_off();
                    },
                    Ok(Req::FsmMsg(msg)) => {
                        local_fsm.send_msg(msg);
                    },
                    Err(_) => {
                        // The parent thread died, so just exit the thread
                        return;
                    }
                };
            }
        });

        ThreadedFsm {
            sender: client_tx,
            receiver: client_rx,
            thread: handle
        }
    }

    fn get_state(&self) -> (&'static str, T::Context) {
        self.sender.send(Req::GetState).unwrap();
        match self.receiver.recv().unwrap() {
            Rpy::State(name, context) => (name, context)
        }
    }

    fn send_msg(&mut self, msg: Msg) {
        self.sender.send(Req::FsmMsg(msg)).unwrap();
    }

    fn trace_on(&mut self, path: &str) {
        self.sender.send(Req::TraceOn(path.to_string())).unwrap();
    }

    fn trace_off(&mut self) {
        self.sender.send(Req::TraceOff).unwrap();
    }
}
