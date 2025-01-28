// engines/phir.rs
#![allow(dead_code)] // TODO: Remove once CLI integration is complete

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::ClassicalEngine;
use crate::errors::QueueError;
use crate::types::{CommandBatch, GateType, MeasurementResult, QuantumCommand, ShotResult};

#[derive(Debug, Deserialize)]
struct PHIRProgram {
    format: String,
    version: String,
    metadata: HashMap<String, String>,
    ops: Vec<Operation>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Operation {
    VariableDefinition {
        data: String,
        data_type: String,
        variable: String,
        size: usize,
    },
    QuantumOp {
        qop: String,
        #[serde(default)]
        angles: Option<(Vec<f64>, String)>,
        args: Vec<(String, usize)>,
    },
    ClassicalOp {
        cop: String,
        args: Vec<(String, usize)>,
        returns: Vec<(String, usize)>,
    },
}

// Internal enum for processing operations without borrowing issues
enum ProcessAction {
    VarDef {
        data: String,
        data_type: String,
        variable: String,
        size: usize,
    },
    Quantum {
        qop: String,
        angles: Option<(Vec<f64>, String)>,
        args: Vec<(String, usize)>,
    },
    Classical {
        cop: String,
        args: Vec<(String, usize)>,
        returns: Vec<(String, usize)>,
    },
}

pub struct PHIREngine {
    program: Option<PHIRProgram>,
    current_op: usize,
    measurement_results: HashMap<String, Vec<u32>>,
    pending_commands: Vec<QuantumCommand>,
    quantum_variables: HashMap<String, usize>,
    classical_variables: HashMap<String, (String, usize)>, // (type, size)
}

impl PHIREngine {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let program: PHIRProgram = serde_json::from_str(&content)?;

        // Validate format and version compatibility
        if program.format != "PHIR/JSON" {
            return Err("Invalid format: expected PHIR/JSON".into());
        }

        if program.version != "0.1.0" {
            return Err(format!("Unsupported PHIR version: {}", program.version).into());
        }

        log::debug!("Loading PHIR program with metadata: {:?}", program.metadata);

        Ok(Self {
            program: Some(program),
            current_op: 0,
            measurement_results: HashMap::new(),
            pending_commands: Vec::new(),
            quantum_variables: HashMap::new(),
            classical_variables: HashMap::new(),
        })
    }

    // Create an empty engine without any program
    fn empty() -> Self {
        Self {
            program: None,
            current_op: 0,
            measurement_results: HashMap::new(),
            pending_commands: Vec::new(),
            quantum_variables: HashMap::new(),
            classical_variables: HashMap::new(),
        }
    }

    fn handle_variable_definition(
        &mut self,
        data: &str,
        data_type: &str,
        variable: &str,
        size: usize,
    ) {
        match data {
            "qvar_define" if data_type == "qubits" => {
                self.quantum_variables.insert(variable.to_string(), size);
                log::debug!("Defined quantum variable {} of size {}", variable, size);
            }
            "cvar_define" => {
                self.classical_variables
                    .insert(variable.to_string(), (data_type.to_string(), size));
                log::debug!(
                    "Defined classical variable {} of type {} and size {}",
                    variable,
                    data_type,
                    size
                );
            }
            _ => log::warn!(
                "Unknown variable definition: {} {} {}",
                data,
                data_type,
                variable
            ),
        }
    }

    fn validate_variable_access(&self, var: &str, idx: usize) -> Result<(), QueueError> {
        // Check quantum variables
        if let Some(&size) = self.quantum_variables.get(var) {
            if idx >= size {
                return Err(QueueError::OperationError(format!(
                    "Index {idx} out of bounds for quantum variable {var} of size {size}"
                )));
            }
            return Ok(());
        }

        // Check classical variables
        if let Some((_, size)) = self.classical_variables.get(var) {
            if idx >= *size {
                return Err(QueueError::OperationError(format!(
                    "Index {idx} out of bounds for classical variable {var} of size {size}"
                )));
            }
            return Ok(());
        }

        Err(QueueError::OperationError(format!(
            "Undefined variable: {var}"
        )))
    }

