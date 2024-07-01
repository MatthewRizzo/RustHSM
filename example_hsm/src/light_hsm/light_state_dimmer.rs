use rust_hsm::{
    events::StateEventsIF,
    state::{StateIF, StateRef},
    state_builder::StateBuilder,
    state_data_delegate::StateDelegateRef,
};

use crate::{
    light_events::LightEvents,
    light_hsm_data::{LightAdjustment, LightHsmDataRef},
    light_states::LightStates,
};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct LightStateDimmer {
    state_data: StateDelegateRef,
    shared_data: LightHsmDataRef,
}

impl LightStateDimmer {
    pub fn new(parent_state: StateRef, shared_data: LightHsmDataRef) -> Rc<RefCell<Self>> {
        let state_builder = StateBuilder::new(
            LightStates::DIMMER as u16,
            "LightStateDimmer".to_string(),
            Some(parent_state.borrow().get_state_data()),
        );

        let built_state = Rc::new(RefCell::new(Self {
            state_data: state_builder.get_delegate(),
            shared_data,
        }));

        state_builder
            .set_concrete_state(built_state.clone())
            .validate_build()
            .expect("Failed to build LightStateDimmer!");

        built_state
    }

    fn set_to_percentage(&mut self, percentage: u8) -> bool {
        let event_res = if percentage == 0 {
            self.state_data
                .borrow_mut()
                .dispatch_event_internally(Rc::new(LightEvents::TurnOff))
        } else if percentage >= 100 {
            self.state_data
                .borrow_mut()
                .dispatch_event_internally(Rc::new(LightEvents::TurnOn))
        } else {
            Ok(())
        }
        .map_or_else(|_| false, |_| true);

        self.shared_data.borrow_mut().set_lighting(percentage) && event_res
    }

    fn set_relative(&mut self, action: LightAdjustment, relative_percentage: u8) -> bool {
        self.shared_data
            .borrow_mut()
            .adjust_lighting_by_percentage(relative_percentage, action);
        true
    }
}

impl StateIF for LightStateDimmer {
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
            // TurnOff and Toggle are implemented by our parent (LightStateOn)
            // Leverage that behavior by not handling them
            _ => false,
        }
    }

    fn get_state_data(&self) -> StateDelegateRef {
        self.state_data.clone()
    }
}
