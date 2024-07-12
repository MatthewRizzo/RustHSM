use rust_hsm::events::{HsmEvent, StateEventsIF};
use strum::Display;

#[repr(u16)]
#[derive(Copy, Clone, strum::FromRepr, Display)]
pub enum LightEvents {
    Toggle = 1,
    /// Sets the light to a value from 1-100
    Set(u8) = 2,
    TurnOff = 3,
    TurnOn = 4,
    /// Reduces the lighting by a percentage from 1-100 if possible
    ReduceByPercent(u8) = 5,
    IncreaseByPercent(u8) = 6,
    InvalidNumArgs(usize) = u16::MAX - 1,
    Invalid = u16::MAX,
}

impl From<&dyn StateEventsIF> for LightEvents {
    fn from(event: &dyn StateEventsIF) -> Self {
        match event.get_event_id() {
            1 => LightEvents::Toggle,
            2 => {
                let setting = event.get_args();
                if setting.len() != 1 {
                    LightEvents::InvalidNumArgs(setting.len())
                } else {
                    LightEvents::Set(*setting.get(0).unwrap())
                }
            }
            3 => LightEvents::TurnOff,
            4 => LightEvents::TurnOn,
            5 => {
                let setting = event.get_args();
                if setting.len() != 1 {
                    LightEvents::InvalidNumArgs(setting.len())
                } else {
                    LightEvents::ReduceByPercent(*setting.get(0).unwrap())
                }
            }
            6 => {
                let setting = event.get_args();
                if setting.len() != 1 {
                    LightEvents::InvalidNumArgs(setting.len())
                } else {
                    LightEvents::IncreaseByPercent(*setting.get(0).unwrap())
                }
            }
            _ => LightEvents::Invalid,
        }
    }
}

impl StateEventsIF for LightEvents {
    fn to_event_base(&self) -> HsmEvent {
        let event_id: u16;
        let mut event_args: Vec<u8> = vec![];

        match self {
            Self::Toggle => event_id = 1,
            Self::Set(value) => {
                event_id = 2;
                event_args.push(value.clone())
            }
            Self::TurnOff => event_id = 3,
            Self::TurnOn => event_id = 4,
            Self::ReduceByPercent(value) => {
                event_id = 5;
                event_args.push(value.clone())
            }
            Self::IncreaseByPercent(value) => {
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