    fn handle_quantum_op(
        &mut self,
        qop: &str,
        angles: &Option<(Vec<f64>, String)>,
        args: &[(String, usize)],
    ) -> Result<bool, QueueError> {
        // Validate all qubit indices first
        for (var, idx) in args {
            self.validate_variable_access(var, *idx)?;
        }

        // Create the command based on operation type
        let cmd = match qop {
            "RZ" => {
                let theta = angles
                    .as_ref()
                    .map(|(angles, _)| angles[0])
                    .ok_or_else(|| QueueError::OperationError("Missing angle for RZ".into()))?;
                QuantumCommand {
                    gate: GateType::RZ { theta },
                    qubits: vec![args[0].1],
                }
            }
            "R1XY" => {
                let (phi, theta) = angles
                    .as_ref()
                    .map(|(angles, _)| (angles[0], angles[1]))
                    .ok_or_else(|| QueueError::OperationError("Missing angles for R1XY".into()))?;
                QuantumCommand {
                    gate: GateType::R1XY { phi, theta },
                    qubits: vec![args[0].1],
                }
            }
            "SZZ" => QuantumCommand {
                gate: GateType::SZZ,
                qubits: vec![args[0].1, args[1].1],
            },
            "H" => QuantumCommand {
                gate: GateType::H,
                qubits: vec![args[0].1],
            },
            "CX" => {
                if args.len() != 2 {
                    return Err(QueueError::OperationError(
                        "CX gate requires control and target qubits".into(),
                    ));
                }
                QuantumCommand {
                    gate: GateType::CX,
                    qubits: vec![args[0].1, args[1].1],
                }
            }
            "Measure" => QuantumCommand {
                gate: GateType::Measure {
                    result_id: self.measurement_results.len(),
                },
                qubits: vec![args[0].1],
            },
            _ => {
                return Err(QueueError::OperationError(format!(
                    "Unknown quantum operation: {qop}"
                )))
            }
        };

        // Add command to pending batch
        self.pending_commands.push(cmd);

        // Return true (indicating we should return commands) when we hit a Result operation
        Ok(false)
    }

