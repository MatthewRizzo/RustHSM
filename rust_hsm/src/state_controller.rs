///! This file contains the logic for a state engine comprised of many
///! composable states
use crate::{
    errors::{HSMError, HSMResult},
    events::{StateEventVec, StateEventsIF},
    state::{StateChainOfResponsibility, StateChainRef, StateId, StateRef, StatesVec},
    state_controller_trait::HsmController,
};

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

/// Compose / decorate your hsm controller with this
pub struct HSMControllerBase {
    hsm_name: String,
    /// We own the vector of states, but the states themselves are owned by others
    states: StatesVec,
    /// Only ever optional before init
    current_state: Option<StateId>,
    /// Used to cache the current known sequence of events
    state_change_string: String,
    follow_up_events_requested: StateEventVec,
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
            follow_up_events_requested: VecDeque::new(),
        }
    }

    pub fn get_current_state(&self) -> StateId {
        self.current_state
            .clone()
            .expect("Requested the current state before it was init by the builder!")
    }
}

impl HsmController for HSMControllerBase {
    fn dispatch_event(&mut self, event: &dyn StateEventsIF) -> HSMResult<()> {
        // Override for a more custom implementation
        self.handle_event(event)
    }

    fn get_current_state_link(&self) -> HSMResult<StateChainRef> {
        if self.current_state.is_none() {
            return Err(HSMError::ControllerNotInitialized());
        }

        let is_state = |state_link: StateChainRef| -> bool {
            state_link
                .borrow()
                .is_state(&self.current_state.clone().expect(
                "This should not be possible, we assert ControllerNotInitialized invariant above.",
            ))
        };

        let index = self.states
            .iter()
            .position(|state_link| is_state(state_link.clone()) )
            .expect("Something un-imaginably bad has happened if the current state is not a valid state!");
        let current_state_link = self.states.get(index).unwrap().clone();
        Ok(current_state_link)
    }

    fn append_to_follow_up_events(&mut self, new_follow_up_events: &mut StateEventVec) {
        self.follow_up_events_requested.append(new_follow_up_events);
    }

    fn set_current_state(&mut self, new_current_state: StateId) {
        self.current_state = Some(new_current_state)
    }

    fn get_states(&self) -> &StatesVec {
        &self.states
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
/// Effectively the public API to the controller for consumers.
/// After it is destroyed / init is called, the controller is self-managing
pub struct HsmControllerBuilder {
    controller_under_construction: HSMControllerBase,
}

impl HsmControllerBuilder {
    pub fn new(hsm_name: String) -> HsmControllerBuilder {
        let controller = HSMControllerBase::new(hsm_name);

        HsmControllerBuilder {
            controller_under_construction: controller,
        }
    }

    pub fn add_state(mut self, new_state: StateRef) -> Self {
        let state_chain = Rc::new(RefCell::new(StateChainOfResponsibility::new(
            new_state.clone(),
            new_state.borrow().get_state_data(),
        )));
        self.controller_under_construction.states.push(state_chain);
        self
    }

    /// Final step in process
    pub fn init(mut self, initial_state_id: u16) -> HSMResult<HSMControllerBase> {
        let initial_state_id_struct = StateId::new(initial_state_id);
        let states = self.controller_under_construction.get_states();

        states
            .iter()
            .find(|state| state.borrow().is_state(&initial_state_id_struct))
            .ok_or_else(|| {
                HSMError::InvalidStateId(format!(
                    "Initial State with Id {} is not valid. There are no added states with that id",
                    initial_state_id
                ))
            })?;

        self.controller_under_construction
            .set_current_state(initial_state_id_struct.clone());
        Ok(self.controller_under_construction)
    }
}
