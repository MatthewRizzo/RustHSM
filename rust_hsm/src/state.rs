//! This file contains the logic for an individual state and how they link together
use std::{boxed::Box, fmt::Display, vec::Vec};

use crate::{
    errors::HSMResult, events::StateEventConstraint, state_engine_delegate::EngineDelegate,
};

/// All valid definitions of a 'class' of state's must be StateTypes.
/// By enforcing these characteristics, the Engine can translate from its
/// limited knowledge set to the true state typing provided by the consumer.
pub trait StateConstraint: Display + Into<u16> + From<u16> + Clone {}

/// An inexpensive token representing a state that can be exchanged for more
/// complex data structures.
#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy)]
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

/// Definition of what makes a struct/enum a state.
/// We assume states are immutable, but if you need to mutate interior data, feel free to do so.
/// Be aware, if you borrow during handle_event and handle_state_*, but do not release it before change_state_during_handle, you could panic.
pub trait StateIF<StateT, EventT: StateEventConstraint> {
    /// Called whenever state is entered (even if transiently).
    /// If multiple states are traveled through, it is called multiple times
    /// by every relevant state.
    /// Note: Can only be called in response to a state (maybe even us)'s handle_event
    fn handle_state_enter(&self) {}

    /// Only called by state settled on in order to handle the event.
    /// Note: Can only be called in response to a state (maybe even us)'s handle_event
    fn handle_state_start(&self) {}

    /// Called when transitioning out of the state
    /// Note: Can only be called in response to a state (maybe even us)'s handle_event
    fn handle_state_exit(&self) {}

    /// All state's implement this.
    /// Recommendation is converting event to an enum and handling the cases you want.
    /// # Return
    /// * True if handled. Do not keep handling
    /// * False if not handled and should be delegated to a higher state.
    fn handle_event(&self, event: &EventT) -> bool;

    /// # Note
    /// Can only be called in response to a state (maybe even us)'s handle_event.
    /// Consequently, make sure any borrows done during handle_event are released before calling this function!
    fn change_state_during_handle(
        &self,
        new_state: u16,
        delegate: EngineDelegate<StateT, EventT>,
    ) -> HSMResult<(), StateT> {
        delegate.change_state(new_state)
    }
}

pub type StateBox<StateT, EventT> = Box<dyn StateIF<StateT, EventT>>;
pub type States<StateT, EventT> = Vec<StateBox<StateT, EventT>>;

/// All elements are cheap data structure or those with copy/clone/rc semantics
pub(crate) struct StateContainer<StateT: StateConstraint, EventT: StateEventConstraint> {
    pub state_ref: StateBox<StateT, EventT>,
    pub state_id: StateId,
}

impl<StateT: StateConstraint, EventT: StateEventConstraint> StateContainer<StateT, EventT> {
    pub(crate) fn new(state_id: StateId, state_ref: StateBox<StateT, EventT>) -> Self {
        Self {
            state_ref,
            state_id,
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // todo - more tests
}
