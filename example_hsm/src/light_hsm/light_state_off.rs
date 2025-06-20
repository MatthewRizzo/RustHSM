use rust_hsm::{state::StateIF, state_engine_channel_delegate::StateEngineDelegate};

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};

pub(crate) struct LightStateOff {
    delegate: StateEngineDelegate<LightStates, LightEvents>,
    shared_data: LightHsmDataRef,
}

impl LightStateOff {
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
        self.handle_turn_on()
    }

    fn handle_turn_on(&mut self) -> bool {
        match self.delegate.change_state(LightStates::ON as u16) {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

impl StateIF<LightStates, LightEvents> for LightStateOff {
    fn handle_event(&mut self, event: &LightEvents) -> bool {
        match event {
            LightEvents::Toggle => self.handle_toggle(),
            LightEvents::TurnOn => self.handle_turn_on(),
            _ => false,
        }
    }

    fn handle_state_start(&mut self) {
        self.shared_data.borrow_mut().turn_off();
        self.shared_data.borrow_mut().off_start_called += 1;
    }

    fn handle_state_enter(&mut self) {
        self.shared_data.borrow_mut().off_enter_called += 1;
    }

    fn handle_state_exit(&mut self) {
        self.shared_data.borrow_mut().off_exit_called += 1;
    }
}
