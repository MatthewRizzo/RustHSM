///! This file contains the logic for an individual state and how they link together
use std::{cell::RefCell, rc::Rc};

use crate::{events::StateEventsIF, state_controller_trait::HsmControllerRef};

#[derive(PartialEq, Clone)]
pub struct StateId {
    id: u16,
}

/// First valid state begins at 0!
/// State Id can be used as vector index's too
impl StateId {
    pub fn new(state_id: u16) -> Self {
        StateId { id: state_id }
    }

    pub fn get_id(&self) -> &u16 {
        &self.id
    }

    pub fn is_top_state(&self) -> bool {
        self.id == Self::get_top_state_id()
    }

    pub fn get_top_state_id() -> u16 {
        0
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "id: {}", self.id)
    }
}

pub type StatesRefVec = Vec<Rc<RefCell<dyn StateChainOfResponsibility>>>;
pub type StateRef = Rc<RefCell<dyn StateChainOfResponsibility>>;

/// Traits that all state's must implement to be used
/// # Reference
/// https://github.com/lpxxn/rust-design-pattern/blob/master/behavioral/chain_of_responsibility.rs
pub trait StateChainOfResponsibility {
    /// Called whenever state is entered (even if transiently).
    /// If multiple states are traveled through, it is called multiple times
    /// by every relevant state
    fn handle_state_enter(&mut self) {}

    /// Only called by state settled on in order to handle the event
    fn handle_state_start(&mut self) {}

    /// Called when transitioning out of the state
    fn handle_state_exit(&mut self) {}

    /// All state's implement this.
    /// Recommendation is converting event to an enum and handling the cases you want.
    /// # Return
    /// * True if handled. Do not keep handling
    /// * False if not handled and should be delegated to a higher state.
    fn handle_event(&mut self, event_id: &dyn StateEventsIF) -> bool;

    fn get_state_data(&self) -> &ComposableStateData;
    fn get_state_data_mut(&mut self) -> &mut ComposableStateData;

    fn get_state_id(&self) -> StateId {
        self.get_state_data().get_state_id()
    }

    fn get_super_state(&self) -> Option<StateRef> {
        self.get_state_data().get_parent_state()
    }

    fn get_state_name(&self) -> String {
        self.get_state_data().get_state_name()
    }

    fn get_hsm(&self) -> HsmControllerRef {
        self.get_state_data().get_hsm()
    }

    /// Gets the path to root. Including self and root.
    fn get_path_to_root_state(&self) -> Vec<StateId> {
        let mut path_to_root = Vec::<StateId>::new();
        path_to_root.push(self.get_state_id().clone());

        let mut current_state = self.get_super_state();

        while let Some(state) = current_state {
            path_to_root.push(state.borrow().get_state_id().clone());

            let opt_parent_state = state.borrow().get_super_state();
            current_state = opt_parent_state;
        }

        path_to_root
    }
}

// Base state struct your actual state's should be composed of
// Has all the information you need to impl the data-oriented API's of the state trait
pub struct ComposableStateData {
    state_id: StateId,
    // None if there is no parent state (i.e. TOP state)
    state_name: String,
    parent_state: Option<StateRef>,
    state_machine: HsmControllerRef,
    requested_state_change: Option<StateId>,
}

impl ComposableStateData {
    pub fn new(
        state_id: u16,
        state_name: String,
        parent_state: Option<StateRef>,
        state_machine: HsmControllerRef,
    ) -> Self {
        Self {
            state_id: StateId::new(state_id),
            state_name,
            parent_state,
            state_machine,
            requested_state_change: None,
        }
    }

    pub(crate) fn get_state_id(&self) -> StateId {
        self.state_id.clone()
    }

    pub(crate) fn get_state_name(&self) -> String {
        self.state_name.clone()
    }

    pub(crate) fn get_parent_state(&self) -> Option<StateRef> {
        self.parent_state.clone()
    }

    /// Retrieves the requested state change by consuming it! Resets the value.
    /// This ensures the same change state is not accidentally requested twice
    /// (i.e. if it is not cleared after it is done)
    pub(crate) fn get_and_reset_requested_state_change(&mut self) -> Option<StateId> {
        self.requested_state_change.take()
    }

    pub fn get_hsm(&self) -> HsmControllerRef {
        self.state_machine.clone()
    }

    /// Stores the requested state change.
    /// The controller will reap the new value once done with its current processing.
    /// Afterwards, this value will be reset.
    /// # Why
    /// The request cannot be submit directly to the controller.
    /// Complicated reason that simplifies to: triggering an event in the controller causes
    /// it to be borrowed mutably.
    /// Likewise, updating the hsm cache to have a new state requires a mutable borrow.
    /// If change state was submit to the controller directly,
    /// the state dispatched to would borrow the controller AGAIN causing a panic.
    /// Instead, indirectly submit the request to the data cache (even if borrowed it is dropped immediately).
    /// Then have the controller "reap" the results of the change request once it is done handling
    /// the event; no extra borrows required.
    pub fn submit_state_change_request(&mut self, new_state: u16) {
        self.requested_state_change = Some(StateId::new(new_state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // todo - more tests
}
