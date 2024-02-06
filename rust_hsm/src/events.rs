///! This file contains the logic behind events that can be used by states
use core::fmt;

/// Most likely, decorating structs will need to impl an event_id -> Enum conversion
#[derive(Eq, PartialEq, Hash)]
pub struct HsmEvent {
    event_id: u16,
    /// event arguments as slices of u8's.
    /// Re-constitute the args on your consumer's end as desired.
    event_args: Vec<u8>,
}

impl HsmEvent {
    pub fn new(event_id: u16, event_args: Vec<u8>) -> HsmEvent {
        HsmEvent {
            event_id,
            event_args,
        }
    }

    pub fn get_event_id(&self) -> u16 {
        self.event_id.clone()
    }

    /// Gets args as slices of u8 buffer. Reconstitute them on your end if needed.
    pub fn get_args(&self) -> Vec<u8> {
        self.event_args.clone()
    }
}

impl fmt::Display for HsmEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event id: {}", self.event_id)
    }
}
