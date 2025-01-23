use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateType {
    RZ { theta: f64 },
    RXY { phi: f64, theta: f64 },
    ZZ,
    Measure { result_id: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCommand {
    pub gate: GateType,
    pub qubits: Vec<usize>,
}

impl QuantumCommand {
    // Helper functions to create commands, matching current format
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
            Some(&"RXY") => {
                if parts.len() != 4 {
                    return Err("Invalid RXY format".into());
                }
                Ok(Self {
                    gate: GateType::RXY {
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
            Some(&"ZZ") => {
                if parts.len() != 3 {
                    return Err("Invalid ZZ format".into());
                }
                Ok(Self {
                    gate: GateType::ZZ,
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
