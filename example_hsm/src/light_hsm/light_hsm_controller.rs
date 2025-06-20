use rust_hsm::{
    errors::HSMResult,
    state_engine::{HSMEngine, HSMEngineBuilder},
};

use crate::light_hsm::{
    light_events::LightEvents,
    light_hsm_data::{LightHsmData, LightHsmDataRef},
    light_state_dimmer::LightStateDimmer,
    light_state_off::LightStateOff,
    light_state_on::LightStateOn,
    light_state_top::LightStateTop,
    light_states::LightStates,
};

use log;

pub struct LightControllerHsm {
    hsm: HSMEngine<LightStates, LightEvents>,
    /// Again...leaking this is a bad idea. It is only done here for testing/asserting
    /// Do NOT do this in a real HSM
    pub(crate) _shared_data: LightHsmDataRef,
}

impl LightControllerHsm {
    pub fn new(engine_log_level: log::LevelFilter) -> Self {
        let shared_data = LightHsmData::new(0);

        let engine_builder = HSMEngineBuilder::new(
            "LightControllerHsm".to_string(),
            LightStates::Top as u16,
            log::LevelFilter::Info,
            engine_log_level,
        );

        #[allow(unused)] // Not every state needs a delegate! But you can acquire it!
        let top_delegate = engine_builder
            .create_delegate(LightStates::Top.into())
            .expect("");
        let on_delegate = engine_builder
            .create_delegate(LightStates::ON.into())
            .expect("");
        let off_delegate = engine_builder
            .create_delegate(LightStates::OFF.into())
            .expect("");
        let dimmer_delegate = engine_builder
            .create_delegate(LightStates::DIMMER.into())
            .expect("");
        assert!(
            engine_builder
                .create_delegate(LightStates::ON.into())
                .is_err(),
            "Requesting the same delegate twice is illegal!"
        );

        let top_state = LightStateTop::new(shared_data.clone());

        // dimmer leverage's similar behavior to on in most cases!
        // the non-shared behavior they impl for themselves!
        // Hence on is dimmer's parent.
        let state_on = LightStateOn::new(shared_data.clone(), on_delegate);
        let state_dimmer = LightStateDimmer::new(shared_data.clone(), off_delegate);
        let state_off = LightStateOff::new(shared_data.clone(), dimmer_delegate);

        let hsm = engine_builder
            .add_state(top_state, LightStates::Top, None)
            .add_state(state_on, LightStates::ON, Some(LightStates::Top))
            .add_state(state_off, LightStates::OFF, Some(LightStates::Top))
            .add_state(state_dimmer, LightStates::DIMMER, Some(LightStates::ON))
            .init(LightStates::DIMMER as u16)
            .unwrap();

        let light_hsm = LightControllerHsm {
            hsm,
            _shared_data: shared_data.clone(),
        };

        light_hsm
    }

    /// Note: exposing the current state is ALSO a really bad idea.
    pub(crate) fn get_current_state(&self) -> LightStates {
        // In a real system you would want to translate from HSMResult -> your result
        self.hsm
            .get_current_state()
            .expect("Called before the HSM was initialized!")
    }

    pub(crate) fn dispatch_into_hsm(&mut self, event: LightEvents) -> HSMResult<(), LightStates> {
        // In a real system you would want to translate from HSMResult -> your result
        self.hsm.dispatch_event(event)
    }

    /// In a real HSM this is a BAD idea. DO NOT LEAK the data
    /// Only doing it here as an example and so some asserts can be done
    pub fn get_light_data(&self) -> LightHsmDataRef {
        self._shared_data.clone()
    }
}
