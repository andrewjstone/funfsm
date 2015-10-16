use event::{Event};
use std::thread;

pub trait Sstage: Sized {
    fn handle_callforward<C, T: Event<C>>(event: T, _: Option<Self>);
    fn handle_callback<C, T: Event<C>>(event: T);
}


// Create this dummy stage so that the threaded stage can call a trait function and compiler
// can resolve types.. this is an issue with Rust.
struct ThreadedSstageDummy;

impl ThreadedSstage for ThreadedSstageDummy {
        fn handle_callforward_internal<C, T: Event<C>>(event: T) {

        }
}

impl Sstage for ThreadedSstageDummy {
    fn handle_callforward<C, T: Event<C> >(event: T, _: Option<Self>) {

    }
    fn handle_callback<C, T: Event<C> + Sized>(event: T) {

    }
}

pub trait ThreadedSstage: Sstage {
    fn handle_callforward<C, T: Event<C> + 'static>(event: T) {
        thread::spawn(move || {
            Sstage::handle_callforward(event, None::<ThreadedSstageDummy>);
        });
    }
    fn handle_callforward_internal<C, T: Event<C> + Sized + 'static>(event: T);
}
