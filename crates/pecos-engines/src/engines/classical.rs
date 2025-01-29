use crate::channels::Message;
use crate::errors::QueueError;
use pecos_core::types::{CommandBatch, ShotResult};

/// Classical engine that processes programs and handles measurements
pub trait ClassicalEngine: Send + Sync {
    fn process_program(&mut self) -> Result<CommandBatch, QueueError>;
    fn handle_measurement(&mut self, measurement: Message) -> Result<(), QueueError>;
    fn get_results(&self) -> Result<ShotResult, QueueError>;
    fn compile(&self) -> Result<(), Box<dyn std::error::Error>>;
}
