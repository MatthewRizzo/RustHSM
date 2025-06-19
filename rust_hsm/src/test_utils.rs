///! Contains structs and data useful across the module when running tests
///
use crate::{
    events::StateEventTrait,
    state::{StateContainer, StateIF, StateId, StateTypeTrait},
    state_engine_channel_delegate::{StateEngineDelegate, StateEngineMessages},
};

use serde::{de::value, Deserialize, Serialize};
// https://crates.io/crates/strum_macros
use std::{ops::Add, sync::mpsc::Sender};
use strum::{self, Display};

#[repr(u16)]
#[derive(strum::FromRepr, Clone, PartialEq, Debug, Display)]
pub(crate) enum TestStates {
    INVALID = u16::MAX,
    TOP = 1,
    LevelA1 = 2,
    LevelB1 = 3,
    LevelA2 = 4,
}

// TODO - a lot of this is boiler plate....try to make a macro!

impl Into<u16> for TestStates {
    fn into(self) -> u16 {
        self as u16
    }
}

impl Into<StateId> for TestStates {
    fn into(self) -> StateId {
        StateId::new(self as u16)
    }
}

impl From<u16> for TestStates {
    fn from(state_id: u16) -> Self {
        match Self::from_repr(state_id) {
            Some(val) => val,
            None => Self::INVALID,
        }
    }
}

impl StateTypeTrait for TestStates {}

// State impls

pub struct Top {}
pub struct A1Impl {}
pub struct B1Impl {}
pub struct A2Impl {}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FData {
    x: i32,
}

#[derive(Debug, Display)]
pub enum TestEvents {
    A,
    B(u8),
    C,
    D,
    E(u8),
    F(FData),
    InvalidNumArgs(usize),
    InvalidDeserialize,
    Invalid,
}

impl StateEventTrait for TestEvents {}

pub struct DummyStateStruct<TestStates: StateTypeTrait> {
    state_started: bool,
    data: StateEngineDelegate<TestStates, TestEvents>,
}

impl<TestStates: StateTypeTrait> StateIF<TestStates, TestEvents> for DummyStateStruct<TestStates> {
    fn handle_event(&mut self, event: &TestEvents) -> bool {
        // let extracted_args = event.extract_event_args();
        match event {
            TestEvents::A => true,
            TestEvents::B(_) => true,
            TestEvents::C => true,
            TestEvents::D => true,
            TestEvents::E(_) => true,
            TestEvents::F(_) => true,
            TestEvents::InvalidNumArgs(_) => true,
            TestEvents::InvalidDeserialize => true,
            TestEvents::Invalid => true,
        }
    }

    /// Do not use enter here. What if we are just traversing through the state?
    /// We could get 0 -> 100 -> 50. That is a bad UX!
    /// Only set it to 100 if we are staying here
    fn handle_state_start(&mut self) {
        self.state_started = true;
    }

    fn handle_state_exit(&mut self) {
        self.state_started = false;
    }
}

impl DummyStateStruct<TestStates> {
    fn new(
        num_states_created_counter: &mut u16,
        delegate_tx: Sender<StateEngineMessages<TestEvents>>,
    ) -> Self {
        let data = StateEngineDelegate::<TestStates, TestEvents>::new(
            delegate_tx,
            StateId::new(num_states_created_counter.clone()),
        );
        *num_states_created_counter = num_states_created_counter.add(1);

        Self {
            state_started: false,
            data,
        }
    }
}

pub(crate) fn fill_state_container(
    state_metadata: TestStates,
    num_states_created_counter: &mut u16,
    delegate_tx: Sender<StateEngineMessages<TestEvents>>,
) -> StateContainer<TestStates, TestEvents> {
    let state_struct = DummyStateStruct::new(num_states_created_counter, delegate_tx);

    let container = StateContainer::<TestStates, TestEvents>::new(
        StateId::new(state_metadata.into()),
        Box::new(state_struct),
    );
    container
}

pub(crate) fn cast_id_vector(state_list: &Vec<StateId>) -> Vec<TestStates> {
    let states: Vec<TestStates> = state_list
        .iter()
        .map(|state_id| TestStates::from(*state_id.get_id()))
        .collect();
    states
}
