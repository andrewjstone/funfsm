use std::io::Write;
use std::fs::{File, OpenOptions};
use fsm::{Fsm, FsmHandler, StateFn};
use channel::{Msg, Envelope};

pub struct LocalFsm<T: FsmHandler> {
    state: StateFn<T>,
    ctx: T::Context,
    out: Vec<Envelope>,
    trace_file: Option<File>
}

impl<T: FsmHandler> LocalFsm<T> {
    pub fn new(ctx: T::Context) -> LocalFsm<T> {
        LocalFsm {
            state: T::initial_state(),
            ctx: ctx,
            out: Vec::new(),
            trace_file: None
        }
    }

    pub fn get_output_envelopes(&mut self) -> &mut Vec<Envelope> {
        &mut self.out
    }
}

impl<T: FsmHandler> Fsm<T> for LocalFsm<T> {
    fn get_state(&self) -> (&'static str, T::Context) {
        (self.state.0, self.ctx.clone())
    }

    fn send_msg(&mut self, msg: Msg) {
        if let Some(ref mut file) = self.trace_file {
            let StateFn(name, f) = self.state;
            // TODO: Do we want to call unwrap here?
            write!(file, "C: {} {:?}\nM: {:?}\n", name, &self.ctx, &msg).unwrap();
            self.state = f(&mut self.ctx, msg, &mut self.out);
            write!(file, "N: {} {:?}\n", self.state.0, &self.ctx).unwrap();
        } else {
            let StateFn(_name, f) = self.state;
            self.state = f(&mut self.ctx, msg, &mut self.out);
        }
    }

    /// Always use the new path, even if tracing is already enabled. This will
    /// just drop and close the old file. It will also overwrite any content
    /// with the same file name.
    // TODO: Maybe wrap in a BufWriter - Benchmark it.
    fn trace_on(&mut self, path: &str) {
        self.trace_file = Some(OpenOptions::new().write(true).create(true).open(path).unwrap());
    }

    fn trace_off(&mut self) {
        if let Some(ref mut file) = self.trace_file {
            file.sync_all().unwrap();
        };
        self.trace_file = None;
    }
}
