use rust_hsm::{
    events::StateEventsIF,
    state::{ComposableStateData, StateChainOfResponsibility, StateRef},
};

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct LightStateOff {
    state_data: ComposableStateData,
    shared_data: LightHsmDataRef,
}

impl LightStateOff {
    pub fn new(parent_state: StateRef, shared_data: LightHsmDataRef) -> Rc<RefCell<Self>> {
        let state_data = ComposableStateData::new(
            LightStates::OFF as u16,
            "LightStateOff".to_string(),
            Some(parent_state),
        );

        Rc::new(RefCell::new(Self {
            state_data,
            shared_data,
        }))
    }

    fn handle_toggle(&mut self) -> bool {
        self.handle_turn_on()
    }

    fn handle_turn_on(&mut self) -> bool {
        self.state_data
            .submit_state_change_request(LightStates::ON as u16);
        true
    }
}

impl StateChainOfResponsibility for LightStateOff {
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

    fn get_state_data(&self) -> &ComposableStateData {
        &self.state_data
    }

    fn get_state_data_mut(&mut self) -> &mut ComposableStateData {
        &mut self.state_data
    }
}
