use rust_hsm::{state::StateIF, state_engine_channel_delegate::StateEngineDelegate};

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};

pub(crate) struct LightStateOn {
    delegate: StateEngineDelegate<LightStates, LightEvents>,
    shared_data: LightHsmDataRef,
}

impl LightStateOn {
    pub fn new(
        shared_data: LightHsmDataRef,
        delegate: StateEngineDelegate<LightStates, LightEvents>,
    ) -> Box<Self> {
        let built_state = Box::new(Self {
            delegate,
            shared_data,
        });

        built_state
    }

    fn handle_toggle(&mut self) -> bool {
        self.handle_turn_off()
    }

    fn handle_turn_off(&mut self) -> bool {
        match self.delegate.change_state(LightStates::OFF as u16) {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

impl StateIF<LightStates, LightEvents> for LightStateOn {
    fn handle_event(&mut self, event: &LightEvents) -> bool {
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
    fn handle_state_start(&mut self) {
        self.shared_data.write().unwrap().on_start_called += 1;
        self.shared_data.write().unwrap().turn_on();
    }

    fn handle_state_enter(&mut self) {
        self.shared_data.write().unwrap().on_enter_called += 1;
    }

    fn handle_state_exit(&mut self) {
        self.shared_data.write().unwrap().on_exit_called += 1;
    }
}
