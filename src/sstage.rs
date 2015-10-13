use event::{Event};
use std::thread;

pub trait Sstage: Sized {
    fn handle_callforward<C, T: Event<C> + Sized>(event: T, _: Option<Self>);
    fn handle_callback<C, T: Event<C> + Sized>(event: T);
}

struct ThreadedSstageDummy;

impl ThreadedSstage for ThreadedSstageDummy {
        fn handle_callforward_internal<C, T: Event<C> + Sized>(event: T) {

        }
}

impl Sstage for ThreadedSstageDummy {
    fn handle_callforward<C, T: Event<C> + Sized>(event: T, _: Option<Self>) {

    }
    fn handle_callback<C, T: Event<C> + Sized>(event: T) {

    }

}

pub trait ThreadedSstage: Sstage {
    fn handle_callforward_internal<C, T: Event<C> + Sized + 'static>(event: T) {
        thread::spawn(move || {
            Sstage::handle_callforward(event, None::<ThreadedSstageDummy>);
        });
    }
}
