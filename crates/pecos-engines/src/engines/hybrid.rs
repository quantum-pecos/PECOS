// PECOS/crates/pecos-engines/src/engines/hybrid.rs
use log::{debug, info};
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use std::sync::Arc;

use super::{ClassicalEngine, QuantumEngine};
use crate::channels::{CommandChannel, MessageChannel};
use crate::errors::QueueError;
use pecos_core::types::{GateType, ShotResult, ShotResults};
use pecos_noise::NoiseModel;

// Base implementation of Hybrid Engine
pub struct HybridEngine<C, M>
where
    C: CommandChannel + Send + Sync + 'static,
    M: MessageChannel + Send + Sync + 'static,
{
    classical: Arc<RwLock<Box<dyn ClassicalEngine>>>,
    quantum: Arc<RwLock<Box<dyn QuantumEngine>>>,
    cmd_channel: Arc<RwLock<C>>,
    meas_channel: Arc<RwLock<M>>,
    noise_model: Arc<RwLock<Option<Box<dyn NoiseModel>>>>,
}

impl<C, M> HybridEngine<C, M>
where
    C: CommandChannel + Send + Sync + 'static + Clone,
    M: MessageChannel + Send + Sync + 'static + Clone,
{
    pub fn new(
        classical: Box<dyn ClassicalEngine>,
        quantum: Box<dyn QuantumEngine>,
        cmd_channel: C,
        meas_channel: M,
    ) -> Self {
        Self {
            classical: Arc::new(RwLock::new(classical)),
            quantum: Arc::new(RwLock::new(quantum)),
            cmd_channel: Arc::new(RwLock::new(cmd_channel)),
            meas_channel: Arc::new(RwLock::new(meas_channel)),
            noise_model: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_noise_model(&self, noise_model: Option<Box<dyn NoiseModel>>) {
        *self.noise_model.write() = noise_model;
    }

    /// Executes a single quantum circuit shot and returns the result.
    ///
    /// This function performs the following steps:
    /// 1. Retrieves quantum commands from the classical engine.
    /// 2. Sends these commands to the quantum engine via the command channel.
    /// 3. Processes measurement results received from the measurement channel.
    /// 4. Retrieves and returns the final results from the classical engine.
    ///
    /// # Returns
    ///
    /// Returns a `ShotResult` representing the results of the shot execution.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `QueueError::LockError`: If a lock cannot be acquired for a resource.
    /// - `QueueError::OperationError`: If an operation is not supported or fails.
    /// - `QueueError::ExecutionError`: If there is a problem executing quantum or classical parts.
    /// - `QueueError::SerializationError`: If there is an issue with serializing or deserializing data.
    pub fn run_shot(&self) -> Result<ShotResult, QueueError> {
        // Get commands from classical engine
        let commands = self.classical.write().process_program()?;
        debug!("Generated {} commands", commands.len());

        // Send commands through channel
        self.cmd_channel.write().send_commands(commands)?;

        // Process measurements
        let measurement = self.meas_channel.write().receive_message()?;
        self.classical.write().handle_measurement(measurement)?;

        // Get final results
        self.classical.read().get_results()
    }

    /// Runs a parallel execution of quantum circuits for a specified number of shots.
    ///
    /// # Parameters
    ///
    /// - `shots`: The total number of shots to execute in parallel.
    /// - `workers`: The number of workers to use for parallel execution.
    ///
    /// # Returns
    ///
    /// Returns a `ShotResults` object containing the processed results for all shots,
    /// or a `QueueError` if an error occurs during execution.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    /// - `QueueError::OperationError` if an operation is not supported.
    /// - `QueueError::ExecutionError` if the quantum engine execution fails.
    /// - `QueueError::LockError` if there is a failure in acquiring or unwrapping a lock.
    pub fn run_parallel(&self, shots: usize, workers: usize) -> Result<ShotResults, QueueError> {
        info!(
            "Starting parallel execution with {} shots and {} workers",
            shots, workers
        );

        let shot_results = Arc::new(Mutex::new(Vec::with_capacity(shots)));

        // Get commands just once from classical engine
        let base_commands = {
            let mut classical = self.classical.write();
            let cmds = classical.process_program()?;
            debug!("Generated base commands: {:?}", cmds);
            cmds
        };

        // Get noise model reference outside the loop
        let noise_model = self.noise_model.read();

        (0..shots)
            .into_par_iter()
            .try_for_each::<_, Result<(), QueueError>>(|shot_idx| {
                debug!("Starting shot {}", shot_idx);
                let mut shot_result = ShotResult::default();

                // Clone the base commands for this shot
                let mut commands = base_commands.clone();

                // Apply noise model independently for this shot
                if let Some(noise_model) = &*noise_model {
                    commands = noise_model.apply_noise(commands);
                    debug!(
                        "Applied noise model for shot {}, commands: {:?}",
                        shot_idx, commands
                    );
                }

                // Process commands through quantum engine
                {
                    let mut quantum = self.quantum.write();
                    // Reset quantum state before processing this shot
                    quantum.reset_state()?;

                    for cmd in &commands {
                        if let Some(measurement) = quantum.process_command(cmd)? {
                            let GateType::Measure { result_id: res_id } = cmd.gate else {
                                continue;
                            };
                            shot_result
                                .measurements
                                .insert(format!("measurement_{res_id}"), measurement);
                        }
                    }
                }

                shot_results.lock().push(shot_result);
                debug!("Completed shot {}", shot_idx);
                Ok(())
            })?;

        let mutex = Arc::try_unwrap(shot_results)
            .map_err(|_| QueueError::LockError("Could not unwrap results".into()))?;

        let raw_results = mutex.into_inner();

        // Convert to our new ShotResults type
        let results = ShotResults::from_measurements(&raw_results);

        // Print results
        results.print();

        Ok(results)
    }
}
