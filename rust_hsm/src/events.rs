///! This file contains the logic behind events that can be used by states

/// Abstracts common functionality for all state events into the trait.
/// Makes impl of actual enum's easier.
pub trait StateEventTrait: std::fmt::Display {
    fn get_event_name(&self) -> String {
        format!("{}", self)
    }
}
