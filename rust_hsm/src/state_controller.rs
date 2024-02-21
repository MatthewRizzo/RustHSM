///! This file contains the logic for a state engine comprised of many
///! composable states
use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventsIF,
    state::{StateRef, StatesRefVec},
    state_controller_trait::HsmController,
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
    /// Create an HSM controller.
    // Highly recommend NOT exposing the HSM beyond your container.
    pub(crate) fn new(hsm_name: String) -> HSMControllerBase {
        HSMControllerBase {
            hsm_name,
            states: vec![],
            current_state: None,
            state_change_string: String::new(),
        }
    }
}

impl HsmController for HSMControllerBase {
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

/// Struct encapsulating the process of building an HsmController.
/// Enforces immutability of the controller as states are added.
pub struct HsmControllerBuilder {
    controller_under_construction: HSMControllerBase,
}

impl HsmControllerBuilder {
    pub fn new(hsm_name: String) -> HsmControllerBuilder {
        // let controller = HSMControllerBase {
        //     hsm_name,
        //     states: vec![],
        //     current_state: None,
        //     state_change_string: String::new(),
        // };
        let controller = HSMControllerBase::new(hsm_name);

        HsmControllerBuilder {
            controller_under_construction: controller,
        }
    }

    pub fn add_state(mut self, new_state: StateRef) -> Self {
        self.controller_under_construction
            .states
            .push(new_state.clone());
        self
    }

    /// Final step in process
    pub fn init(mut self, initial_state: StateRef) -> HSMResult<HSMControllerBase> {
        let initial_state_id = initial_state.borrow().get_state_id();
        let states = self.controller_under_construction.get_states();
        if *initial_state_id.get_id() as usize >= states.len() {
            return Err(HSMError::InvalidStateId(format!(
                "Initial State with Id {} is not valid. There are only {} states!",
                *initial_state_id.get_id(),
                states.len() - 1
            )));
        }

        self.controller_under_construction
            .set_current_state(initial_state);
        Ok(self.controller_under_construction)
    }
}
