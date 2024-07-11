///! This file contains the logic for an individual state and how they link together
use std::{cell::RefCell, rc::Rc};

use crate::{
    errors::HSMResult,
    events::StateEventsIF,
    state_data_delegate::{StateDataDelegate, StateDelegateRef},
};

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

/// The State reference accepted as inputs for the controller to manage
pub type StateRef = Rc<RefCell<dyn StateIF>>;

/// The only owner of the chain is the controller! no one else should be aware of its existence!
pub type StateChainRef = Rc<RefCell<StateChainOfResponsibility>>;
pub type StatesVec = Vec<StateChainRef>;

pub trait StateIF {
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

    // TODO - I really really wish there was a way to remove this
    // At the very least we have limited the blast range&damage through encapsulation
    fn get_state_data(&self) -> StateDelegateRef;

    // Have a state fire an event back at the controller while handling another event!
    // Dispatches another event to the controller from an internal state.
    // Allows a state to fire into the controller (i.e. a timer expires).
    // Will append to list of other events dispatched internal
}

/// If you have one chain in the link, you can navigate around the rest of the links in the tree!
pub struct StateChainOfResponsibility {
    pub(crate) state: StateRef,
    pub(crate) delegate: Rc<RefCell<StateDataDelegate>>,
}

impl StateChainOfResponsibility {
    pub(crate) fn new(state: StateRef, delegate: StateDelegateRef) -> Self {
        Self { state, delegate }
    }

    pub(crate) fn handle_event(&self, event: &dyn StateEventsIF) -> bool {
        self.state.borrow_mut().handle_event(event)
    }

    /// Delegate behavior to the link's data member
    pub(crate) fn delegate_operation(&self) -> StateDelegateRef {
        self.delegate.clone()
    }

    /// Gets the path to root. Including self and root.
    pub(crate) fn get_path_to_root_state(&self) -> HSMResult<Vec<StateId>> {
        let mut path_to_root = Vec::<StateId>::new();
        path_to_root.push(
            self.delegate
                .borrow()
                .get_details()?
                .borrow()
                .get_state_id()
                .clone(),
        );

        let mut current_state_delegate = self
            .delegate_operation()
            .borrow()
            .get_details()?
            .borrow()
            .get_parent_delegate();

        while let Some(state) = current_state_delegate {
            // path_to_root.push(state.borrow().get_state_id().clone());
            path_to_root.push(state.borrow().get_details()?.borrow().get_state_id());

            let opt_parent_state_delegate =
                state.borrow().get_details()?.borrow().get_parent_delegate();
            current_state_delegate = opt_parent_state_delegate;
        }

        Ok(path_to_root)
    }

    pub(crate) fn is_state(&self, state_id: &StateId) -> bool {
        &self
            .delegate
            .borrow()
            .get_details()
            .expect("Compared a state to ours before our delegate was initialized!")
            .borrow()
            .get_state_id()
            == state_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // todo - more tests
}
