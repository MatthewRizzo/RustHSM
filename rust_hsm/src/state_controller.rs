use std::{cell::RefCell, rc::Rc};

///! This file contains the logic for a state engine comprised of many
///! composable states
use crate::{
    events::HsmEvent,
    state::{StateId, StateRef, StatesRefVec},
    state_controller_trait::{HsmController, HsmControllerRef},
};

/// Compose / decorate your hsm controller with this
pub struct DecoratableHSMControllerBase {
    /// We own the vector of states, but the states themselves are owned by others
    states: StatesRefVec,
    /// Only ever optional before init
    current_state: Option<StateRef>,
    /// Only set during handle_event if there is a change_state
    requested_new_state: Option<StateId>,
    /// Used to cache the current known sequence of events
    state_change_string: String,
}

impl DecoratableHSMControllerBase {
    pub fn new() -> HsmControllerRef {
        Rc::new(RefCell::new(DecoratableHSMControllerBase {
            states: vec![],
            current_state: None,
            requested_new_state: None,
            state_change_string: String::new(),
        }))
    }
}

impl HsmController for DecoratableHSMControllerBase {
    fn add_state(&mut self, new_state: StateRef) {
        self.states.push(new_state.clone());
    }

    fn external_dispatch_into_hsm(&mut self, event: &HsmEvent) {
        // Override for a more custom implementation
        self.handle_event(event)
    }

    fn get_current_state(&self) -> StateRef {
        if self.current_state.is_none() {
            assert!(
                false,
                "HSM not initialized! Make sure to call Init before using state-related API's!"
            )
        }
        self.current_state
            .clone()
            .expect("HSM not initialized! Make sure to call Init before using state-related API's!")
            .clone()
    }

    fn set_current_state(&mut self, new_current_state: StateRef) {
        self.current_state = Some(new_current_state)
    }

    fn get_requested_new_state(&self) -> Option<StateId> {
        self.requested_new_state.clone()
    }

    fn set_requested_new_state(&mut self, requested_new_state: StateId) {
        self.requested_new_state = Some(requested_new_state)
    }

    fn get_states(&self) -> StatesRefVec {
        self.states.clone()
    }

    fn get_state_change_string(&mut self) -> &mut String {
        &mut self.state_change_string
    }

    fn clear_requested_new_state(&mut self) {
        self.requested_new_state = None;
    }
}
