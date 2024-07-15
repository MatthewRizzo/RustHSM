///! This file contains the logic for an individual state and how they link together
use std::{boxed::Box, cell::RefCell, fmt::Display};

use crate::events::StatefulEvent;

/// All valid definitions of a 'class' of state's must be StateTypes.
/// By enforcing these characteristics, the Engine can translate from its
/// limited knowledge set to the true state typing provided by the consumer.
pub trait StateTypeTrait: Display + Into<u16> + From<u16> + Clone {}

/// An inexpensive token representing a state that can be exchanged for more
/// complex data structures.
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub(crate) struct StateId {
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
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "id: {}", self.id)
    }
}

impl From<u16> for StateId {
    fn from(state_id: u16) -> Self {
        Self::new(state_id)
    }
}

/// The State reference accepted as inputs for the controller to manage
pub type StateBox<StateType> = Box<dyn StateIF<StateType>>;

pub trait StateIF<StateType: StateTypeTrait> {
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
    fn handle_event(&mut self, event_id: &StatefulEvent) -> bool;
}

/// All elements are cheap data structure or those with copy/clone/rc semantics
pub(crate) struct StateContainer<StateType: StateTypeTrait> {
    pub state_ref: RefCell<StateBox<StateType>>,
    pub state_id: StateId,
}

impl<StateType: StateTypeTrait> StateContainer<StateType> {
    pub(crate) fn new(state_id: StateId, state_ref: StateBox<StateType>) -> Self {
        Self {
            state_ref: RefCell::new(state_ref),
            state_id,
        }
    }

    pub(crate) fn get_state_id(&self) -> &StateId {
        &self.state_id
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // todo - more tests
}
