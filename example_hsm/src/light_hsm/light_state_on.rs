use rust_hsm::{
    events::StateEventsIF,
    state::{StateIF, StateRef},
    state_builder::StateBuilder,
    state_data_delegate::StateDelegateRef,
};

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct LightStateOn {
    state_data: StateDelegateRef,
    shared_data: LightHsmDataRef,
}

impl LightStateOn {
    pub fn new(parent_state: StateRef, shared_data: LightHsmDataRef) -> Rc<RefCell<Self>> {
        let state_builder = StateBuilder::new(
            LightStates::ON as u16,
            "LightStateOn".to_string(),
            Some(parent_state.borrow().get_state_data()),
        );

        let built_state = Rc::new(RefCell::new(Self {
            state_data: state_builder.get_delegate(),
            shared_data,
        }));

        state_builder
            .set_concrete_state(built_state.clone())
            .validate_build()
            .expect("Failed to build LightStateOn!");

        built_state
    }

    fn handle_toggle(&mut self) -> bool {
        self.handle_turn_off()
    }

    fn handle_turn_off(&mut self) -> bool {
        // TODO - make a macro that takes state data and calls change state for you!
        match self
            .state_data
            .borrow_mut()
            .submit_state_change_request(LightStates::OFF as u16)
        {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

impl StateIF for LightStateOn {
    fn handle_event(&mut self, event: &dyn StateEventsIF) -> bool {
        let events: LightEvents = LightEvents::from(event);
        // top returns true for all events
        match events {
            LightEvents::Toggle => self.handle_toggle(),
            LightEvents::TurnOff => self.handle_turn_off(),
            _ => false,
        }
    }

    /// Do not use enter here. What if we are just traversing through the state?
    /// We could get 0 -> 100 -> 50. That is a bad UX!
    /// Only set it to 100 if we are staying here
    fn handle_state_start(&mut self) {
        self.shared_data.borrow_mut().turn_on();
    }

    fn get_state_data(&self) -> StateDelegateRef {
        self.state_data.clone()
    }
}
