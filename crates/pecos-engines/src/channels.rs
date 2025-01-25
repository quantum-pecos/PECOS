// PECOS/crates/pecos-engines/src/channels.rs
use crate::errors::QueueError;
use crate::types::{CommandBatch, MeasurementResult};

pub trait CommandChannel: Send + Sync {
    fn send_commands(&mut self, cmds: CommandBatch) -> Result<(), QueueError>;
    fn flush(&mut self) -> Result<(), QueueError>;
}

pub trait MeasurementChannel: Send + Sync {
    fn receive_measurement(&mut self) -> Result<MeasurementResult, QueueError>;
}

pub mod stdio;

#[cfg(unix)]
pub mod shared_memory;
