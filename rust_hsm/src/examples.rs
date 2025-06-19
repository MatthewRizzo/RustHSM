///! Contains structs and infrastructure useful for minimal examples in docs (and tests)
use crate::{
    events::StateEventTrait,
    state::{StateIF, StateId, StateTypeTrait},
    state_engine_channel_delegate::StateEngineDelegate,
};

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

impl Into<u16> for ExampleStates {
    fn into(self) -> u16 {
        self as u16
    }
}

impl Into<StateId> for ExampleStates {
    fn into(self) -> StateId {
        StateId::new(self as u16)
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

impl StateTypeTrait for ExampleStates {}

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
    pub delegate: StateEngineDelegate<ExampleStates, ExampleEvents>,
}

impl ExampleStateData {
    pub fn new(delegate: StateEngineDelegate<ExampleStates, ExampleEvents>) -> Self {
        Self {
            state_entered: false,
            state_exited: false,
            count_a_handled_true: Default::default(),
            count_b_handled_true: Default::default(),
            count_c_handled_true: Default::default(),
            count_d_handled_true: Default::default(),
            count_e_handled_true: Default::default(),
            count_f_handled_true: Default::default(),
            delegate,
        }
    }
}

// End of States Data //

// Start of State definitions //

pub struct Top {
    pub data: ExampleStateData,
}

pub struct A1Impl {
    pub data: ExampleStateData,
}

pub struct B1Impl {
    pub data: ExampleStateData,
}

pub struct A2Impl {
    pub data: ExampleStateData,
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

impl StateEventTrait for ExampleEvents {}

// End of Example Event //

// Start of State Impl //

impl Top {
    pub fn new(delegate: StateEngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}
impl A1Impl {
    pub fn new(delegate: StateEngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}
impl B1Impl {
    pub fn new(delegate: StateEngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}
impl A2Impl {
    pub fn new(delegate: StateEngineDelegate<ExampleStates, ExampleEvents>) -> Box<Self> {
        Box::new(Self {
            data: ExampleStateData::new(delegate),
        })
    }
}

// State impls of the StateIF

impl StateIF<ExampleStates, ExampleEvents> for Top {
    fn handle_event(&mut self, event: &ExampleEvents) -> bool {
        match event {
            ExampleEvents::A => {
                self.data.count_a_handled_true += 1;
                true
            }
            ExampleEvents::B(_) => {
                self.data.count_b_handled_true += 1;
                true
            }
            ExampleEvents::C => {
                self.data.count_c_handled_true += 1;
                true
            }
            ExampleEvents::D => {
                self.data.count_d_handled_true += 1;
                true
            }
            ExampleEvents::E(_) => {
                self.data.count_e_handled_true += 1;
                true
            }
            ExampleEvents::F(_) => {
                self.data.count_f_handled_true += 1;
                true
            }
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    fn handle_state_start(&mut self) {
        self.data.state_entered = true;
    }

    fn handle_state_exit(&mut self) {
        self.data.state_exited = true;
    }
}

impl StateIF<ExampleStates, ExampleEvents> for A1Impl {
    fn handle_event(&mut self, event: &ExampleEvents) -> bool {
        // A1 handles all events besides b
        match event {
            ExampleEvents::A => {
                self.data.count_a_handled_true += 1;
                true
            }
            ExampleEvents::B(_) => false,
            ExampleEvents::C => {
                self.data.count_c_handled_true += 1;
                true
            }
            ExampleEvents::D => {
                self.data.count_d_handled_true += 1;
                true
            }
            ExampleEvents::E(_) => {
                self.data.count_e_handled_true += 1;
                true
            }
            ExampleEvents::F(_) => {
                self.data.count_f_handled_true += 1;
                true
            }
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    fn handle_state_start(&mut self) {
        self.data.state_entered = true;
    }

    fn handle_state_exit(&mut self) {
        self.data.state_exited = true;
    }
}

impl StateIF<ExampleStates, ExampleEvents> for B1Impl {
    fn handle_event(&mut self, event: &ExampleEvents) -> bool {
        // B handles all events besides A
        match event {
            ExampleEvents::A => {
                return false;
            }
            ExampleEvents::B(_) => {
                self.data.count_b_handled_true += 1;
                true
            }
            ExampleEvents::C => {
                self.data.count_c_handled_true += 1;
                true
            }
            ExampleEvents::D => {
                self.data.count_d_handled_true += 1;
                true
            }
            ExampleEvents::E(_) => {
                self.data.count_e_handled_true += 1;
                true
            }
            ExampleEvents::F(_) => {
                self.data.count_f_handled_true += 1;
                true
            }
            ExampleEvents::InvalidNumArgs(_) => true,
            ExampleEvents::InvalidDeserialize => true,
            ExampleEvents::Invalid => true,
        }
    }

    fn handle_state_start(&mut self) {
        self.data.state_entered = true;
    }

    fn handle_state_exit(&mut self) {
        self.data.state_exited = true;
    }
}

impl StateIF<ExampleStates, ExampleEvents> for A2Impl {
    fn handle_event(&mut self, event: &ExampleEvents) -> bool {
        // A2 only handles B. All other events are handled by our parent
        match event {
            ExampleEvents::A => false,
            ExampleEvents::B(_) => {
                self.data.count_b_handled_true += 1;
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

    fn handle_state_start(&mut self) {
        self.data.state_entered = true;
    }

    fn handle_state_exit(&mut self) {
        self.data.state_exited = true;
    }
}

// Start of State Impl //
