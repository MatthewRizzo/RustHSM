//! Module encapsulating the state data delegate which can be used extensively
//! throughout the library but is obscured to consumers
use crate::{errors::HSMResult, events::StateEventConstraint};
use std::rc::Rc;

/// Trait representing a valid object delegating powers of the Engine to others (states).
/// Allows states to know about the HSM while the HSM knows about the states (indirectly through their trait).
pub trait EngineDelegateIF<StateT, EventT: StateEventConstraint> {
    /// Command the HSM to change state while handling your event.
    fn change_state(&self, new_state: u16) -> HSMResult<(), StateT>;

    /// Command the HSM to handle an event.
    /// If this is called while handling another event, it will be queued until the current completes.
    /// If many requests are queued by states, they will be handled FIFO.
    fn internal_handle_event(&self, event: EventT) -> HSMResult<(), StateT>;
}

pub type EngineDelegate<StateT, EventT> = Rc<dyn EngineDelegateIF<StateT, EventT>>;

#[cfg(test)]
pub mod delegate_test_utils {
    use std::{cell::RefCell, default::Default, marker::PhantomData};

    use super::*;
    use crate::events::StateEventConstraint;

    #[derive(Debug, strum::Display, PartialEq, Clone)]
    pub enum DelegateTestEvent {
        TestA,
        TestB(String),
    }

    /// Mocked delegate that can be used to test states separate from the engine.
    pub struct MockedDelegate<StateT, EventT: StateEventConstraint> {
        pub change_states_requested: RefCell<Vec<u16>>,
        pub internal_events_handled: RefCell<Vec<EventT>>,
        marker: PhantomData<StateT>,
    }

    impl<StateT, EventT: StateEventConstraint> MockedDelegate<StateT, EventT> {
        pub fn new() -> Self {
            Self {
                change_states_requested: RefCell::new(vec![]),
                internal_events_handled: RefCell::new(vec![]),
                marker: PhantomData,
            }
        }
    }

    impl<StateT, EventT: StateEventConstraint> EngineDelegateIF<StateT, EventT>
        for MockedDelegate<StateT, EventT>
    {
        fn change_state(&self, new_state: u16) -> HSMResult<(), StateT> {
            self.change_states_requested.borrow_mut().push(new_state);
            Ok(())
        }

        fn internal_handle_event(&self, event: EventT) -> HSMResult<(), StateT> {
            self.internal_events_handled.borrow_mut().push(event);
            Ok(())
        }
    }

    fn create_mock_delegate<StateT, EventT: StateEventConstraint>() -> MockedDelegate<StateT, EventT>
    {
        MockedDelegate {
            change_states_requested: Default::default(),
            internal_events_handled: Default::default(),
            marker: PhantomData,
        }
    }
}
