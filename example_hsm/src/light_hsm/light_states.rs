// https://crates.io/crates/strum_macros
use strum::{self, Display};

use rust_hsm::state::StateConstraint;

#[repr(u16)]
#[derive(strum::AsRefStr, strum::FromRepr, Display, Clone, PartialEq, Debug)]
pub(crate) enum LightStates {
    INVALID = u16::MAX,
    Top = 1,
    ON = 2,
    OFF = 3,
    DIMMER = 4,
}

// TODO - a lot of this is boiler plate....try to make a macro!
impl From<LightStates> for u16 {
    fn from(val: LightStates) -> Self {
        val as u16
    }
}

impl From<u16> for LightStates {
    fn from(state_id: u16) -> Self {
        match Self::from_repr(state_id) {
            Some(val) => val,
            None => Self::INVALID,
        }
    }
}

impl StateConstraint for LightStates {}
