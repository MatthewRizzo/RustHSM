use thiserror::Error;

pub type HSMResult<T> = std::result::Result<T, HSMError>;

// https://stackoverflow.com/a/62101140/14810215
#[derive(Error, Debug)]
pub enum HSMError {
    #[error("Generic Error")]
    GenericError(String),
    #[error("InvalidStateId Error")]
    InvalidStateId(String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Event Not Implemented Error")]
    EventNotImplemented(String),
}
