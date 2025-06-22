//! Contains structs and data useful across the module when running tests
///
use crate::{
    examples::ExampleStates,
    examples::*,
    state::{StateConstraint, StateIF, StateId},
    state_engine::HSMEngine,
    state_engine_delegate::delegate_test_utils::MockedDelegate,
};

use log;
use log::LevelFilter;
use std::{cell::RefCell, ops::Add, rc::Rc};

pub struct DummyStateStruct<ExampleStates: StateConstraint> {
    state_started: RefCell<bool>,
    _delegate: MockedDelegate<ExampleStates, ExampleEvents>,
    _num_created: u16,
}

impl<ExampleStates: StateConstraint> StateIF<ExampleStates, ExampleEvents>
    for DummyStateStruct<ExampleStates>
{
    fn handle_event(&self, event: &ExampleEvents) -> bool {
        // let extracted_args = event.extract_event_args();
        match event {
            ExampleEvents::A => true,
            ExampleEvents::B(_) => true,
            ExampleEvents::C => true,
            ExampleEvents::D => true,
            ExampleEvents::E(_) => true,
            ExampleEvents::F(_) => true,
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    /// Do not use enter here. What if we are just traversing through the state?
    /// We could get 0 -> 100 -> 50. That is a bad UX!
    /// Only set it to 100 if we are staying here
    fn handle_state_start(&self) {
        *self.state_started.borrow_mut() = true;
    }

    fn handle_state_exit(&self) {
        *self.state_started.borrow_mut() = false;
    }
}

impl DummyStateStruct<ExampleStates> {
    pub(crate) fn new(num_states_created_counter: &mut u16) -> Box<Self> {
        *num_states_created_counter = num_states_created_counter.add(1);

        Box::new(Self {
            state_started: RefCell::new(false),
            _delegate: MockedDelegate::new(),
            _num_created: num_states_created_counter.clone(),
        })
    }
}

pub(crate) fn cast_id_vector(state_list: &Vec<StateId>) -> Vec<ExampleStates> {
    let states: Vec<ExampleStates> = state_list
        .iter()
        .map(|state_id| ExampleStates::from(*state_id.get_id()))
        .collect();
    states
}

pub fn create_test_hsm() -> Rc<HSMEngine<ExampleStates, ExampleEvents>> {
    let engine = HSMEngine::new("TestHsm".to_string(), LevelFilter::Info).unwrap();
    let top = Top::new(HSMEngine::get_delegate(&engine));
    let a1 = A1Impl::new(HSMEngine::get_delegate(&engine));
    let b1 = B1Impl::new(HSMEngine::get_delegate(&engine));
    let a2 = A2Impl::new(HSMEngine::get_delegate(&engine));

    engine.add_state(top, ExampleStates::Top, None).unwrap();
    engine
        .add_state(a1, ExampleStates::LevelA1, Some(ExampleStates::Top))
        .unwrap();
    engine
        .add_state(b1, ExampleStates::LevelB1, Some(ExampleStates::Top))
        .unwrap();
    engine
        .add_state(a2, ExampleStates::LevelA2, Some(ExampleStates::LevelA1))
        .unwrap();
    engine
}
