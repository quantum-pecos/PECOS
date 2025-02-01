use crate::channels::Message;
use crate::errors::QueueError;
use pecos_core::types::{CommandBatch, ShotResult};

/// Classical engine that processes programs and handles measurements
pub trait ClassicalEngine: Send + Sync {
    /// Processes the classical program and generates a batch of quantum commands
    /// to be sent for execution.
    ///
    /// # Returns
    ///
    /// Returns a `CommandBatch` containing the quantum commands to execute if successful.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `QueueError::OperationError`: If the program processing fails or encounters unsupported operations.
    /// - `QueueError::LockError`: If a lock cannot be acquired during the execution process.
    fn process_program(&mut self) -> Result<CommandBatch, QueueError>;
    /// Handles a measurement received from the quantum engine.
    ///
    /// This function takes a `measurement` message and processes it to update
    /// the state or results of the classical engine.
    ///
    /// # Parameters
    ///
    /// - `measurement`: A `Message` containing the measurement data to process.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `QueueError::OperationError`: If the measurement processing fails or encounters
    ///   unsupported operations.
    /// - `QueueError::LockError`: If a lock cannot be acquired during the measurement handling process.
    fn handle_measurement(&mut self, measurement: Message) -> Result<(), QueueError>;
    /// Retrieves the results of the execution process after all measurements are handled.
    ///
    /// # Returns
    ///
    /// Returns a `ShotResult` containing the measurements and results generated
    /// during the execution process.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `QueueError::OperationError`: If result retrieval fails or is unsupported.
    /// - `QueueError::LockError`: If a lock cannot be acquired to access required resources.
    fn get_results(&self) -> Result<ShotResult, QueueError>;
    /// Compiles the classical program into an intermediate representation or directly
    /// into commands that can be executed by the engine.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the compilation is successful, or an `Err` containing
    /// a boxed error if the compilation fails.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `Box<dyn std::error::Error>`: If there is a compilation error due to syntax issues,
    ///   unsupported features, or internal errors in the engine's implementation.
    fn compile(&self) -> Result<(), Box<dyn std::error::Error>>;
}
