//! Contains structs and infrastructure useful for minimal examples in docs (and tests)
use crate::{
    events::StateEventConstraint,
    state::{StateConstraint, StateIF, StateId},
    state_engine_delegate::EngineDelegate,
};

use std::cell::RefCell;

// Start of States Enum //

#[repr(u16)]
#[derive(strum::FromRepr, Clone, PartialEq, Debug, strum::Display)]
pub enum ExampleStates {
    INVALID = u16::MAX,
    Top = 1,
    LevelA1 = 2,
    LevelB1 = 3,
    LevelA2 = 4,
}

impl From<ExampleStates> for u16 {
    fn from(val: ExampleStates) -> Self {
        val as u16
    }
}

impl From<ExampleStates> for StateId {
    fn from(val: ExampleStates) -> Self {
        StateId::new(val as u16)
    }
}

impl From<u16> for ExampleStates {
    fn from(state_id: u16) -> Self {
        match Self::from_repr(state_id) {
            Some(val) => val,
            None => Self::INVALID,
        }
    }
}

impl StateConstraint for ExampleStates {}

// End of States Enum //

// Start of States Data //

pub struct ExampleStateData {
    pub state_entered: bool,
    pub state_exited: bool,
    pub count_a_handled_true: u16,
    pub count_b_handled_true: u16,
    pub count_c_handled_true: u16,
    pub count_d_handled_true: u16,
    pub count_e_handled_true: u16,
    pub count_f_handled_true: u16,
    pub delegate: EngineDelegate<ExampleStates, ExampleEvents>,
}

impl ExampleStateData {
    pub fn new(delegate: EngineDelegate<ExampleStates, ExampleEvents>) -> RefCell<Self> {
        RefCell::new(Self {
            state_entered: false,
            state_exited: false,
            count_a_handled_true: Default::default(),
            count_b_handled_true: Default::default(),
            count_c_handled_true: Default::default(),
            count_d_handled_true: Default::default(),
            count_e_handled_true: Default::default(),
            count_f_handled_true: Default::default(),
            delegate,
        })
    }
}

// End of States Data //

// Start of State definitions //

// Wrap the data in RefCell to reflect the fact that
// 1) Our states might want to mutate their data during handle_event
// 2) Handle_event is const
// 3) The hsm own const references to the states with the expectation that consumers will not directly interact with the states.

pub struct Top {
    pub data: RefCell<ExampleStateData>,
}

pub struct A1Impl {
    pub data: RefCell<ExampleStateData>,
}

pub struct B1Impl {
    pub data: RefCell<ExampleStateData>,
}

pub struct A2Impl {
    pub data: RefCell<ExampleStateData>,
}

// End of State definitions //

// Start of Example Event //
#[derive(Debug, Default, PartialEq)]
pub struct ExampleFData {
    x: i32,
}

#[derive(Debug, strum::Display)]
pub enum ExampleEvents {
    A,
    B(u8),
    C,
    D,
    E(u8),
    F(ExampleFData),
    InvalidNumArgs(usize),
    InvalidDeserialize,
    Invalid,
}

impl StateEventConstraint for ExampleEvents {}

// End of Example Event //

// Start of State Impl //

impl Top {
    pub fn new(delegate: EngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}
impl A1Impl {
    pub fn new(delegate: EngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}
impl B1Impl {
    pub fn new(delegate: EngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}
impl A2Impl {
    pub fn new(delegate: EngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}

// State impls of the StateIF

impl StateIF<ExampleStates, ExampleEvents> for Top {
    fn handle_event(&self, event: &ExampleEvents) -> bool {
        match event {
            ExampleEvents::A => {
                self.data.borrow_mut().count_a_handled_true += 1;
                true
            }
            ExampleEvents::B(_) => {
                self.data.borrow_mut().count_b_handled_true += 1;
                true
            }
            ExampleEvents::C => {
                self.data.borrow_mut().count_c_handled_true += 1;
                true
            }
            ExampleEvents::D => {
                self.data.borrow_mut().count_d_handled_true += 1;
                true
            }
            ExampleEvents::E(_) => {
                self.data.borrow_mut().count_e_handled_true += 1;
                true
            }
            ExampleEvents::F(_) => {
                self.data.borrow_mut().count_f_handled_true += 1;
                true
            }
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    fn handle_state_start(&self) {
        self.data.borrow_mut().state_entered = true;
    }

    fn handle_state_exit(&self) {
        self.data.borrow_mut().state_exited = true;
    }
}

impl StateIF<ExampleStates, ExampleEvents> for A1Impl {
    fn handle_event(&self, event: &ExampleEvents) -> bool {
        // A1 handles all events besides b
        match event {
            ExampleEvents::A => {
                self.data.borrow_mut().count_a_handled_true += 1;
                true
            }
            ExampleEvents::B(_) => false,
            ExampleEvents::C => {
                self.data.borrow_mut().count_c_handled_true += 1;
                true
            }
            ExampleEvents::D => {
                self.data.borrow_mut().count_d_handled_true += 1;
                true
            }
            ExampleEvents::E(_) => {
                self.data.borrow_mut().count_e_handled_true += 1;
                true
            }
            ExampleEvents::F(_) => {
                self.data.borrow_mut().count_f_handled_true += 1;
                true
            }
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    fn handle_state_start(&self) {
        self.data.borrow_mut().state_entered = true;
    }

    fn handle_state_exit(&self) {
        self.data.borrow_mut().state_exited = true;
    }
}

impl StateIF<ExampleStates, ExampleEvents> for B1Impl {
    fn handle_event(&self, event: &ExampleEvents) -> bool {
        // B handles all events besides A
        match event {
            ExampleEvents::A => false,
            ExampleEvents::B(_) => {
                self.data.borrow_mut().count_b_handled_true += 1;
                true
            }
            ExampleEvents::C => {
                self.data.borrow_mut().count_c_handled_true += 1;
                true
            }
            ExampleEvents::D => {
                self.data.borrow_mut().count_d_handled_true += 1;
                true
            }
            ExampleEvents::E(_) => {
                self.data.borrow_mut().count_e_handled_true += 1;
                true
            }
            ExampleEvents::F(_) => {
                self.data.borrow_mut().count_f_handled_true += 1;
                true
            }
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    fn handle_state_start(&self) {
        self.data.borrow_mut().state_entered = true;
    }

    fn handle_state_exit(&self) {
        self.data.borrow_mut().state_exited = true;
    }
}

impl StateIF<ExampleStates, ExampleEvents> for A2Impl {
    fn handle_event(&self, event: &ExampleEvents) -> bool {
        // A2 only handles B. All other events are handled by our parent
        match event {
            ExampleEvents::A => false,
            ExampleEvents::B(_) => {
                self.data.borrow_mut().count_b_handled_true += 1;
                true
            }
            ExampleEvents::C => false,
            ExampleEvents::D => false,
            ExampleEvents::E(_) => false,
            ExampleEvents::F(_) => false,
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    fn handle_state_start(&self) {
        self.data.borrow_mut().state_entered = true;
    }

    fn handle_state_exit(&self) {
        self.data.borrow_mut().state_exited = true;
    }
}

// Start of State Impl //
