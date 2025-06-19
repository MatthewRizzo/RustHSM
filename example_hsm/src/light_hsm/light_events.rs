use rust_hsm::events::StateEventTrait;
use strum::Display;

#[derive(Debug, Display)]
pub enum LightEvents {
    Toggle,
    /// Sets the light to a value from 1-100
    Set(u8),
    TurnOff,
    TurnOn,
    /// Reduces the lighting by a percentage from 1-100 if possible
    ReduceByPercent(u8),
    IncreaseByPercent(u8),
    InvalidNumArgs(usize),
    Invalid,
}

impl StateEventTrait for LightEvents {}
