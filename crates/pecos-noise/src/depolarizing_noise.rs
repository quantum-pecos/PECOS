use crate::noise_model::NoiseModel;
use parking_lot::Mutex;
use pecos_core::types::{CommandBatch, GateType, QuantumCommand};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::sync::Arc;

/// Simple depolarizing noise model that applies random Pauli errors
pub struct DepolarizingNoise {
    /// Probability of applying a noise operation after each gate
    probability: f64,
    /// Shared random number generator
    rng: Arc<Mutex<StdRng>>,
}

impl DepolarizingNoise {
    #[must_use]
    pub fn new(probability: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&probability),
            "Probability must be between 0 and 1"
        );
        Self {
            probability,
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        }
    }

    /// Helper to create sequence of gates for Pauli X
    fn x_gates(qubit: usize) -> Vec<QuantumCommand> {
        vec![
            QuantumCommand {
                gate: GateType::RZ {
                    theta: std::f64::consts::PI,
                },
                qubits: vec![qubit],
            },
            QuantumCommand {
                gate: GateType::H,
                qubits: vec![qubit],
            },
            QuantumCommand {
                gate: GateType::RZ {
                    theta: std::f64::consts::PI,
                },
                qubits: vec![qubit],
            },
            QuantumCommand {
                gate: GateType::H,
                qubits: vec![qubit],
            },
        ]
    }

    /// Helper to create sequence of gates for Pauli Y
    fn y_gates(qubit: usize) -> Vec<QuantumCommand> {
        vec![
            QuantumCommand {
                gate: GateType::RZ {
                    theta: std::f64::consts::PI,
                },
                qubits: vec![qubit],
            },
            QuantumCommand {
                gate: GateType::H,
                qubits: vec![qubit],
            },
            QuantumCommand {
                gate: GateType::RZ {
                    theta: std::f64::consts::FRAC_PI_2,
                },
                qubits: vec![qubit],
            },
            QuantumCommand {
                gate: GateType::H,
                qubits: vec![qubit],
            },
        ]
    }

    /// Helper to create Pauli Z gate
    fn z_gate(qubit: usize) -> QuantumCommand {
        QuantumCommand {
            gate: GateType::RZ {
                theta: std::f64::consts::PI,
            },
            qubits: vec![qubit],
        }
    }
}

impl NoiseModel for DepolarizingNoise {
    fn apply_noise(&self, commands: CommandBatch) -> CommandBatch {
        let mut noisy_commands = Vec::new();
        let mut rng = self.rng.lock();

        for cmd in commands {
            // Add the original command
            noisy_commands.push(cmd.clone());

            // For each qubit in the command, maybe apply noise
            for &qubit in &cmd.qubits {
                if rng.gen::<f64>() < self.probability {
                    // Randomly choose X, Y, or Z error
                    match rng.gen::<f64>() * 3.0 {
                        x if x < 1.0 => noisy_commands.extend(Self::x_gates(qubit)),
                        x if x < 2.0 => noisy_commands.extend(Self::y_gates(qubit)),
                        _ => noisy_commands.push(Self::z_gate(qubit)),
                    }
                }
            }
        }

        noisy_commands
    }
}
