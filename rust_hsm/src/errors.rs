use thiserror::Error;

// pub type HSMResult<T> = std::result::Result<T, HSMError>;
pub type HSMResult<T, States> = std::result::Result<T, HSMError<States>>;

// https://stackoverflow.com/a/62101140/14810215
#[derive(Error, Debug)]
pub enum HSMError<StateT> {
    #[error("State {0} with id {1} already added, but is getting added again!")]
    AddDuplicateStateId(StateT, u16),
    #[error("Event Not Implemented Error: {0}")]
    EventNotImplemented(String),
    #[error("StateEngine was never initialized. Make sure to call init before using state-related API's!")]
    EngineNotInitialized(),
    #[error("Generic Error")]
    GenericError(String),
    #[error("Expected State {0} to have parent state with id {1}. But it was never added to controller! Should be impossible")]
    ImpossibleStateMismatch(StateT, StateT),
    #[error("State {0} never added to controller! But requested by {1}!")]
    InvalidStateId(StateT, String),
    #[error("You asked for the LCA between the same nodes!")]
    LCAOfSameNode(),
    #[error("Error validating the relationship between states: {0}")]
    MapValidationError(String),
    #[error("Requesting change state to {0}, but there was already a change state request to {1} while handling {2}" )]
    MultipleConcurrentChangeState(StateT, StateT, String),
    #[error("Reserved State {0} with id {1} as Top, but then added state {2} with id {3} without parents")]
    MultipleTopState(String, u16, String, u16),
}
