use std::{cell::RefCell, rc::Rc};

use rust_hsm::{
    events::StateEventsIF, state::StateIF, state_builder::StateBuilder,
    state_data_delegate::StateDelegateRef,
};

use crate::{light_events::LightEvents, light_states::LightStates};

pub struct LightStateTop {
    state_data: StateDelegateRef,
}

impl LightStateTop {
    pub fn new() -> Rc<RefCell<Self>> {
        let state_builder =
            StateBuilder::new(LightStates::TOP as u16, "LightStateTop".to_string(), None);

        let built_state = Rc::new(RefCell::new(Self {
            state_data: state_builder.get_delegate(),
        }));

        state_builder
            .set_concrete_state(built_state.clone())
            .validate_build()
            .expect("Failed to build LightStateDimmer!");

        built_state
    }
}

impl StateIF for LightStateTop {
    fn handle_event(&mut self, event: &dyn StateEventsIF) -> bool {
        let events: LightEvents = LightEvents::from(event);
        // top returns true for all events
        match events {
            _ => true,
        }
    }

    fn get_state_data(&self) -> StateDelegateRef {
        self.state_data.clone()
    }
}
