use rust_hsm::{events::StateEventsIF, state::StateIF};

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

impl StateIF<LightStates> for LightStateTop {
    fn handle_event(&mut self, event: &dyn StateEventsIF) -> bool {
        let events: LightEvents = LightEvents::from(event);
        // top returns true for all events
        match events {
            _ => true,
        }
    }
    fn handle_state_start(&mut self) {
        self.shared_data.borrow_mut().top_start_called += 1;
    }

    fn handle_state_enter(&mut self) {
        self.shared_data.borrow_mut().top_enter_called += 1;
    }

    fn handle_state_exit(&mut self) {
        self.shared_data.borrow_mut().top_exit_called += 1;
    }
}
