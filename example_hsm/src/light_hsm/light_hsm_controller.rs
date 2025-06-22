use rust_hsm::{errors::HSMResult, state_engine::HSMEngine};

use crate::light_hsm::{
    light_events::LightEvents,
    light_hsm_data::{LightHsmData, LightHsmDataRef},
    light_state_dimmer::LightStateDimmer,
    light_state_off::LightStateOff,
    light_state_on::LightStateOn,
    light_state_top::LightStateTop,
    light_states::LightStates,
};

use std::rc::Rc;

pub struct LightControllerHsm {
    engine: Rc<HSMEngine<LightStates, LightEvents>>,
    /// Again...leaking this is a bad idea. It is only done here for testing/asserting
    /// Do NOT do this in a real HSM
    pub(crate) _shared_data: LightHsmDataRef,
}

impl LightControllerHsm {
    pub fn new(engine_log_level: log::LevelFilter) -> Self {
        let shared_data = LightHsmData::new(0);

        let engine = HSMEngine::new("LightControllerHsm".to_string(), engine_log_level).unwrap();

        let top_state = LightStateTop::new(shared_data.clone());

        // dimmer leverage's similar behavior to on in most cases!
        // the non-shared behavior they impl for themselves!
        // Hence on is dimmer's parent.
        let state_on = LightStateOn::new(shared_data.clone(), HSMEngine::get_delegate(&engine));
        let state_dimmer =
            LightStateDimmer::new(shared_data.clone(), HSMEngine::get_delegate(&engine));
        let state_off = LightStateOff::new(shared_data.clone(), HSMEngine::get_delegate(&engine));

        engine.add_state(top_state, LightStates::Top, None).unwrap();
        engine
            .add_state(state_on, LightStates::ON, Some(LightStates::Top))
            .unwrap();
        engine
            .add_state(state_off, LightStates::OFF, Some(LightStates::Top))
            .unwrap();
        engine
            .add_state(state_dimmer, LightStates::DIMMER, Some(LightStates::ON))
            .unwrap();
        engine.init(LightStates::DIMMER as u16).unwrap();

        LightControllerHsm {
            engine,
            _shared_data: shared_data.clone(),
        }
    }

    /// Note: exposing the current state is ALSO a really bad idea.
    pub(crate) fn get_current_state(&self) -> LightStates {
        // In a real system you would want to translate from HSMResult -> your result
        self.engine.get_current_state().unwrap()
    }

    pub(crate) fn dispatch_into_hsm(&self, event: LightEvents) -> HSMResult<(), LightStates> {
        // In a real system you would want to translate from HSMResult -> your result
        self.engine.dispatch_event(event)
    }

    /// In a real HSM this is a BAD idea. DO NOT LEAK the data
    /// Only doing it here as an example and so some asserts can be done
    pub fn get_light_data(&self) -> LightHsmDataRef {
        self._shared_data.clone()
    }
}
