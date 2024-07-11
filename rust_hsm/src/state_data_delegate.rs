///! Module encapsulating the state data delegate which can be used extensively
///! throughout the library but is obscured to consumers
use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventRef,
    state::{StateId, StateRef},
};

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub type StateDelegateRef = Rc<RefCell<StateDataDelegate>>;
type StateDelegateDetailRef = Rc<RefCell<StateDataDelegateDetail>>;

/// Channel to send commands & info from StateFoo -> Controller while handling.
/// The HSM backend uses this information to properly handle events.
/// Helps decouple StateIF and handling of events from sending messages back to the controller!
pub struct StateDataDelegate {
    /// Details of the delegate consumers of the library should not be aware of
    pub(self) details: Option<StateDelegateDetailRef>,
}

/// Encapsulates all details for what is needed for a true delegate
pub(crate) struct StateDataDelegateDetail {
    pub(crate) state_id: StateId,
    // None if there is no parent state (i.e. TOP state)
    pub(crate) state_name: String,
    pub(crate) current_state: StateRef,
    /// The chain of responsibility from the parent node and up. Disconnected from our node.
    /// TODO - move this out of the detail?
    pub(crate) parent_delegate: Option<StateDelegateRef>,
    pub(crate) requested_state_change: Option<StateId>,
    pub(crate) follow_up_events_requested: VecDeque<StateEventRef>,
}

impl StateDataDelegate {
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
    pub fn submit_state_change_request(&mut self, new_state: u16) -> HSMResult<()> {
        self.get_details()?.borrow_mut().requested_state_change = Some(StateId::new(new_state));
        Ok(())
    }

    pub fn dispatch_event_internally(&mut self, event: StateEventRef) -> HSMResult<()> {
        self.get_details()?
            .borrow_mut()
            .follow_up_events_requested
            .push_back(event);
        Ok(())
    }

    /// Build a temporary version of the delegate while a real consumer is still
    /// instantiating their state!
    pub(crate) fn build_temporary() -> StateDelegateRef {
        Rc::new(RefCell::new(StateDataDelegate { details: None }))
    }

    /// Used by Builder to complete the real delegate when it is possible
    pub(crate) fn build_real(
        state_id: u16,
        state_name: String,
        current_state: StateRef,
        parent_delegate: Option<StateDelegateRef>,
    ) -> StateDelegateRef {
        // Although the consumer is unaware, we have enough to rebuild the
        // chain from minimal context without leaking its existence to them.
        let details =
            StateDataDelegateDetail::new(state_id, state_name, current_state, parent_delegate);

        Rc::new(RefCell::new(StateDataDelegate {
            details: Some(details),
        }))
    }

    /// Meant to be used by the builder once the true delegate is created
    pub(crate) fn set_details(&mut self, new_delegate: StateDelegateRef) -> HSMResult<()> {
        let new_details = new_delegate.borrow_mut().get_details()?;
        self.details.replace(new_details);
        Ok(())
    }

    pub(crate) fn get_details(&self) -> HSMResult<StateDelegateDetailRef> {
        match self.details.clone() {
            None => Err(HSMError::DelegateDetailsNotSet()),
            Some(details) => Ok(details),
        }
    }
}

/// TODO - This might be a very C++/C pImpl approach to the design.
/// Maybe the crate impls should be brought back to the main delegate?
/// Why removed from main struct:
///         cluttered struct impl with functions the user shouldn't even have to know exist.
///         How is that any different from many pub(crate) fn and only a few pub fn? less clutter??
/// Even the idea that consumers could get access to the innards is scary though.
/// Best practice research is required.
impl StateDataDelegateDetail {
    pub(crate) fn new(
        state_id: u16,
        state_name: String,
        current_state: StateRef,
        parent_delegate: Option<StateDelegateRef>,
    ) -> StateDelegateDetailRef {
        Rc::new(RefCell::new(StateDataDelegateDetail {
            state_id: StateId::new(state_id),
            state_name,
            current_state,
            parent_delegate,
            requested_state_change: None,
            follow_up_events_requested: VecDeque::new(),
        }))
    }

    pub(crate) fn get_state_id(&self) -> StateId {
        self.state_id.clone()
    }

    pub(crate) fn get_state_name(&self) -> String {
        self.state_name.clone()
    }

    pub(crate) fn get_current_state_trait(&self) -> StateRef {
        self.current_state.clone()
    }

    pub(crate) fn get_parent_delegate(&self) -> Option<StateDelegateRef> {
        self.parent_delegate.clone()
    }

    /// Retrieves the requested state change by consuming it! Resets the value.
    /// This ensures the same change state is not accidentally requested twice
    /// (i.e. if it is not cleared after it is done)
    pub(crate) fn get_and_reset_requested_state_change(&mut self) -> Option<StateId> {
        self.requested_state_change.take()
    }

    /// Retrieves the next event requested for processing by consuming it!
    /// This ensures the same event is not accidentally performed twice.
    /// No-op if there are no follow-up / requested events!
    /// Similar to the data structure API but exposes to controller trait!
    pub(crate) fn get_and_reset_follow_up_events(&mut self) -> VecDeque<StateEventRef> {
        let consumed = self.follow_up_events_requested.clone();
        self.follow_up_events_requested.clear();
        consumed
    }
}
