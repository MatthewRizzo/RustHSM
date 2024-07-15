use rust_hsm::{
    events::StateEventsIF, state::StateIF, state_engine_channel_delegate::StateEngineDelegate,
};

use crate::{
    light_events::LightEvents,
    light_hsm_data::{LightAdjustment, LightHsmDataRef},
    light_states::LightStates,
};

pub(crate) struct LightStateDimmer {
    delegate: StateEngineDelegate<LightStates>,
    shared_data: LightHsmDataRef,
}

impl LightStateDimmer {
    pub fn new(
        shared_data: LightHsmDataRef,
        delegate: StateEngineDelegate<LightStates>,
    ) -> Box<Self> {
        let built_state = Box::new(Self {
            delegate,
            shared_data,
        });

        built_state
    }

    fn set_to_percentage(&mut self, percentage: u8) -> bool {
        let event_res: bool = if percentage == 0 {
            self.delegate
                .dispatch_event_internally(Box::new(LightEvents::TurnOff))
        } else if percentage >= 100 {
            self.delegate
                .dispatch_event_internally(Box::new(LightEvents::TurnOn))
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

impl StateIF<LightStates> for LightStateDimmer {
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

    fn handle_state_start(&mut self) {
        self.shared_data.borrow_mut().dimmer_start_called += 1;
    }

    fn handle_state_enter(&mut self) {
        self.shared_data.borrow_mut().dimmer_enter_called += 1;
    }

    fn handle_state_exit(&mut self) {
        self.shared_data.borrow_mut().dimmer_exit_called += 1;
    }
}
