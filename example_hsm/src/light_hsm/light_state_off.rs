use rust_hsm::{state::StateIF, state_engine_delegate::EngineDelegate};

use crate::{
    light_events::LightEvents, light_hsm_data::LightHsmDataRef, light_states::LightStates,
};

pub(crate) struct LightStateOff {
    delegate: EngineDelegate<LightStates, LightEvents>,
    shared_data: LightHsmDataRef,
}

impl LightStateOff {
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
        self.handle_turn_on()
    }

    fn handle_turn_on(&self) -> bool {
        match self.delegate.change_state(LightStates::ON as u16) {
            Ok(()) => true,
            Err(_) => false,
        }
    }
}

impl StateIF<LightStates, LightEvents> for LightStateOff {
    fn handle_event(&self, event: &LightEvents) -> bool {
        match event {
            LightEvents::Toggle => self.handle_toggle(),
            LightEvents::TurnOn => self.handle_turn_on(),
            _ => false,
        }
    }

    fn handle_state_start(&self) {
        self.shared_data.borrow_mut().turn_off();
        self.shared_data.borrow_mut().off_start_called += 1;
    }

    fn handle_state_enter(&self) {
        self.shared_data.borrow_mut().off_enter_called += 1;
    }

    fn handle_state_exit(&self) {
        self.shared_data.borrow_mut().off_exit_called += 1;
    }
}
