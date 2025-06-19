///! Module encapsulating the state data delegate which can be used extensively
///! throughout the library but is obscured to consumers
use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventTrait,
    state::{StateId, StateTypeTrait},
};

use std::{marker::PhantomData, sync::mpsc::Sender};

type RequestingStateId = StateId;

pub(crate) enum StateEngineMessages<StateEvents> {
    ChangeState(RequestingStateId, StateId),
    FireEvent(RequestingStateId, StateEvents),
}

/// # What is this?
///
/// * A delegate that can owned by 1 state.
/// * It is used to send messages from trait type state object -> engine.
/// * There is a 1-1 relationship between a state/state_id and its delegate
///     * This is enforced by the engine builder.
/// * There is a 1-* relationship between the engine and delegates via channel.
/// * The engine is not aware that delegates exist.
///
/// # Why Is this Necessary?
/// * Concrete State implementation must have affordances to talk to the engine
/// to lodge various requests.
/// * For example, changing state, or firing another event internal the engine.
/// * However, the engine must ALSO be aware of states because it OWNS them.
/// * This implicitly creates a circular-mutable dependency; an untenable design
/// * The solution is delegates, i.e. this struct.
///
/// # How it Works
/// * The engine owns the rx of a channel.
/// * Each delegate owns a copy to the tx side of a channel.
/// * The HSMEngineBuilder creates the channel; providing rx to its engine.
/// * Consumers of the engine-under construction request info from builder:
///     * Unique delegate for a given StateId.
///         * Multiple attempts to create a delegate for a given state error.
///         * The only "shared" ownership is for a given tx channel reserved
///           ONCE per state. ONLY states are allowed to create senders!
///         * Non-states should talk to the engine via non-channel mechanisms!
///             * ....for now. In theory the engine api's could be channels.
/// * States do not talk to the engine. They talk to their delegate.
///     * Internally, the delegate is sending requests to engine through channel
///     * However (and this is important for mutability!), the state is unaware
/// * From the engine's perspective, it checks the rx side of delegate channel.
///     * The check should be done lazily; while handling an event.
///     * States/delegates are unable to send requests without prompting event.
///     * The engine handles pending requests sent by delegates over their tx.
/// ## Advantages
///
/// * Avoids circular dependency between engine<->states
/// * Helps simplify state-side code to talk to engine.
/// * Allows for simpler integration to consumers via builders.
/// * Reduces the number of Rc\<Refcell<Foo\>> and its related borrowing.
/// * Reduces overall mutability of the system.
///
/// ## Disadvantages
///
/// * Requires more code on both sides the channel management within the library
///
/// ```rust
/// use crate::state_engine::HSMEngineBuilder;
/// use crate::test_utils::TestStates;
/// use std::boxed::Box;
/// use log::LevelFilter;
/// let builder = HSMEngineBuilder::new("FooHsm", 1, LevelFilter::Info, LevelFilter::Info);
///
/// let state_1a_delegate = builder.create_delegate(TestStates::LevelA1)?;
/// let state_1a_delegate_copy = builder.create_delegate(TestStates::LevelA1)?; // This WILL error
/// let state_1b_delegate = builder.create_delegate(TestStates::LevelAB)?;
///
/// let state_1a: Box<A1Impl> = A1Impl::new(state_1a_delegate.to_owned());
/// let state_1b: Box<ABImpl> = ABImpl::new(state_1b_delegate.to_owned());
/// let engine: HSMEngine<TestStates> = builder
///     .add_state(state_1a.to_owned(), TestStates::LevelA1, Some(TestStates::Top))
///     .add_state(state_1b.to_owner(), TestStates::LevelAB, Some(TestStates::Top))
///     .init(TestStates::LevelA1)
///     .unwrap();
/// ```
pub struct StateEngineDelegate<StateType: StateTypeTrait, StateEvents: StateEventTrait> {
    pub(crate) sender_to_engine: Sender<StateEngineMessages<StateEvents>>,
    /// Think of this like a user-agent and or a token to provide the engine for
    /// each request!
    delegated_state_id: StateId,
    state_enum_phantom: PhantomData<StateType>,
}

/// # Params
/// * delegated_state_id the Id of the state requesting this delegate!
impl<StateType: StateTypeTrait, StateEvents: StateEventTrait>
    StateEngineDelegate<StateType, StateEvents>
{
    pub(crate) fn new(
        sender_to_engine: Sender<StateEngineMessages<StateEvents>>,
        delegated_state_id: StateId,
    ) -> Self {
        Self {
            sender_to_engine,
            delegated_state_id,
            state_enum_phantom: PhantomData,
        }
    }
    /// # Why
    /// The request cannot be submit directly to the controller.
    /// Complicated reason that simplifies to: triggering an event in the controller causes
    ///     it to be borrowed mutably.
    /// Likewise, updating the hsm cache to have a new state requires a mutable borrow.
    /// If change state was submit to the controller directly, the state
    /// dispatched to would borrow the controller AGAIN causing a panic.
    /// Instead, indirectly submit the request to the data cache
    ///     (even if borrowed it is dropped immediately).
    /// Then have the controller "reap" the results of the change request once
    ///     it is done handling the event; no extra borrows required.
    pub fn change_state(&mut self, new_state: u16) -> HSMResult<(), StateType> {
        let evt = StateEngineMessages::ChangeState(
            self.delegated_state_id.clone(),
            StateId::new(new_state),
        );
        self.sender_to_engine
            .send(evt)
            .map_err(|_| HSMError::DelegateNotConnected())
    }

    pub fn dispatch_event_internally(&mut self, event: StateEvents) -> HSMResult<(), StateType> {
        let evt = StateEngineMessages::FireEvent(self.delegated_state_id.clone(), event);
        self.sender_to_engine
            .send(evt)
            .map_err(|_| HSMError::DelegateNotConnected())
    }
}
