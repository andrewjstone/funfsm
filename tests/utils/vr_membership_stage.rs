use fsm::sstage::{Sstage, ThreadedSstage};
use fsm::event::{Event};

pub struct VRMembershipStage;

impl Sstage for VRMembershipStage {
    fn handle_callforward<C, T: Event<C> + Sized>(event: T, _: Option<Self>) {
        match event {
            vr_membership_events => {
                println!("received event");
                event.process();
            }
        }

    }

    fn handle_callback<C, T: Event<C> + Sized>(event: T) {

    }
}

pub struct VRMembershipStage2;

impl ThreadedSstage for VRMembershipStage2 {
    fn handle_callforward_internal<C, T: Event<C> + Sized>(event: T) {

    }
}

impl Sstage for VRMembershipStage2 {
    fn handle_callback<C, T: Event<C> + Sized>(event: T) {

    }
    fn handle_callforward<C, T: Event<C> + Sized>(event: T, _: Option<Self>) {
        match event {
            vr_membership_events => {
                println!("received event 2");
                event.process();
            }
        }
    }
}
