use std::{cell::RefCell, rc::Rc};

use rust_hsm::{
    events::StateEventsIF,
    state::{ComposableStateData, StateChainOfResponsibility},
};

use crate::{light_events::LightEvents, light_states::LightStates};

pub struct LightStateTop {
    state_data: ComposableStateData,
}

impl LightStateTop {
    pub fn new() -> Rc<RefCell<Self>> {
        let data =
            ComposableStateData::new(LightStates::TOP as u16, "LightStateTop".to_string(), None);

        Rc::new(RefCell::new(Self { state_data: data }))
    }
}

impl StateChainOfResponsibility for LightStateTop {
    fn handle_event(&mut self, event: &dyn StateEventsIF) -> bool {
        let events: LightEvents = LightEvents::from(event);
        // top returns true for all events
        match events {
            _ => true,
        }
    }

    fn get_state_data(&self) -> &ComposableStateData {
        &self.state_data
    }

    fn get_state_data_mut(&mut self) -> &mut ComposableStateData {
        &mut self.state_data
    }
}
