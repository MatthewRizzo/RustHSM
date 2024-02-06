use std::{cell::RefCell, rc::Rc};

use rust_hsm::{
    state::StateRef, state_controller::DecoratableHSMControllerBase,
    state_controller_trait::HsmControllerRef,
};

use crate::light_hsm::{
    light_events::LightEvents,
    light_hsm_data::{LightHsmData, LightHsmDataRef},
    light_state_dimmer::LightStateDimmer,
    light_state_off::LightStateOff,
    light_state_on::LightStateOn,
    light_state_top::LightStateTop,
};

pub struct LightHsmController {
    hsm: HsmControllerRef,
    _shared_data: LightHsmDataRef,
}

impl LightHsmController {
    pub fn new() -> Rc<RefCell<Self>> {
        let hsm: HsmControllerRef = DecoratableHSMControllerBase::new();

        // Start the light "off"
        let shared_data = LightHsmData::new(0);

        let light_hsm = Rc::new(RefCell::new(LightHsmController {
            hsm,
            _shared_data: shared_data.clone(),
        }));

        let top_state = LightStateTop::new(light_hsm.borrow().get_hsm());

        let state_on = LightStateOn::new(
            top_state.clone(),
            light_hsm.borrow().get_hsm(),
            shared_data.clone(),
        );

        let state_off = LightStateOff::new(
            top_state.clone(),
            light_hsm.borrow().get_hsm(),
            shared_data.clone(),
        );

        // on is parent of dimmer - dimmer inherits most of on's behavior,
        // BUT has some extra behavior special to itself
        let state_dimmer =
            LightStateDimmer::new(state_on.clone(), light_hsm.borrow().get_hsm(), shared_data);

        light_hsm.borrow_mut().add_state(top_state);
        light_hsm.borrow_mut().add_state(state_on);
        light_hsm.borrow_mut().add_state(state_off.clone());
        light_hsm.borrow_mut().add_state(state_dimmer);

        // Start with the lights off
        light_hsm.borrow_mut().init(state_off.clone());

        light_hsm
    }

    pub fn get_hsm(&self) -> HsmControllerRef {
        self.hsm.clone()
    }

    pub fn add_state(&mut self, new_state: StateRef) {
        self.hsm.borrow_mut().add_state(new_state)
    }

    /// Note: exposing init like this is discouraged, but can be done!
    pub fn init(&mut self, starting_state: StateRef) {
        let _ = self.hsm.borrow_mut().init(starting_state);
    }

    /// Note: exposing the current state is ALSO a really bad idea.
    pub fn get_current_state(&self) -> StateRef {
        self.hsm.borrow().get_current_state().clone()
    }

    pub fn dispatch_into_hsm(&mut self, event: LightEvents) {
        let event_base = event.to_event_base();
        self.hsm
            .borrow_mut()
            .external_dispatch_into_hsm(&event_base);
    }
}
