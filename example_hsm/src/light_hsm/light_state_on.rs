use rust_hsm::{
    events::StateEventsIF,
    state::{ComposableStateData, StateChainOfResponsibility, StateRef},
    state_controller_trait::HsmControllerRef,
};

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct LightStateOn {
    state_data: ComposableStateData,
    shared_data: LightHsmDataRef,
}

impl LightStateOn {
    pub fn new(
        parent_state: StateRef,
        hsm: HsmControllerRef,
        shared_data: LightHsmDataRef,
    ) -> Rc<RefCell<Self>> {
        let state_data = ComposableStateData::new(
            LightStates::ON as u16,
            "LightStateOn".to_string(),
            Some(parent_state),
            hsm,
        );

        Rc::new(RefCell::new(Self {
            state_data,
            shared_data,
        }))
    }

    fn handle_toggle(&mut self) -> bool {
        self.handle_turn_off()
    }

    fn handle_turn_off(&mut self) -> bool {
        // TODO - make a macro that takes state data and calls change state for you!
        self.state_data
            .submit_state_change_request(LightStates::OFF as u16);
        true
    }
}

impl StateChainOfResponsibility for LightStateOn {
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

    fn get_state_data(&self) -> &ComposableStateData {
        &self.state_data
    }
}
