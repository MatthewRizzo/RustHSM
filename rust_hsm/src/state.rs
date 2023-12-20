///! Contains generic struct representing a valid state / its traits
use crate::tree::{NodeDataConstraints, Tree};
use std::collections::VecDeque;

/// Traits that all state's must implement to be used
pub trait StateBaseBehavior {
    /// Called whenever state is entered (even if transiently).
    /// If multiple states are traveled through, it is called multiple times
    /// by every relevant state
    fn handle_state_enter(&self);

    /// Only called by state settled on in order to handle the event
    fn handle_state_start(&self);

    /// Called when transitioning out of the state
    fn handle_state_exit(&self);
}

/// Trait all HSM context's need to implement
trait ChangeState<StateImpl> {
    type T;
    // Change state to the state with the provided id
    fn change_state(&mut self, state_id: u32);
}

#[cfg(test)]
mod tests {
    use super::*;
    // todo - more tests
}
