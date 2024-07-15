///! Contains structs and data useful across the module when running tests
///
use crate::{
    events::{HsmEvent, StateEventsIF, StatefulEvent},
    state::{StateContainer, StateIF, StateId, StateTypeTrait},
    state_engine_channel_delegate::{StateEngineDelegate, StateEngineMessages},
};
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

fn build_test_delegate() -> StateEngineDelegate<TestStates> {
    todo!()
}

pub struct Top {}
pub struct A1Impl {}
pub struct B1Impl {}
pub struct A2Impl {}

#[repr(u16)]
#[derive(Copy, Clone, strum::FromRepr, Debug, Display)]
pub enum TestEvents {
    A = 1,
    /// Sets the light to a value from 1-100
    B(u8) = 2,
    C = 3,
    D = 4,
    E(u8) = 5,
    F(u8) = 6,
    InvalidNumArgs(usize) = u16::MAX - 1,
    Invalid = u16::MAX,
}

impl From<&dyn StateEventsIF> for TestEvents {
    fn from(event: &dyn StateEventsIF) -> Self {
        match event.get_event_id() {
            1 => TestEvents::A,
            2 => {
                let setting = event.get_args();
                if setting.len() != 1 {
                    TestEvents::InvalidNumArgs(setting.len())
                } else {
                    TestEvents::B(*setting.get(0).unwrap())
                }
            }
            3 => TestEvents::C,
            4 => TestEvents::D,
            5 => {
                let setting = event.get_args();
                if setting.len() != 1 {
                    TestEvents::InvalidNumArgs(setting.len())
                } else {
                    TestEvents::E(*setting.get(0).unwrap())
                }
            }
            6 => {
                let setting = event.get_args();
                if setting.len() != 1 {
                    TestEvents::InvalidNumArgs(setting.len())
                } else {
                    TestEvents::F(*setting.get(0).unwrap())
                }
            }
            _ => TestEvents::Invalid,
        }
    }
}

impl StateEventsIF for TestEvents {
    fn to_event_base(&self) -> HsmEvent {
        let event_id: u16;
        let mut event_args: Vec<u8> = vec![];

        match self {
            Self::A => event_id = 1,
            Self::B(value) => {
                event_id = 2;
                event_args.push(value.clone())
            }
            Self::C => event_id = 3,
            Self::D => event_id = 4,
            Self::E(value) => {
                event_id = 5;
                event_args.push(value.clone())
            }
            Self::F(value) => {
                event_id = 6;
                event_args.push(value.clone())
            }
            Self::InvalidNumArgs(_usize) => {
                event_id = 7;
            }
            Self::Invalid => event_id = 8,
        }

        HsmEvent::new(event_id, event_args)
    }
}

pub struct DummyStateStruct<TestStates: StateTypeTrait> {
    state_started: bool,
    data: StateEngineDelegate<TestStates>,
}

impl<TestStates: StateTypeTrait> StateIF<TestStates> for DummyStateStruct<TestStates> {
    fn handle_event(&mut self, event: &StatefulEvent) -> bool {
        let events: TestEvents = TestEvents::from(event);
        match events {
            _ => false,
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
    fn new(num_states_created_counter: &mut u16, delegate_tx: Sender<StateEngineMessages>) -> Self {
        let data = StateEngineDelegate::<TestStates>::new(
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
    delegate_tx: Sender<StateEngineMessages>,
) -> StateContainer<TestStates> {
    let state_struct = DummyStateStruct::new(num_states_created_counter, delegate_tx);

    let container = StateContainer::<TestStates>::new(
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
