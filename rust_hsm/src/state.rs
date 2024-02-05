///! This file contains the logic for an individual state and how they link together
use std::{cell::RefCell, rc::Rc};

use crate::{
    callback::Callback,
    callback_registry::{CallbackRegistry, CallbackRegistryPair},
    events::DecoratableEventBase,
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

pub type RefStates = Vec<Rc<RefCell<ComposableStateBase>>>;

pub type EventHandlerRegistry = CallbackRegistry<(), DecoratableEventBase, Callback<(), bool>>;

// Base state struct your actual state's should be composed of
pub struct ComposableStateBase {
    state_id: StateId,
    // None if there is no parent state (i.e. TOP state)
    state_name: String,
    parent_state: Option<Rc<RefCell<Self>>>,
    event_handlers: Rc<RefCell<EventHandlerRegistry>>,
    handle_state_exit: Option<Callback<(), ()>>,
    handle_state_enter: Option<Callback<(), ()>>,
    handle_state_start: Option<Callback<(), ()>>,
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
}

impl<'a> ComposableStateBase {
    /// RAII register all event handlers!
    /// If multiple handlers for the same function are registered, last one wins!
    pub fn new(
        state_id: StateId,
        state_name: String,
        parent_state: Option<Rc<RefCell<Self>>>,
        handle_state_exit: Option<Callback<(), ()>>,
        handle_state_enter: Option<Callback<(), ()>>,
        handle_state_start: Option<Callback<(), ()>>,
        event_handlers: Vec<CallbackRegistryPair<(), DecoratableEventBase, Callback<(), bool>>>,
    ) -> Self {
        let registry = CallbackRegistry::new(event_handlers);

        ComposableStateBase {
            state_id,
            state_name,
            parent_state,
            event_handlers: Rc::new(RefCell::new(registry)),
            handle_state_exit,
            handle_state_enter,
            handle_state_start,
        }
    }

    pub(crate) fn get_state_name(&self) -> &String {
        &self.state_name
    }

    pub fn get_state_id(&self) -> &StateId {
        &self.state_id
    }

    /// Gets path to root from this state. Inclusive on both ends.
    pub(crate) fn get_path_to_root(&self) -> Vec<StateId> {
        let mut root_path: Vec<StateId> = vec![];
        root_path.push(self.state_id.clone());

        let current_state_optional = self;
        while let Some(current_state) = current_state_optional.parent_state.clone() {
            let current_state_id = &current_state.as_ref().borrow().state_id;
            root_path.push(current_state_id.clone());
        }

        root_path
    }

    pub(crate) fn handle_state_exit(&self) {
        if self.handle_state_exit.is_some() {
            Callback::fire_through_reference(&self.handle_state_exit, ());
        }
    }

    pub(crate) fn handle_state_enter(&self) {
        if self.handle_state_enter.is_some() {
            Callback::fire_through_reference(&self.handle_state_enter, ());
        }
    }

    pub(crate) fn handle_state_start(&self) {
        if self.handle_state_start.is_some() {
            Callback::fire_through_reference(&self.handle_state_start, ());
        }
    }

    pub(crate) fn get_parent_state(current_state: &Self) -> Option<Rc<RefCell<Self>>> {
        current_state.parent_state.clone()
    }

    /// True if this state handles it. False otherwise.
    pub(crate) fn handle_event(current_state: &Self, event: &DecoratableEventBase) -> bool {
        let current_registry = &current_state.event_handlers;
        current_registry
            .as_ref()
            .borrow()
            .dispatch_to_registry(event, ())
            .is_some()
    }
}

impl PartialEq<ComposableStateBase> for ComposableStateBase {
    fn eq(&self, other: &ComposableStateBase) -> bool {
        self.state_id == other.state_id && self.parent_state == other.parent_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // todo - more tests
}
