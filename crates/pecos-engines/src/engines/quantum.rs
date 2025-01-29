// PECOS/crates/pecos-engines/src/engines/quantum.rs
use crate::channels::Message;
use crate::errors::QueueError;
use log::debug;
use pecos_core::types::{GateType, QuantumCommand};
use pecos_qsim::{ArbitraryRotationGateable, CliffordGateable, QuantumSimulator};

/// Quantum engine that processes commands and generates measurements
pub trait QuantumEngine: Send + Sync {
    /// Processes a single quantum command and optionally generates a measurement.
    ///
    /// # Parameters
    ///
    /// * `cmd` - The quantum command to be processed, containing the gate type and qubit indices.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Message))` if a measurement is generated.
    /// * `Ok(None)` if no measurement is generated.
    /// * `Err(QueueError)` if an error occurs during processing.
    ///
    /// # Errors
    ///
    /// * [`QueueError::OperationError`] - If the operation is not supported by the simulator.
    /// * [`QueueError::ExecutionError`] - If the simulator fails during command execution.
    fn process_command(&mut self, cmd: &QuantumCommand) -> Result<Option<Message>, QueueError>;

    /// Resets the internal state of the quantum engine to its initial state.
    ///
    /// # Errors
    ///
    /// Returns a [`QueueError`] if the underlying quantum simulator fails
    /// to reset its state.
    fn reset_state(&mut self) -> Result<(), QueueError>;
}

// Engine for simulators that only support Clifford gates
pub struct CliffordEngine<S>
where
    S: QuantumSimulator + CliffordGateable<usize> + Send + Sync,
{
    simulator: S,
}

impl<S> CliffordEngine<S>
where
    S: QuantumSimulator + CliffordGateable<usize> + Send + Sync,
{
    pub fn new(simulator: S) -> Self {
        Self { simulator }
    }
}

impl<S> QuantumEngine for CliffordEngine<S>
where
    S: QuantumSimulator + CliffordGateable<usize> + Send + Sync,
{
    fn process_command(&mut self, cmd: &QuantumCommand) -> Result<Option<Message>, QueueError> {
        match &cmd.gate {
            GateType::H => {
                debug!("Processing H gate on qubit {:?}", cmd.qubits[0]);
                self.simulator.h(cmd.qubits[0]);
                Ok(None)
            }
            GateType::CX => {
                debug!(
                    "Processing CX gate with control {:?} and target {:?}",
                    cmd.qubits[0], cmd.qubits[1]
                );
                self.simulator.cx(cmd.qubits[0], cmd.qubits[1]);
                Ok(None)
            }
            GateType::SZZ => {
                debug!("Processing SZZ gate on qubits {:?}", cmd.qubits);
                self.simulator.szz(cmd.qubits[0], cmd.qubits[1]);
                Ok(None)
            }
            GateType::Measure { result_id: _ } => {
                let result = self.simulator.mz(cmd.qubits[0]);
                let measurement = u32::from(result.outcome);
                debug!(
                    "Generated measurement {} for qubit {:?}",
                    measurement, cmd.qubits[0]
                );
                Ok(Some(measurement))
            }
            GateType::RZ { .. } | GateType::R1XY { .. } => Err(QueueError::OperationError(
                "This simulator only supports Clifford operations".into(),
            )),
        }
    }

    fn reset_state(&mut self) -> Result<(), QueueError> {
        self.simulator.reset(); // Assuming this method exists in your simulator
        Ok(())
    }
}

// Engine for simulators that support arbitrary rotations
pub struct FullEngine<S>
where
    S: QuantumSimulator + CliffordGateable<usize> + ArbitraryRotationGateable<usize> + Send + Sync,
{
    simulator: S,
}

impl<S> FullEngine<S>
where
    S: QuantumSimulator + CliffordGateable<usize> + ArbitraryRotationGateable<usize> + Send + Sync,
{
    pub fn new(simulator: S) -> Self {
        Self { simulator }
    }
}

impl<S> QuantumEngine for FullEngine<S>
where
    S: QuantumSimulator + CliffordGateable<usize> + ArbitraryRotationGateable<usize> + Send + Sync,
{
    fn process_command(&mut self, cmd: &QuantumCommand) -> Result<Option<Message>, QueueError> {
        match &cmd.gate {
            GateType::H => {
                debug!("Processing H gate on qubit {:?}", cmd.qubits[0]);
                self.simulator.h(cmd.qubits[0]);
                Ok(None)
            }
            GateType::CX => {
                debug!(
                    "Processing CX gate with control {:?} and target {:?}",
                    cmd.qubits[0], cmd.qubits[1]
                );
                self.simulator.cx(cmd.qubits[0], cmd.qubits[1]);
                Ok(None)
            }
            GateType::SZZ => {
                debug!("Processing SZZ gate on qubits {:?}", cmd.qubits);
                self.simulator.szz(cmd.qubits[0], cmd.qubits[1]);
                Ok(None)
            }
            GateType::Measure { result_id: _ } => {
                let result = self.simulator.mz(cmd.qubits[0]);
                let measurement = u32::from(result.outcome);
                debug!(
                    "Generated measurement {} for qubit {:?}",
                    measurement, cmd.qubits[0]
                );
                Ok(Some(measurement))
            }
            GateType::RZ { theta } => {
                debug!(
                    "Processing RZ gate with theta={} on qubit {:?}",
                    theta, cmd.qubits[0]
                );
                self.simulator.rz(*theta, cmd.qubits[0]);
                Ok(None)
            }
            GateType::R1XY { phi, theta } => {
                debug!(
                    "Processing R1XY gate with phi={}, theta={} on qubit {:?}",
                    phi, theta, cmd.qubits[0]
                );
                self.simulator.r1xy(*theta, *phi, cmd.qubits[0]);
                Ok(None)
            }
        }
    }

    fn reset_state(&mut self) -> Result<(), QueueError> {
        self.simulator.reset(); // Assuming this method exists in your simulator
        Ok(())
    }
}

// Factory function to create the appropriate engine based on simulator type
pub fn new_quantum_engine<S>(simulator: S) -> Box<dyn QuantumEngine>
where
    S: QuantumSimulator + CliffordGateable<usize> + Send + Sync + 'static,
{
    Box::new(CliffordEngine::new(simulator))
}

pub fn new_quantum_engine_full<S>(simulator: S) -> Box<dyn QuantumEngine>
where
    S: QuantumSimulator
        + CliffordGateable<usize>
        + ArbitraryRotationGateable<usize>
        + Send
        + Sync
        + 'static,
{
    Box::new(FullEngine::new(simulator))
}
