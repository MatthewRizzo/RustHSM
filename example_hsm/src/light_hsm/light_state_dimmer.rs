use rust_hsm::{state::StateIF, state_engine_delegate::EngineDelegate};

use crate::{
    light_events::LightEvents,
    light_hsm_data::{LightAdjustment, LightHsmDataRef},
    light_states::LightStates,
};

pub(crate) struct LightStateDimmer {
    delegate: EngineDelegate<LightStates, LightEvents>,
    shared_data: LightHsmDataRef,
}

impl LightStateDimmer {
    pub fn new(
        shared_data: LightHsmDataRef,
        delegate: EngineDelegate<LightStates, LightEvents>,
    ) -> Box<Self> {
        Box::new(Self {
            delegate,
            shared_data,
        })
    }

    fn set_to_percentage(&self, percentage: u8) -> bool {
        let event_res: bool = if percentage == 0 {
            self.delegate.internal_handle_event(LightEvents::TurnOff)
        } else if percentage >= 100 {
            self.delegate.internal_handle_event(LightEvents::TurnOn)
        } else {
            Ok(())
        }
        .map_or_else(|_| false, |_| true);

        self.shared_data.borrow_mut().set_lighting(percentage) && event_res
    }

    fn set_relative(&self, action: LightAdjustment, relative_percentage: u8) -> bool {
        self.shared_data
            .borrow_mut()
            .adjust_lighting_by_percentage(relative_percentage, action);
        true
    }
}

impl StateIF<LightStates, LightEvents> for LightStateDimmer {
    fn handle_event(&self, event: &LightEvents) -> bool {
        // top returns true for all events
        match event {
            LightEvents::Set(percentage) => self.set_to_percentage(*percentage),
            LightEvents::ReduceByPercent(percentage) => {
                self.set_relative(LightAdjustment::Decrease, *percentage)
            }
            LightEvents::IncreaseByPercent(percentage) => {
                self.set_relative(LightAdjustment::Increase, *percentage)
            }
            // TurnOff and Toggle are implemented by our parent (LightStateOn)
            // Leverage that behavior by not handling them
            _ => false,
        }
    }

    fn handle_state_start(&self) {
        self.shared_data.borrow_mut().dimmer_start_called += 1;
    }

    fn handle_state_enter(&self) {
        self.shared_data.borrow_mut().dimmer_enter_called += 1;
    }

    fn handle_state_exit(&self) {
        self.shared_data.borrow_mut().dimmer_exit_called += 1;
    }
}
