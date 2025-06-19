///! Module encapsulating the state data delegate which can be used extensively
///! throughout the library but is obscured to consumers
use crate::{
    errors::{HSMError, HSMResult},
    events::StateEventTrait,
    state::{StateId, StateTypeTrait},
};

use std::{marker::PhantomData, sync::mpsc::Sender};

type RequestingStateId = StateId;
type TargetedStateId = StateId;

pub(crate) enum StateEngineMessages<StateEvents> {
    ChangeState(RequestingStateId, TargetedStateId),
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
/// use rust_hsm::state_engine::{HSMEngine, HSMEngineBuilder};
/// use rust_hsm::examples::{ExampleStates, ExampleEvents, Top, A1Impl, B1Impl, A2Impl};
/// use std::boxed::Box;
/// use log::LevelFilter;
/// let builder: HSMEngineBuilder<ExampleStates, ExampleEvents> = HSMEngineBuilder::new(
///     "FooHsm".to_string(),
///     ExampleStates::Top as u16,
///     LevelFilter::Info, LevelFilter::Info
/// );
///
/// let top_delegate = builder.create_delegate(ExampleStates::Top as u16).expect("");
/// let state_1a_delegate = builder.create_delegate(ExampleStates::LevelA1 as u16).expect("");
/// let state_1a_delegate_copy = builder.create_delegate(ExampleStates::LevelA1 as u16); // This WILL error
/// assert!(state_1a_delegate_copy.is_err());
/// let state_1b_delegate = builder.create_delegate(ExampleStates::LevelA2 as u16).expect("");
///
/// let state_top: Box<A1Impl> = A1Impl::new(top_delegate);
/// let state_1a: Box<A1Impl> = A1Impl::new(state_1a_delegate);
/// let state_1b: Box<B1Impl> = B1Impl::new(state_1b_delegate);
/// // re-using delegates will fail to compile because they cannot be copied or re-used after move
/// // let state_2a: Box<A2Impl> = A2Impl::new(state_1a_delegate);
/// let engine: HSMEngine<ExampleStates, ExampleEvents> = builder
///     .add_state(state_top, ExampleStates::Top, None)
///     .add_state(state_1a, ExampleStates::LevelA1, Some(ExampleStates::Top))
///     .add_state(state_1b, ExampleStates::LevelB1, Some(ExampleStates::Top))
///     .init(ExampleStates::LevelA1 as u16)
///     .expect("Failed to init hsm");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::mpsc::{channel, Receiver},
        time::Duration,
    };
    use crate::examples::*;

    #[derive(Debug, strum::Display, PartialEq, Clone)]
    pub enum DelegateTestEvent {
        TestA,
        TestB(String),
    }

    impl StateEventTrait for DelegateTestEvent {}

    struct MockedDelegate
    {
        delegate: StateEngineDelegate<ExampleStates, DelegateTestEvent>,
        mock_rx_channel: Receiver<StateEngineMessages<DelegateTestEvent>>
    }

    impl MockedDelegate {
        fn get_next_event(&mut self) -> Option<StateEngineMessages<DelegateTestEvent>>{
            match self.mock_rx_channel.recv_timeout(Duration::new(0, 0)) {
                Err(_) => None, // all proxy requests have been processed! We are done!
                Ok(req) => Some(req),
            }
        }
    }

    fn create_mock_delegate(
        state_id: u16
    ) -> MockedDelegate
    {
        let (tx, rx) = channel::<StateEngineMessages<DelegateTestEvent>>();
        let delegate = StateEngineDelegate::new(
            tx,
            StateId::new(state_id)
        );

        MockedDelegate{
            delegate,
            mock_rx_channel: rx
        }
    }

    fn is_evt_change_state(
        evt: Option<StateEngineMessages<DelegateTestEvent>>,
        expected_requester: u16,
        expected_target: u16
    ) -> bool
    {
        if evt.is_none() {
            return  false;
        }

        match evt.unwrap() {
            StateEngineMessages::ChangeState(requester, target) => {
                *requester.get_id() == expected_requester
                && *target.get_id() == expected_target
            }
            StateEngineMessages::FireEvent(_, _) => false
        }
    }

    fn is_evt_dispatch_event(
        evt: Option<StateEngineMessages<DelegateTestEvent>>,
        expected_requester: u16,
        expected_sent_event: DelegateTestEvent
    ) -> bool
    {
        if evt.is_none() {
            return false;
        }

        match evt.unwrap() {
            StateEngineMessages::FireEvent(requester, sent_event) => {
                *requester.get_id() == expected_requester
                && sent_event == expected_sent_event
            }
            StateEngineMessages::ChangeState(_, _) => false
        }
    }

    #[test]
    fn change_state()
    {
        let state_id = 0;
        let new_state_id_1 = 1;
        let new_state_id_2 = 2;
        let new_state_id_3 = 3;
        let new_state_id_4 = 4;
        let mut mock = create_mock_delegate(state_id);
        mock.delegate.change_state(new_state_id_1).expect("Sending change state should work!");

        let received_evt = mock.get_next_event();
        assert!(is_evt_change_state(received_evt, state_id, new_state_id_1));
        assert!(mock.get_next_event().is_none());

        println!("Sending many change state's");
        mock.delegate.change_state(new_state_id_2).expect("Sending change state should work!");
        mock.delegate.change_state(new_state_id_3).expect("Sending change state should work!");
        mock.delegate.change_state(new_state_id_4).expect("Sending change state should work!");
        mock.delegate.change_state(new_state_id_1).expect("Sending change state should work!");

        assert!(is_evt_change_state(mock.get_next_event(), state_id, new_state_id_2));
        assert!(is_evt_change_state(mock.get_next_event(), state_id, new_state_id_3));
        assert!(is_evt_change_state(mock.get_next_event(), state_id, new_state_id_4));
        assert!(is_evt_change_state(mock.get_next_event(), state_id, new_state_id_1));

    }

    #[test]
    fn dispatch_event_internally()
    {
        let state_id = 0;
        let mut mock = create_mock_delegate(state_id);

        let evt_a = DelegateTestEvent::TestA;
        let evt_b = DelegateTestEvent::TestB("FakeString".to_string());
        mock.delegate.dispatch_event_internally(evt_a.clone()).expect("Sending event should work!");
        mock.delegate.dispatch_event_internally(evt_b.clone()).expect("Sending event should work!");

        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_a.clone()));
        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_b.clone()));

        // Channel is empty
        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_b.clone()) == false);

        // Put many more in the channel
        mock.delegate.dispatch_event_internally(evt_a.clone()).expect("Sending event should work!");
        mock.delegate.dispatch_event_internally(evt_a.clone()).expect("Sending event should work!");
        mock.delegate.dispatch_event_internally(evt_a.clone()).expect("Sending event should work!");
        mock.delegate.dispatch_event_internally(evt_a.clone()).expect("Sending event should work!");
        mock.delegate.dispatch_event_internally(evt_a.clone()).expect("Sending event should work!");

        // Check that only the events we expect are there
        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_b.clone()) == false);
        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_b.clone()) == false);
        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_b.clone()) == false);
        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_b.clone()) == false);
        assert!(is_evt_dispatch_event(mock.get_next_event(), state_id, evt_b.clone()) == false);

    }

    #[test]
    fn disconnect_channel()
    {
        let state_id = 0;
        let mut mock = create_mock_delegate(state_id);

        println!("Dropping the channel so tx's fail!");
        drop(mock.mock_rx_channel);

        match mock.delegate.change_state(2) {
            Ok(_) => assert!(false),
            Err(err) => assert!(matches!(err, HSMError::DelegateNotConnected()))
        }

        let evt = DelegateTestEvent::TestA;
        match mock.delegate.dispatch_event_internally(evt) {
            Ok(_) => assert!(false),
            Err(err) => assert!(matches!(err, HSMError::DelegateNotConnected()))
        }
    }

}
