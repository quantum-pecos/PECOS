use serde_json;
use std::error::Error;
use std::sync::PoisonError;
use std::{fmt, io};

/// Custom error type for queue operations
#[derive(Debug)]
pub enum QueueError {
    LockError(String),
    OperationError(String),
    ExecutionError(String),
    SerializationError(String),
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueueError::LockError(msg) => write!(f, "Queue lock error: {}", msg),
            QueueError::OperationError(msg) => write!(f, "Queue operation error: {}", msg),
            QueueError::ExecutionError(msg) => write!(f, "Program execution error: {}", msg),
            QueueError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for QueueError {}

impl From<io::Error> for QueueError {
    fn from(err: io::Error) -> Self {
        QueueError::ExecutionError(err.to_string())
    }
}

impl From<serde_json::Error> for QueueError {
    fn from(err: serde_json::Error) -> Self {
        QueueError::SerializationError(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for QueueError {
    fn from(err: PoisonError<T>) -> Self {
        QueueError::LockError(err.to_string())
    }
}

impl From<Box<dyn Error>> for QueueError {
    fn from(err: Box<dyn Error>) -> Self {
        QueueError::ExecutionError(err.to_string())
    }
}
