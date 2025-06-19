use crate::state::StateTypeTrait;
use thiserror::Error;

// pub type HSMResult<T> = std::result::Result<T, HSMError>;
pub type HSMResult<T, StateType> = std::result::Result<T, HSMError<StateType>>;

// https://stackoverflow.com/a/62101140/14810215
#[derive(Error, Debug)]
pub enum HSMError<StateType: StateTypeTrait> {
    // pub enum HSMError {
    #[error("Generic Error")]
    GenericError(String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Event Not Implemented Error: {0}")]
    EventNotImplemented(String),
    #[error("Delegate Details Never Set. Use builder!")]
    DelegateDetailsNotSet(),
    #[error("Concrete StateIF never provided!. Use StateBuilder builder correctly!!")]
    ConcreteStateNotProvidedToBuilder(),
    #[error("Controller was never initialized. Make sure to call HSMEngineBuilder  Init before using state-related API's!")]
    ControllerNotInitialized(),
    #[error("State {0} never added to controller! And yet it was made the current state!")]
    InvalidStateId(StateType),
    #[error("Expected State {0} to have parent state with id {1}. But it was never added to controller! Should be impossible")]
    ImpossibleStateMismatch(StateType, StateType),
    #[error("You asked for the LCA between the same nodes!")]
    LCAOfSameNode(),
    #[error("Your delegate is not connected to the StateEngine")]
    DelegateNotConnected(),
    #[error("State with id {0} is not a valid state according to the StateType provided! Did you request a delegate for a non-existent state?")]
    NotAState(u16),
    #[error("State {0} has already been provided a delegate! The same state cannot be allocated >1 delegate!")]
    AlreadyDelegated(StateType),
    #[error("Requesting change state to {0}, but there was already a change state request to {1} while handling {2}" )]
    MultipleConcurrentChangeState(StateType, StateType, String),
}
