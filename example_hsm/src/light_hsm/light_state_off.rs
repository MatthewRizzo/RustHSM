use rust_hsm::{
    events::HsmEvent,
    state::{ComposableStateData, StateChainOfResponsibility, StateRef},
    state_controller_trait::HsmControllerRef,
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
    pub fn new(
        parent_state: StateRef,
        hsm: HsmControllerRef,
        shared_data: LightHsmDataRef,
    ) -> Rc<RefCell<Self>> {
        let state_data = ComposableStateData::new(
            LightStates::OFF as u16,
            "LightStateOff".to_string(),
            Some(parent_state),
            hsm,
        );

        Rc::new(RefCell::new(Self {
            state_data,
            shared_data,
        }))
    }

    fn handle_toggle(&self) -> bool {
        self.handle_turn_on()
    }

    fn handle_turn_on(&self) -> bool {
        // TODO - make a macro that takes state data and calls change state for you!
        self.state_data
            .get_hsm()
            .borrow_mut()
            .change_state(LightStates::ON as u16);
        true
    }
}

impl StateChainOfResponsibility for LightStateOff {
    fn handle_event(&mut self, event_id: &HsmEvent) -> bool {
        let events: LightEvents = LightEvents::from(event_id);
        // top returns true for all events
        match events {
            LightEvents::Toggle => self.handle_toggle(),
            LightEvents::TurnOff => self.handle_turn_on(),
            _ => false,
        }
    }

    fn handle_state_start(&mut self) {
        self.shared_data.borrow_mut().turn_off();
    }

    fn get_state_data(&self) -> &ComposableStateData {
        &self.state_data
    }
}
