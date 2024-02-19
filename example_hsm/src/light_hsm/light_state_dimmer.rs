use rust_hsm::{
    events::StateEventsIF,
    state::{ComposableStateData, StateChainOfResponsibility, StateRef},
    state_controller_trait::HsmControllerRef,
};

use crate::{
    light_events::LightEvents,
    light_hsm_data::{LightAdjustment, LightHsmDataRef},
    light_states::LightStates,
};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct LightStateDimmer {
    state_data: ComposableStateData,
    shared_data: LightHsmDataRef,
}

impl LightStateDimmer {
    pub fn new(
        parent_state: StateRef,
        hsm: HsmControllerRef,
        shared_data: LightHsmDataRef,
    ) -> Rc<RefCell<Self>> {
        let state_data = ComposableStateData::new(
            LightStates::DIMMER as u16,
            "LightStateDimmer".to_string(),
            Some(parent_state),
            hsm,
        );

        Rc::new(RefCell::new(Self {
            state_data,
            shared_data,
        }))
    }

    fn set_to_percentage(&mut self, percentage: u8) -> bool {
        self.shared_data.borrow_mut().set_lighting(percentage)
    }

    fn set_relative(&mut self, action: LightAdjustment, relative_percentage: u8) -> bool {
        self.shared_data
            .borrow_mut()
            .adjust_lighting_by_percentage(relative_percentage, action);
        true
    }
}

impl StateChainOfResponsibility for LightStateDimmer {
    fn handle_event(&mut self, event_id: &dyn StateEventsIF) -> bool {
        let events: LightEvents = LightEvents::from(event_id);
        // top returns true for all events
        match events {
            LightEvents::Set(percentage) => self.set_to_percentage(percentage),
            LightEvents::ReduceByPercent(percentage) => {
                self.set_relative(LightAdjustment::Decrease, percentage)
            }
            LightEvents::IncreaseByPercent(percentage) => {
                self.set_relative(LightAdjustment::Increase, percentage)
            }
            _ => false,
        }
    }

    fn get_state_data(&self) -> &ComposableStateData {
        &self.state_data
    }
}
