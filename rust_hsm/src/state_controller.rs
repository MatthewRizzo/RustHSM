use std::{cell::RefCell, rc::Rc};

///! This file contains the logic for a state engine comprised of many
///! composable states
use crate::{
    events::StateEventsIF,
    state::{StateRef, StatesRefVec},
    state_controller_trait::{HsmController, HsmControllerRef},
};

/// Compose / decorate your hsm controller with this
pub struct HSMControllerBase {
    hsm_name: String,
    /// We own the vector of states, but the states themselves are owned by others
    states: StatesRefVec,
    /// Only ever optional before init
    current_state: Option<StateRef>,
    /// Used to cache the current known sequence of events
    state_change_string: String,
}

impl HSMControllerBase {
    pub fn new(hsm_name: String) -> HsmControllerRef {
        Rc::new(RefCell::new(HSMControllerBase {
            hsm_name,
            states: vec![],
            current_state: None,
            state_change_string: String::new(),
        }))
    }
}

impl HsmController for HSMControllerBase {
    fn add_state(&mut self, new_state: StateRef) {
        self.states.push(new_state.clone());
    }

    fn external_dispatch_into_hsm(&mut self, event: &dyn StateEventsIF) {
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

    fn get_states(&self) -> StatesRefVec {
        self.states.clone()
    }

    fn get_state_change_string(&mut self) -> &mut String {
        &mut self.state_change_string
    }

    fn get_hsm_name(&self) -> String {
        self.hsm_name.clone()
    }
}
