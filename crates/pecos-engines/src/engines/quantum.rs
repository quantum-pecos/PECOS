// PECOS/crates/pecos-engines/src/engines/quantum.rs
use super::QuantumEngine;
use crate::errors::QueueError;
use crate::types::{GateType, MeasurementResult, QuantumCommand};
use log::debug;
use rand::Rng;

pub struct QuantumSimulator;

impl Default for QuantumSimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumSimulator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl QuantumEngine for QuantumSimulator {
    fn process_command(
        &mut self,
        cmd: &QuantumCommand,
    ) -> Result<Option<MeasurementResult>, QueueError> {
        match &cmd.gate {
            GateType::RZ { theta } => {
                debug!(
                    "Processing RZ gate with theta={} on qubit {:?}",
                    theta, cmd.qubits[0]
                );
                Ok(None)
            }
            GateType::R1XY { phi, theta } => {
                debug!(
                    "Processing R1XY gate with phi={}, theta={} on qubit {:?}",
                    phi, theta, cmd.qubits[0]
                );
                Ok(None)
            }
            GateType::SZZ => {
                debug!("Processing SZZ gate on qubits {:?}", cmd.qubits);
                Ok(None)
            }
            GateType::Measure { result_id } => {
                let mut rng = rand::thread_rng(); // Create RNG only when needed // TODO: create once per worker...
                let measurement = rng.gen_range(0..=1);
                debug!(
                    "Generated measurement {} for qubit {:?} (result_id: {})",
                    measurement, cmd.qubits[0], result_id
                );
                Ok(Some(measurement))
            }
        }
    }
}
