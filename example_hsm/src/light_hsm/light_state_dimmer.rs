use std::process::Output;

use rust_hsm::{state::StateIF, state_engine_channel_delegate::StateEngineDelegate};

use crate::{
    light_events::LightEvents,
    light_hsm_data::{LightAdjustment, LightHsmDataRef},
    light_states::LightStates,
};

pub(crate) struct LightStateDimmer {
    delegate: StateEngineDelegate<LightStates, LightEvents>,
    shared_data: LightHsmDataRef,
}

impl LightStateDimmer {
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

    // async fn set_to_percentage(&mut self, percentage: u8) -> bool {
    //     let event_res = if percentage == 0 {
    //         match self.delegate.dispatch_event_internally(LightEvents::TurnOff, true){
    //             Ok(x) => {x.unwrap().await; true},
    //             Err(_) => false
    //         }
    //     } else if percentage >= 100 {
    //         match self.delegate.dispatch_event_internally(LightEvents::TurnOn, true) {
    //             Ok(x) => {x.unwrap().await; true},
    //             Err(_) => false
    //         }
    //     } else {
    //         true
    //     };

    //     self.shared_data.write().unwrap().set_lighting(percentage) && event_res
    // }

    fn set_to_percentage(&mut self, percentage: u8) {
        if percentage == 0 {
            self.delegate
                .async_dispatch_event_internally(LightEvents::TurnOff);
        } else if percentage >= 100 {
            self.delegate
                .async_dispatch_event_internally(LightEvents::TurnOn);
        };

        self.shared_data.write().unwrap().set_lighting(percentage);
    }

    fn set_relative(&mut self, action: LightAdjustment, relative_percentage: u8) -> bool {
        self.shared_data
            .write()
            .unwrap()
            .adjust_lighting_by_percentage(relative_percentage, action);
        true
    }
}

impl StateIF<LightStates, LightEvents> for LightStateDimmer {
    fn handle_event(&mut self, event: &LightEvents) -> bool {
        // top returns true for all events
        match event {
            LightEvents::Set(percentage) => {
                self.set_to_percentage(*percentage);
                true
            },
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

    fn handle_state_start(&mut self) {
        self.shared_data.write().unwrap().dimmer_start_called += 1;
    }

    fn handle_state_enter(&mut self) {
        self.shared_data.write().unwrap().dimmer_enter_called += 1;
    }

    fn handle_state_exit(&mut self) {
        self.shared_data.write().unwrap().dimmer_exit_called += 1;
    }
}