    fn handle_classical_op(
        &mut self,
        cop: &str,
        args: &[(String, usize)],
        returns: &[(String, usize)],
    ) -> Result<bool, QueueError> {
        // Validate all variable accesses
        for (var, idx) in args.iter().chain(returns) {
            self.validate_variable_access(var, *idx)?;
        }

        if cop == "Result" {
            let meas_var = &args[0].0;
            let meas_idx = args[0].1;
            let return_var = &returns[0].0;
            let return_idx = returns[0].1;

            log::debug!(
                "Will store measurement {}[{}] in return location {}[{}]",
                meas_var,
                meas_idx,
                return_var,
                return_idx
            );

            // Return true if this is the last Result operation in a sequence
            // We can check this by looking at the next operation
            if let Some(prog) = &self.program {
                let next_op = prog.ops.get(self.current_op + 1);
                match next_op {
                    Some(Operation::ClassicalOp { cop: next_cop, .. }) if next_cop == "Result" => {
                        // More Result operations coming, keep accumulating
                        Ok(false)
                    }
                    _ => {
                        // No more Result operations, flush the batch
                        Ok(true)
                    }
                }
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

impl Default for PHIREngine {
    fn default() -> Self {
        Self::empty()
    }
}

impl ClassicalEngine for PHIREngine {
    fn process_program(&mut self) -> Result<CommandBatch, QueueError> {
        let mut measurement_count = 0;

        loop {
            // First, check if we've reached the end of the program
            let current_op = match &self.program {
                Some(program) if self.current_op < program.ops.len() => {
                    // Clone or copy the necessary data from the current operation
                    match &program.ops[self.current_op] {
                        Operation::VariableDefinition {
                            data,
                            data_type,
                            variable,
                            size,
                        } => ProcessAction::VarDef {
                            data: data.clone(),
                            data_type: data_type.clone(),
                            variable: variable.clone(),
                            size: *size,
                        },
                        Operation::QuantumOp { qop, angles, args } => {
                            if qop == "Measure" {
                                let command = QuantumCommand {
                                    gate: GateType::Measure {
                                        result_id: measurement_count,
                                    },
                                    qubits: vec![args[0].1],
                                };
                                self.pending_commands.push(command);
                                measurement_count += 1;
                                ProcessAction::Quantum {
                                    qop: qop.clone(),
                                    angles: angles.clone(),
                                    args: args.clone(),
                                }
                            } else {
                                ProcessAction::Quantum {
                                    qop: qop.clone(),
                                    angles: angles.clone(),
                                    args: args.clone(),
                                }
                            }
                        }
                        Operation::ClassicalOp { cop, args, returns } => ProcessAction::Classical {
                            cop: cop.clone(),
                            args: args.clone(),
                            returns: returns.clone(),
                        },
                    }
                }
                _ => {
                    // End of program, return any remaining commands
                    return Ok(std::mem::take(&mut self.pending_commands));
                }
            };

            // Process the operation
            let should_return = match current_op {
                ProcessAction::VarDef {
                    data,
                    data_type,
                    variable,
                    size,
                } => {
                    self.handle_variable_definition(&data, &data_type, &variable, size);
                    Ok(false)
                }
                ProcessAction::Quantum { qop, angles, args } => {
                    if qop != "Measure" {
                        self.handle_quantum_op(&qop, &angles, &args)
                    } else {
                        Ok(false) // Already handled in the match above
                    }
                }
                ProcessAction::Classical { cop, args, returns } => {
                    self.handle_classical_op(&cop, &args, &returns)
                }
            }?;

            // Increment the operation counter
            self.current_op += 1;

            // If we should return and we have pending commands, return them
            if should_return && !self.pending_commands.is_empty() {
                return Ok(std::mem::take(&mut self.pending_commands));
            }
        }
    }

    fn handle_measurement(&mut self, measurement: MeasurementResult) -> Result<(), QueueError> {
        let result_id = self.measurement_results.len();
        self.measurement_results
            .insert(format!("measurement_{result_id}"), vec![measurement]);
        Ok(())
    }

    fn get_results(&self) -> Result<ShotResult, QueueError> {
        let mut results = ShotResult::default();

        // Sort keys to ensure consistent ordering
        let mut keys: Vec<_> = self.measurement_results.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(measurements) = self.measurement_results.get(key) {
                if let Some(&value) = measurements.first() {
                    results.measurements.insert(key.clone(), value);
                }
            }
        }
        Ok(results)
    }

    fn compile(&self) -> Result<(), Box<dyn std::error::Error>> {
        // No compilation needed for PHIR/JSON
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_phir_engine_basic() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let program_path = dir.path().join("test.json");

        // Create a test program
        let program = r#"{
            "format": "PHIR/JSON",
            "version": "0.1.0",
            "metadata": {"test": "true"},
            "ops": [
                {
                    "data": "qvar_define",
                    "data_type": "qubits",
                    "variable": "q",
                    "size": 2
                },
                {
                    "data": "cvar_define",
                    "data_type": "i64",
                    "variable": "m",
                    "size": 2
                },
                {
                    "qop": "R1XY",
                    "angles": [[0.1, 0.2], "rad"],
                    "args": [["q", 0]]
                },
                {
                    "qop": "Measure",
                    "args": [["q", 0]],
                    "returns": [["m", 0]]
                },
                {"cop": "Result", "args": [["m", 0]], "returns": [["result", 0]]}
            ]
        }"#;

        let mut file = File::create(&program_path)?;
        file.write_all(program.as_bytes())?;

        let mut engine = PHIREngine::new(&program_path)?;

        // Process program and verify commands
        let commands = engine.process_program()?;
        assert_eq!(commands.len(), 2);

        // Test measurement handling
        engine.handle_measurement(1)?;

        // Verify results
        let results = engine.get_results()?;
        assert_eq!(results.measurements.len(), 1);
        assert_eq!(results.measurements["m_0"], 1);

        Ok(())
    }
}
