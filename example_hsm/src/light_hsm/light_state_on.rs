use rust_hsm::{state::StateIF, state_engine_delegate::EngineDelegate};

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};

pub(crate) struct LightStateOn {
    delegate: EngineDelegate<LightStates, LightEvents>,
    shared_data: LightHsmDataRef,
}

impl LightStateOn {
    pub fn new(
        shared_data: LightHsmDataRef,
        delegate: EngineDelegate<LightStates, LightEvents>,
    ) -> Box<Self> {
        Box::new(Self {
            delegate,
            shared_data,
        })
    }

    fn handle_toggle(&self) -> bool {
        self.handle_turn_off()
    }

    fn handle_turn_off(&self) -> bool {
        match self.delegate.change_state(LightStates::OFF as u16) {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

impl StateIF<LightStates, LightEvents> for LightStateOn {
    fn handle_event(&self, event: &LightEvents) -> bool {
        // top returns true for all events
        match event {
            LightEvents::Toggle => self.handle_toggle(),
            LightEvents::TurnOff => self.handle_turn_off(),
            _ => false,
        }
    }

    /// Do not use enter here. What if we are just traversing through the state?
    /// We could get 0 -> 100 -> 50. That is a bad UX!
    /// Only set it to 100 if we are staying here
    fn handle_state_start(&self) {
        self.shared_data.borrow_mut().on_start_called += 1;
        self.shared_data.borrow_mut().turn_on();
    }

    fn handle_state_enter(&self) {
        self.shared_data.borrow_mut().on_enter_called += 1;
    }

    fn handle_state_exit(&self) {
        self.shared_data.borrow_mut().on_exit_called += 1;
    }
}
