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

pub(crate) struct LightStateOff {
    state_data: StateDelegateRef,
    shared_data: LightHsmDataRef,
}

impl LightStateOff {
    pub fn new(parent_state: StateRef, shared_data: LightHsmDataRef) -> Rc<RefCell<Self>> {
        let state_builder = StateBuilder::new(
            LightStates::OFF as u16,
            "LightStateOff".to_string(),
            Some(parent_state.borrow().get_state_data()),
        );

        let built_state = Rc::new(RefCell::new(Self {
            state_data: state_builder.get_delegate(),
            shared_data,
        }));

        state_builder
            .set_concrete_state(built_state.clone())
            .validate_build()
            .expect("Failed to build LightStateOff!");

        built_state
    }

    fn handle_toggle(&mut self) -> bool {
        self.handle_turn_on()
    }

    fn handle_turn_on(&mut self) -> bool {
        match self
            .state_data
            .borrow_mut()
            .submit_state_change_request(LightStates::ON as u16)
        {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

impl StateIF for LightStateOff {
    fn handle_event(&mut self, event: &dyn StateEventsIF) -> bool {
        let events: LightEvents = LightEvents::from(event);
        // top returns true for all events
        match events {
            LightEvents::Toggle => self.handle_toggle(),
            LightEvents::TurnOn => self.handle_turn_on(),
            _ => false,
        }
    }

    fn handle_state_start(&mut self) {
        self.shared_data.borrow_mut().turn_off();
    }

    fn get_state_data(&self) -> StateDelegateRef {
        self.state_data.clone()
    }
}
