// PECOS/crates/pecos-engines/gateable_types.rs
use pecos_qsim::{SparseStab, VecSet};

/// Base trait for any quantum simulator
pub trait BaseQuantumSimulator {
    fn new(num_qubits: usize) -> Self;
    fn reset(&mut self);
    fn num_qubits(&self) -> usize;
}

/// Capability to perform all Clifford operations including measurements
pub trait CliffordGateable {
    // Single qubit operations
    fn h(&mut self, qubit: usize);

    // Two qubit operations
    fn cx(&mut self, control: usize, target: usize);

    // Measurements
    fn mz(&mut self, qubit: usize) -> (bool, bool); // (result, was_deterministic)
}

/// Capability to perform arbitrary rotations (non-Clifford)
pub trait ArbitraryRotationGateable {
    fn r1xy(&mut self, theta: f64, phi: f64, qubit: usize);
}

// Implementation for our stabilizer simulator
impl BaseQuantumSimulator for SparseStab<VecSet<u32>, u32> {
    fn new(num_qubits: usize) -> Self {
        SparseStab::new(num_qubits)
    }
    fn reset(&mut self) {
        self.reset();
    }
    fn num_qubits(&self) -> usize {
        self.num_qubits()
    }
}

impl CliffordGateable for SparseStab<VecSet<u32>, u32> {
    fn h(&mut self, qubit: usize) {
        self.h(qubit);
    }
    fn cx(&mut self, control: usize, target: usize) {
        self.cx(control, target);
    }
    fn mz(&mut self, qubit: usize) -> (bool, bool) {
        self.mz(qubit)
    }
}