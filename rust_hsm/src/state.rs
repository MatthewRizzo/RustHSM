use std::{rc::Rc, cell::RefCell};

///! Contains generic struct representing a valid state / its traits
use crate::tree::{NodeDataConstraints, Tree, TreeOperations, NodeOperations};

pub struct StateId {
    pub id: u16
}


impl StateId {
    pub fn new(state_id: u16) -> Self {
        StateId {id: state_id}
    }
}

pub struct EventBase {
    event_id: u16
}

impl EventBase {
    pub fn new(event_id: u16) -> EventBase {
        EventBase { event_id }
    }
}

// Compose / decorate your hsm controller with this
// todo - do this using dynamic dispatch
pub struct HSMControllerDecoratorBase
{
    states: Vec<Rc<RefCell<dyn StateBaseBehavior>>>,
    current_state: Rc<RefCell<dyn StateBaseBehavior>>
}

impl HSMControllerDecoratorBase {
    /// get LCA between current state and other state
    fn find_lca(&self, other_state: Rc<RefCell<Self>>) -> Rc<RefCell<Self>>
    {

    }

}

/// Traits that all state's must implement to be used
pub trait StateBaseBehavior {
    // type EventEnum;

    /// Called whenever state is entered (even if transiently).
    /// If multiple states are traveled through, it is called multiple times
    /// by every relevant state
    fn handle_state_enter(&mut self) {}

    /// Only called by state settled on in order to handle the event
    fn handle_state_start(&mut self) {}

    /// Called when transitioning out of the state
    fn handle_state_exit(&mut self) {}

    /// Attempts to handle the event with the given id.
    /// # Return
    /// * true if this state handles it
    /// * false if this state does not handle it
    fn handle_event(&mut self, event_id: u16) -> bool {
        false
    }

    // todo - get rid of fancy tree and just impl this
    // tie API for controller / context to just using this
    fn get_parent_state(&mut self) -> Self;
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
