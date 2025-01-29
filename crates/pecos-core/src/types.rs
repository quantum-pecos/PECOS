// PECOS/crates/pecos-engines/src/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateType {
    RZ { theta: f64 },
    R1XY { phi: f64, theta: f64 },
    SZZ,
    H,
    CX,
    Measure { result_id: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCommand {
    pub gate: GateType,
    pub qubits: Vec<usize>,
}

impl QuantumCommand {
    /// Parses quantum circuit commands from string representation.
    ///
    /// # Format
    /// Commands must follow these formats:
    /// - RZ theta qubit
    /// - R1XY phi theta qubit
    /// - SZZ qubit1 qubit2
    /// - M qubit `result_id`
    ///
    /// All numeric parameters should be valid floating point numbers.
    /// Qubits and `result_id` should be valid integers.
    ///
    /// # Examples
    /// ```
    /// use pecos_core::types::QuantumCommand;
    /// let cmd = QuantumCommand::parse_from_str("RZ 0.5 1").unwrap();
    /// let cmd = QuantumCommand::parse_from_str("R1XY 0.1 0.2 0").unwrap();
    /// let cmd = QuantumCommand::parse_from_str("SZZ 0 1").unwrap();
    /// let cmd = QuantumCommand::parse_from_str("M 0 42").unwrap();
    /// ```
    ///
    /// # Errors
    /// Returns error strings for:
    /// - Wrong number of parameters for command type
    /// - Invalid numeric values for angles/ids
    /// - Unknown command type
    /// - Empty command string
    pub fn parse_from_str(cmd_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        match parts.first() {
            Some(&"RZ") => {
                if parts.len() != 3 {
                    return Err("Invalid RZ format".into());
                }
                Ok(Self {
                    gate: GateType::RZ {
                        theta: parts[1]
                            .parse()
                            .map_err(|e| format!("Invalid theta: {e}"))?,
                    },
                    qubits: vec![parts[2]
                        .parse()
                        .map_err(|e| format!("Invalid qubit: {e}"))?],
                })
            }
            Some(&"R1XY") => {
                if parts.len() != 4 {
                    return Err("Invalid R1XY format".into());
                }
                Ok(Self {
                    gate: GateType::R1XY {
                        phi: parts[1].parse().map_err(|e| format!("Invalid phi: {e}"))?,
                        theta: parts[2]
                            .parse()
                            .map_err(|e| format!("Invalid theta: {e}"))?,
                    },
                    qubits: vec![parts[3]
                        .parse()
                        .map_err(|e| format!("Invalid qubit: {e}"))?],
                })
            }
            Some(&"SZZ") => {
                if parts.len() != 3 {
                    return Err("Invalid SZZ format".into());
                }
                Ok(Self {
                    gate: GateType::SZZ,
                    qubits: vec![
                        parts[1]
                            .parse()
                            .map_err(|e| format!("Invalid qubit1: {e}"))?,
                        parts[2]
                            .parse()
                            .map_err(|e| format!("Invalid qubit2: {e}"))?,
                    ],
                })
            }
            Some(&"H") => {
                if parts.len() != 2 {
                    return Err("Invalid H format".into());
                }
                Ok(Self {
                    gate: GateType::H,
                    qubits: vec![parts[1]
                        .parse()
                        .map_err(|e| format!("Invalid qubit: {e}"))?],
                })
            }
            Some(&"CX") => {
                if parts.len() != 3 {
                    return Err("Invalid CX format".into());
                }
                Ok(Self {
                    gate: GateType::CX,
                    qubits: vec![
                        parts[1]
                            .parse()
                            .map_err(|e| format!("Invalid control qubit: {e}"))?,
                        parts[2]
                            .parse()
                            .map_err(|e| format!("Invalid target qubit: {e}"))?,
                    ],
                })
            }
            Some(&"M") => {
                if parts.len() != 3 {
                    return Err("Invalid M format".into());
                }
                Ok(Self {
                    gate: GateType::Measure {
                        result_id: parts[2]
                            .parse()
                            .map_err(|e| format!("Invalid result_id: {e}"))?,
                    },
                    qubits: vec![parts[1]
                        .parse()
                        .map_err(|e| format!("Invalid qubit: {e}"))?],
                })
            }
            _ => Err(format!(
                "Unknown command type: {}",
                parts.first().unwrap_or(&"<empty>")
            )),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ShotResult {
    pub measurements: HashMap<String, u32>,
}

// For communication
pub type CommandBatch = Vec<QuantumCommand>;
pub type MeasurementResult = u32;

// Statistics
#[derive(Debug)]
pub struct MeasurementStatistics {
    pub total_shots: usize,
    pub per_qubit_stats: HashMap<String, QubitStats>,
}

#[derive(Debug)]
pub struct QubitStats {
    pub zeros: usize,
    pub ones: usize,
}

impl Default for QubitStats {
    fn default() -> Self {
        Self::new()
    }
}

impl QubitStats {
    #[must_use]
    pub fn new() -> Self {
        Self { zeros: 0, ones: 0 }
    }

    pub fn add_measurement(&mut self, value: u32) {
        match value {
            0 => self.zeros += 1,
            1 => self.ones += 1,
            _ => log::warn!("Invalid measurement value: {}", value),
        }
    }

    #[must_use]
    pub fn total_measurements(&self) -> usize {
        self.zeros + self.ones
    }
}

#[derive(Debug, Clone)]
pub struct ShotResults {
    pub shots: Vec<HashMap<String, String>>,
}

impl Default for ShotResults {
    fn default() -> Self {
        Self::new()
    }
}

impl ShotResults {
    #[must_use]
    pub fn new() -> Self {
        Self { shots: Vec::new() }
    }

    #[must_use]
    pub fn from_measurements(results: &[ShotResult]) -> Self {
        let mut shots = Vec::new();

        for shot in results {
            let mut processed_results: HashMap<String, String> = HashMap::new();
            let mut measurement_values = Vec::new();

            let mut keys: Vec<_> = shot.measurements.keys().collect();
            keys.sort();

            for key in &keys {
                if key.starts_with("measurement_") {
                    if let Some(&value) = shot.measurements.get(*key) {
                        measurement_values.push(value.to_string());
                    }
                } else if let Some(&value) = shot.measurements.get(*key) {
                    processed_results.insert((*key).to_string(), value.to_string());
                }
            }

            if !measurement_values.is_empty() {
                processed_results.insert("result".to_string(), measurement_values.concat());
            }

            shots.push(processed_results);
        }

        Self { shots }
    }

    pub fn print(&self) {
        println!("{self}");
    }
}

impl fmt::Display for ShotResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[")?;

        for (i, shot) in self.shots.iter().enumerate() {
            // Get all keys and sort them for consistent output
            let mut keys: Vec<_> = shot.keys().collect();
            keys.sort();

            write!(f, "  {{")?;
            for (j, key) in keys.iter().enumerate() {
                write!(f, "\"{}\": \"{}\"", key, shot.get(*key).unwrap())?;
                if j < keys.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            if i < self.shots.len() - 1 {
                writeln!(f, "}},")?;
            } else {
                writeln!(f, "}}")?;
            }
        }

        write!(f, "]")
    }
}
