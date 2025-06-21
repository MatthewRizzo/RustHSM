use rust_hsm::state::StateIF;

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};

pub(crate) struct LightStateTop {
    shared_data: LightHsmDataRef,
    // Not every state needs a delegate!
}

impl LightStateTop {
    pub fn new(shared_data: LightHsmDataRef) -> Box<Self> {
        let built_state = Box::new(Self { shared_data });

        built_state
    }
}

impl StateIF<LightStates, LightEvents> for LightStateTop {
    fn handle_event(&self, event: &LightEvents) -> bool {
        // top returns true for all events
        match event {
            _ => true,
        }
    }
    fn handle_state_start(&self) {
        self.shared_data.borrow_mut().top_start_called += 1;
    }

    fn handle_state_enter(&self) {
        self.shared_data.borrow_mut().top_enter_called += 1;
    }

    fn handle_state_exit(&self) {
        self.shared_data.borrow_mut().top_exit_called += 1;
    }
}
