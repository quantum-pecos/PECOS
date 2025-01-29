pub mod hybrid;
pub mod phir_engine;
pub mod quantum;

pub use crate::channels::{CommandChannel, MeasurementChannel};
pub use crate::errors::QueueError;
use crate::noise::NoiseModel;
pub use crate::types::{CommandBatch, MeasurementResult, QuantumCommand, ShotResult};
use parking_lot::RwLock;
use std::sync::Arc;

/// Classical engine that processes programs and handles measurements
pub trait ClassicalEngine: Send + Sync {
    fn process_program(&mut self) -> Result<CommandBatch, QueueError>;
    fn handle_measurement(&mut self, measurement: MeasurementResult) -> Result<(), QueueError>;
    fn get_results(&self) -> Result<ShotResult, QueueError>;
    fn compile(&self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Quantum engine that processes commands and generates measurements
pub trait QuantumEngine: Send + Sync {
    fn process_command(
        &mut self,
        cmd: &QuantumCommand,
    ) -> Result<Option<MeasurementResult>, QueueError>;

    fn reset_state(&mut self) -> Result<(), QueueError>;
}

// Base implementation of Hybrid Engine
pub struct HybridEngine<C, M>
where
    C: CommandChannel + Send + Sync + 'static,
    M: MeasurementChannel + Send + Sync + 'static,
{
    classical: Arc<RwLock<Box<dyn ClassicalEngine>>>,
    quantum: Arc<RwLock<Box<dyn QuantumEngine>>>,
    cmd_channel: Arc<RwLock<C>>,
    meas_channel: Arc<RwLock<M>>,
    noise_model: Arc<RwLock<Option<Box<dyn NoiseModel>>>>,
}
