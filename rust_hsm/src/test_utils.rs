///! Contains structs and data useful across the module when running tests
///
use crate::{
    examples::*,
    state::{StateContainer, StateIF, StateId, StateTypeTrait},
    state_engine::{HSMEngine, HSMEngineBuilder, HSMInterface},
    state_engine_channel_delegate::{StateEngineDelegate, StateEngineMessages},
};

use log;
use tokio::sync::mpsc::UnboundedSender;

use std::{ops::Add, sync::mpsc::Sender};

pub struct DummyStateStruct<ExampleStates: StateTypeTrait> {
    state_started: bool,
    _data: StateEngineDelegate<ExampleStates, ExampleEvents>,
}

impl<ExampleStates: StateTypeTrait> StateIF<ExampleStates, ExampleEvents>
    for DummyStateStruct<ExampleStates>
{
    fn handle_event(&mut self, event: &ExampleEvents) -> bool {
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
    fn handle_state_start(&mut self) {
        self.state_started = true;
    }

    fn handle_state_exit(&mut self) {
        self.state_started = false;
    }
}

impl DummyStateStruct<ExampleStates> {
    fn new(
        num_states_created_counter: &mut u16,
        delegate_tx: UnboundedSender<StateEngineMessages<ExampleStates, ExampleEvents>>,
    ) -> Self {
        let data = StateEngineDelegate::<ExampleStates, ExampleEvents>::new(
            delegate_tx,
            StateId::new(num_states_created_counter.clone()),
            log::LevelFilter::Debug,
        );
        *num_states_created_counter = num_states_created_counter.add(1);

        Self {
            state_started: false,
            _data: data,
        }
    }
}

pub(crate) fn fill_state_container(
    state_metadata: ExampleStates,
    num_states_created_counter: &mut u16,
    delegate_tx: UnboundedSender<StateEngineMessages<ExampleStates, ExampleEvents>>,
) -> StateContainer<ExampleStates, ExampleEvents> {
    let state_struct = DummyStateStruct::new(num_states_created_counter, delegate_tx);

    let container = StateContainer::<ExampleStates, ExampleEvents>::new(
        StateId::new(state_metadata.into()),
        Box::new(state_struct),
    );
    container
}

pub(crate) fn cast_id_vector(state_list: &Vec<StateId>) -> Vec<ExampleStates> {
    let states: Vec<ExampleStates> = state_list
        .iter()
        .map(|state_id| ExampleStates::from(*state_id.get_id()))
        .collect();
    states
}

/// Builds an hsm and implicitly tests the builder!
pub(crate) fn build_test_hsm(
    initial_state: ExampleStates,
) -> HSMInterface<ExampleStates, ExampleEvents> {
    let engine_builder: HSMEngineBuilder<ExampleStates, ExampleEvents> = HSMEngineBuilder::new(
        "TestHsm".to_string(),
        ExampleStates::Top as u16,
        log::LevelFilter::Info,
        log::LevelFilter::Debug,
        log::LevelFilter::Debug,
    );

    let top_delegate = engine_builder
        .create_delegate(ExampleStates::Top.into())
        .expect("");
    let a1_delegate = engine_builder
        .create_delegate(ExampleStates::LevelA1.into())
        .expect("");
    let b1_delegate = engine_builder
        .create_delegate(ExampleStates::LevelB1.into())
        .expect("");
    let a2_delegate = engine_builder
        .create_delegate(ExampleStates::LevelA2.into())
        .expect("");

    let state_top = Top::new(top_delegate);
    let state_a1_impl = A1Impl::new(a1_delegate);
    let state_b1_impl = B1Impl::new(b1_delegate);
    let state_a2_impl = A2Impl::new(a2_delegate);

    let hsm_interface = engine_builder
        .add_state(state_top, ExampleStates::Top, None)
        .add_state(
            state_a1_impl,
            ExampleStates::LevelA1,
            Some(ExampleStates::Top),
        )
        .add_state(
            state_b1_impl,
            ExampleStates::LevelB1,
            Some(ExampleStates::Top),
        )
        .add_state(
            state_a2_impl,
            ExampleStates::LevelA2,
            Some(ExampleStates::LevelA1),
        )
        .init(initial_state as u16)
        .unwrap();

    hsm_interface
}
