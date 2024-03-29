use rust_hsm::{
    state::StateRef, state_controller::{HSMControllerBase, HsmControllerBuilder},
    state_controller_trait::HsmController,
};

use crate::light_hsm::{
    light_events::LightEvents,
    light_hsm_data::{LightHsmData, LightHsmDataRef},
    light_state_dimmer::LightStateDimmer,
    light_state_off::LightStateOff,
    light_state_on::LightStateOn,
    light_state_top::LightStateTop,
};

pub struct LightControllerHsm {
    hsm: HSMControllerBase,
    /// Again...leaking this is a bad idea. It is only done here for testing/asserting
    pub(crate) _shared_data: LightHsmDataRef,
}

impl LightControllerHsm {
    pub fn new() -> Self {
        let shared_data = LightHsmData::new(0);

        let top_state = LightStateTop::new();

        // Both on and off leverage similar behavior to dimmer in most cases!
        // the non-shared behavior they impl for themselves!
        // Hence dimmer is their parent.
        let state_dimmer = LightStateDimmer::new(top_state.clone(), shared_data.clone());
        let state_on = LightStateOn::new(state_dimmer.clone(), shared_data.clone());
        let state_off = LightStateOff::new(state_dimmer.clone(), shared_data.clone());

        let hsm = HsmControllerBuilder::new("LightControllerHsm".to_string())
            .add_state(top_state)
            .add_state(state_on)
            .add_state(state_off.clone())
            .add_state(state_dimmer)
            // Start the light "off"
            .init(state_off)
            .unwrap();

        let light_hsm = LightControllerHsm {
            hsm,
            _shared_data: shared_data.clone(),
        };

        light_hsm
    }

    /// Note: exposing the current state is ALSO a really bad idea.
    pub fn get_current_state(&self) -> StateRef {
        self.hsm.get_current_state()
    }

    pub fn dispatch_into_hsm(&mut self, event: LightEvents) {
        self.hsm.external_dispatch_into_hsm(&event)
    }

    /// In a real HSM this is a BAD idea. DO NOT LEAK the data
    /// Only doing it here as an example and so some asserts can be done
    pub fn get_light_data(&self) -> LightHsmDataRef {
        self._shared_data.clone()
    }
}
