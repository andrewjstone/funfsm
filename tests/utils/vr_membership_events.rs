use fsm::event::{Event};
use fsm::sstage::{Sstage};
use super::vr_context::{VRContext};
use super::vr_membership_stage::{VRMembershipStage};

pub struct AddNodeEvent<C> {
    name: String,
    context: C,
    currentStage: StagesForCallForward
}

enum StagesForCallForward {
    VRMembershipStage,
    VRMembershipStage2
}

impl<C:VRContext> Event<C> for AddNodeEvent<C> {
    fn is_failed(&self) -> bool {
        return true;
    }

    fn generate_response(&self) -> bool {
        return true;
    }

    fn process(&self) {
        match self.currentStage {
            StagesForCallForward::VRMembershipStage => {
                self.currentStage = StagesForCallForward::VRMembershipStage2;
                <super::vr_membership_stage::VRMembershipStage as Sstage>::handle_callforward(self, None::<super::vr_membership_stage::VRMembershipStage>);
            },
            StagesForCallForward::VRMembershipStage2 => {
                self.generate_response();
            }
        }
    }
}

impl<C:VRContext> AddNodeEvent<C> {
    pub fn new(context: C) -> Self {
        AddNodeEvent {
            name: "TestEvent".to_string(),
            context: context,
            currentStage: StagesForCallForward::VRMembershipStage
        }
    }
}
