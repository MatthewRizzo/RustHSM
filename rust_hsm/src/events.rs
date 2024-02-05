///! This file contains the logic behind events that can be used by states
use core::fmt;

/// Most likely, decorating structs will need to impl an event_id -> Enum conversion
#[derive(Eq, PartialEq, Hash)]
pub struct DecoratableEventBase {
    event_id: u16,
}

impl DecoratableEventBase {
    pub fn new(event_id: u16) -> DecoratableEventBase {
        DecoratableEventBase { event_id }
    }
}

impl fmt::Display for DecoratableEventBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event id: {}", self.event_id)
    }
}
