///! This file contains the logic behind events that can be used by states
use core::fmt;

/// Abstracts common functionality for all state events into the trait.
/// Makes impl of actual enum's easier.
/// You will have to hand-roll the conversion from IF -> concrete enum
/// # Implementing Display
/// This is important for logging of events and how they cause state transitions!
/// The typical format for the write!() should be
/// ```rust
///     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///         match self {
///             Self::Evt1 => write!(f, "Evt1"),
///             Self::EvtWithArgs(args) => write!(f, "EvtWithArgs(formatted_args)"),
///         }
///     }
/// ```
pub trait StateEventsIF: fmt::Display {
    /// Converts the known-event-enum into the concrete event struct
    fn to_event_base(&self) -> HsmEvent;

    /// Gets the id of the event. Useful for avoiding size-issues at compile time
    fn get_event_id(&self) -> u16 {
        self.to_event_base().get_event_id()
    }

    fn get_event_name(&self) -> String;

    /// Gets args as slices of u8 buffer. Reconstitute them on your end if needed.
    /// Can most likely be delegated to HsmEvent.
    fn get_args(&self) -> Vec<u8> {
        self.to_event_base().get_args()
    }
}

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
