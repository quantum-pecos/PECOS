// Copyright 2024 The PECOS Developers
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License.You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use super::arbitrary_rotation_gateable::ArbitraryRotationGateable;
use super::clifford_gateable::{CliffordGateable, MeasurementResult};
use super::quantum_simulator::QuantumSimulator;
use pecos_core::SimRng;
use rand_chacha::ChaCha8Rng;

use num_complex::Complex64;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct StateVec<R = ChaCha8Rng>
where
    R: SimRng,
{
    num_qubits: usize,
    state: Vec<Complex64>,
    rng: R,
}

impl StateVec {
    /// Create a new state initialized to |0...0⟩
    #[must_use]
    #[inline]
    pub fn new(num_qubits: usize) -> StateVec<ChaCha8Rng> {
        let rng = ChaCha8Rng::from_entropy();
        StateVec::with_rng(num_qubits, rng)
    }
}

impl<R> StateVec<R>
where
    R: SimRng,
{
    /// Returns the number of qubits in the system
    ///
    /// # Returns
    /// * `usize` - The total number of qubits this simulator is configured to handle
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{QuantumSimulator, StateVec};
    /// let state = StateVec::new(2);
    /// let num = state.num_qubits();
    /// assert_eq!(num, 2);
    /// ```
    #[must_use]
    #[inline]
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    pub fn with_rng(num_qubits: usize, rng: R) -> Self {
        let size = 1 << num_qubits; // 2^n
        let mut state = vec![Complex64::new(0.0, 0.0); size];
        state[0] = Complex64::new(1.0, 0.0); // Prep |0...0>
        StateVec {
            num_qubits,
            state,
            rng,
        }
    }

    /// Initialize from a custom state vector
    ///
    /// # Panics
    ///
    /// Panics if the input state requires more qubits then `StateVec` has.
    #[must_use]
    #[inline]
    pub fn from_state(state: Vec<Complex64>, rng: R) -> Self {
        let num_qubits = state.len().trailing_zeros() as usize;
        assert_eq!(1 << num_qubits, state.len(), "Invalid state vector size");
        StateVec {
            num_qubits,
            state,
            rng,
        }
    }

    /// Prepare a specific computational basis state
    ///
    /// # Panics
    ///
    /// Panics if `basis_state` >= `2^num_qubits` (i.e., if the basis state index is too large for the number of qubits)
    #[inline]
    pub fn prepare_computational_basis(&mut self, basis_state: usize) -> &mut Self {
        assert!(basis_state < 1 << self.num_qubits);
        self.state.fill(Complex64::new(0.0, 0.0));
        self.state[basis_state] = Complex64::new(1.0, 0.0);
        self
    }

    /// Prepare all qubits in |+⟩ state
    #[inline]
    pub fn prepare_plus_state(&mut self) -> &mut Self {
        let factor = Complex64::new(1.0 / f64::from(1 << self.num_qubits), 0.0);
        self.state.fill(factor);
        self
    }

    /// Returns reference to the state vector
    #[must_use]
    #[inline]
    pub fn state(&self) -> &[Complex64] {
        &self.state
    }

    /// Returns the probability of measuring a specific basis state
    ///
    /// # Panics
    ///
    /// Panics if `basis_state` >= `2^num_qubits` (i.e., if the basis state index is too large for the number of qubits)
    #[must_use]
    #[inline]
    pub fn probability(&self, basis_state: usize) -> f64 {
        assert!(basis_state < 1 << self.num_qubits);
        self.state[basis_state].norm_sqr()
    }

    /// Apply a general single-qubit unitary gate
    ///
    /// # Examples
    /// ```
    /// use pecos_qsim::state_vec::StateVec;
    /// use std::f64::consts::FRAC_1_SQRT_2;
    /// use num_complex::Complex64;
    /// let mut q = StateVec::new(1);
    /// // Apply Hadamard gate
    /// q.single_qubit_rotation(0,
    ///     Complex64::new(FRAC_1_SQRT_2, 0.0),  // u00
    ///     Complex64::new(FRAC_1_SQRT_2, 0.0),  // u01
    ///     Complex64::new(FRAC_1_SQRT_2, 0.0),  // u10
    ///     Complex64::new(-FRAC_1_SQRT_2, 0.0), // u11
    /// );
    /// ```
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `qubit` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    pub fn single_qubit_rotation(
        &mut self,
        target: usize,
        u00: Complex64,
        u01: Complex64,
        u10: Complex64,
        u11: Complex64,
    ) -> &mut Self {
        let step = 1 << target;
        for i in (0..self.state.len()).step_by(2 * step) {
            for offset in 0..step {
                let j = i + offset;
                let k = j ^ step;

                let a = self.state[j];
                let b = self.state[k];

                self.state[j] = u00 * a + u01 * b;
                self.state[k] = u10 * a + u11 * b;
            }
        }
        self
    }

    /// Apply a general two-qubit unitary given by a 4x4 complex matrix
    /// U = [[u00, u01, u02, u03],
    ///      [u10, u11, u12, u13],
    ///      [u20, u21, u22, u23],
    ///      [u30, u31, u32, u33]]
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `qubit1` and `qubit2` are valid qubit indices (i.e., `< number of qubits`).
    /// - `qubit1 != qubit2`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    pub fn two_qubit_unitary(
        &mut self,
        qubit1: usize,
        qubit2: usize,
        matrix: [[Complex64; 4]; 4],
    ) -> &mut Self {
        // Make sure qubit1 < qubit2 for consistent ordering
        let (q1, q2) = if qubit1 < qubit2 {
            (qubit1, qubit2)
        } else {
            (qubit2, qubit1)
        };

        // Process state vector in groups of 4 amplitudes
        for i in 0..self.state.len() {
            let bit1 = (i >> q1) & 1;
            let bit2 = (i >> q2) & 1;

            // Only process each set of 4 states once
            if bit1 == 0 && bit2 == 0 {
                // Calculate indices for all four basis states
                let i00 = i;
                let i01 = i ^ (1 << q2);
                let i10 = i ^ (1 << q1);
                let i11 = i ^ (1 << q1) ^ (1 << q2);

                // Store original amplitudes
                let a00 = self.state[i00];
                let a01 = self.state[i01];
                let a10 = self.state[i10];
                let a11 = self.state[i11];

                // Apply the 4x4 unitary transformation
                self.state[i00] = matrix[0][0] * a00
                    + matrix[0][1] * a01
                    + matrix[0][2] * a10
                    + matrix[0][3] * a11;
                self.state[i01] = matrix[1][0] * a00
                    + matrix[1][1] * a01
                    + matrix[1][2] * a10
                    + matrix[1][3] * a11;
                self.state[i10] = matrix[2][0] * a00
                    + matrix[2][1] * a01
                    + matrix[2][2] * a10
                    + matrix[2][3] * a11;
                self.state[i11] = matrix[3][0] * a00
                    + matrix[3][1] * a01
                    + matrix[3][2] * a10
                    + matrix[3][3] * a11;
            }
        }
        self
    }
}

impl QuantumSimulator for StateVec {
    #[inline]
    fn reset(&mut self) -> &mut Self {
        self.prepare_computational_basis(0)
    }
}

impl CliffordGateable<usize> for StateVec {
    /// Apply Pauli-X gate
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn x(&mut self, target: usize) -> &mut Self {
        let step = 1 << target;

        for i in (0..self.state.len()).step_by(2 * step) {
            for offset in 0..step {
                self.state.swap(i + offset, i + offset + step);
            }
        }
        self
    }

    /// Apply Y = [[0, -i], [i, 0]] gate to target qubit
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn y(&mut self, target: usize) -> &mut Self {
        for i in 0..self.state.len() {
            if (i >> target) & 1 == 0 {
                let flipped_i = i ^ (1 << target);
                let temp = self.state[i];
                self.state[i] = -Complex64::i() * self.state[flipped_i];
                self.state[flipped_i] = Complex64::i() * temp;
            }
        }
        self
    }

    /// Apply Z = [[1, 0], [0, -1]] gate to target qubit
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn z(&mut self, target: usize) -> &mut Self {
        for i in 0..self.state.len() {
            if (i >> target) & 1 == 1 {
                self.state[i] = -self.state[i];
            }
        }
        self
    }

    #[inline]
    fn sz(&mut self, q: usize) -> &mut Self {
        self.single_qubit_rotation(
            q,
            Complex64::new(1.0, 0.0), // u00
            Complex64::new(0.0, 0.0), // u01
            Complex64::new(0.0, 0.0), // u10
            Complex64::new(0.0, 1.0), // u11
        )
    }

    /// Apply Hadamard gate to the target qubit
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn h(&mut self, target: usize) -> &mut Self {
        let factor = Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0);
        let step = 1 << target;

        for i in (0..self.state.len()).step_by(2 * step) {
            for offset in 0..step {
                let j = i + offset;
                let paired_j = j ^ step;

                let a = self.state[j];
                let b = self.state[paired_j];

                self.state[j] = factor * (a + b);
                self.state[paired_j] = factor * (a - b);
            }
        }
        self
    }

    /// Apply controlled-X gate
    /// CX = |0⟩⟨0| ⊗ I + |1⟩⟨1| ⊗ X
    ///
    /// See [`CliffordGateable::cx`] for detailed documentation.
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `control` and `target` are valid qubit indices (i.e., `< number of qubits`).
    /// - `control != target`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn cx(&mut self, control: usize, target: usize) -> &mut Self {
        for i in 0..self.state.len() {
            let control_val = (i >> control) & 1;
            let target_val = (i >> target) & 1;
            if control_val == 1 && target_val == 0 {
                let flipped_i = i ^ (1 << target);
                self.state.swap(i, flipped_i);
            }
        }
        self
    }

    /// Apply controlled-Y gate
    /// CY = |0⟩⟨0| ⊗ I + |1⟩⟨1| ⊗ Y
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `control` and `target` are valid qubit indices (i.e., `< number of qubits`).
    /// - `control != target`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn cy(&mut self, control: usize, target: usize) -> &mut Self {
        // Only process when control bit is 1 and target bit is 0
        for i in 0..self.state.len() {
            let control_val = (i >> control) & 1;
            let target_val = (i >> target) & 1;

            if control_val == 1 && target_val == 0 {
                let flipped_i = i ^ (1 << target);

                // Y gate has different phases than X
                // Y = [[0, -i], [i, 0]]
                let temp = self.state[i];
                self.state[i] = -Complex64::i() * self.state[flipped_i];
                self.state[flipped_i] = Complex64::i() * temp;
            }
        }
        self
    }

    /// Apply controlled-Z gate
    /// CZ = |0⟩⟨0| ⊗ I + |1⟩⟨1| ⊗ Z
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `control` and `target` are valid qubit indices (i.e., `< number of qubits`).
    /// - `control != target`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn cz(&mut self, control: usize, target: usize) -> &mut Self {
        // CZ is simpler - just add phase when both control and target are 1
        for i in 0..self.state.len() {
            let control_val = (i >> control) & 1;
            let target_val = (i >> target) & 1;

            if control_val == 1 && target_val == 1 {
                self.state[i] = -self.state[i];
            }
        }
        self
    }

    /// Apply a SWAP gate between two qubits
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `qubit1` and `qubit2` are valid qubit indices (i.e., `< number of qubits`).
    /// - `qubit1 != qubit2`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn swap(&mut self, qubit1: usize, qubit2: usize) -> &mut Self {
        let step1 = 1 << qubit1;
        let step2 = 1 << qubit2;

        for i in 0..self.state.len() {
            let bit1 = (i >> qubit1) & 1;
            let bit2 = (i >> qubit2) & 1;

            if bit1 != bit2 {
                let swapped_index = i ^ step1 ^ step2;
                if i < swapped_index {
                    self.state.swap(i, swapped_index);
                }
            }
        }
        self
    }

    /// Measure a single qubit in the Z basis and collapse the state
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn mz(&mut self, target: usize) -> MeasurementResult {
        let step = 1 << target;
        let mut prob_one = 0.0;

        // Calculate probability of measuring |1⟩
        for i in (0..self.state.len()).step_by(2 * step) {
            for offset in 0..step {
                let idx = i + offset + step; // Target bit = 1 positions
                prob_one += self.state[idx].norm_sqr();
            }
        }

        // Decide measurement outcome
        let result = usize::from(self.rng.gen::<f64>() < prob_one);

        // Collapse and normalize state
        let mut norm = 0.0;
        for i in 0..self.state.len() {
            let bit = (i >> target) & 1;
            if bit == result {
                norm += self.state[i].norm_sqr();
            } else {
                self.state[i] = Complex64::new(0.0, 0.0);
            }
        }

        let norm_inv = 1.0 / norm.sqrt();
        for amp in &mut self.state {
            *amp *= norm_inv;
        }

        MeasurementResult {
            outcome: result != 0,
            is_deterministic: false,
        }
    }
}

impl ArbitraryRotationGateable<usize> for StateVec {
    /// Gate RX(θ) = exp(-i θ X/2) = cos(θ/2) I - i*sin(θ/2) X
    /// RX(θ) = [[cos(θ/2), -i*sin(θ/2)],
    ///          [-i*sin(θ/2), cos(θ/2)]]
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn rx(&mut self, theta: f64, target: usize) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        self.single_qubit_rotation(
            target,
            Complex64::new(cos, 0.0),
            Complex64::new(0.0, -sin),
            Complex64::new(0.0, -sin),
            Complex64::new(cos, 0.0),
        )
    }

    /// Gate RY(θ) = exp(-i θ Y/2) = cos(θ/2) I - i*sin(θ/2) Y
    /// RY(θ) = [[cos(θ/2), -sin(θ/2)],
    ///          [-sin(θ/2), cos(θ/2)]]
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn ry(&mut self, theta: f64, target: usize) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        self.single_qubit_rotation(
            target,
            Complex64::new(cos, 0.0),
            Complex64::new(-sin, 0.0),
            Complex64::new(sin, 0.0),
            Complex64::new(cos, 0.0),
        )
    }

    /// Gate RZ(θ) = exp(-i θ Z/2) = cos(θ/2) I - i*sin(θ/2) Z
    /// RZ(θ) = [[cos(θ/2)-i*sin(θ/2), 0],
    ///          [0, cos(θ/2)+i*sin(θ/2)]]
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn rz(&mut self, theta: f64, target: usize) -> &mut Self {
        let e_pos = Complex64::from_polar(1.0, -theta / 2.0);
        let e_neg = Complex64::from_polar(1.0, theta / 2.0);

        self.single_qubit_rotation(
            target,
            e_pos,
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            e_neg,
        )
    }

    /// Apply U(theta, phi, lambda) gate
    /// `U1_3` = [[cos(θ/2), -e^(iλ)sin(θ/2)],
    ///         [e^(iφ)sin(θ/2), e^(i(λ+φ))cos(θ/2)]]
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn u(&mut self, theta: f64, phi: f64, lambda: f64, target: usize) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();

        // Calculate matrix elements
        let u00 = Complex64::new(cos, 0.0);
        let u01 = -Complex64::from_polar(sin, lambda);
        let u10 = Complex64::from_polar(sin, phi);
        let u11 = Complex64::from_polar(cos, phi + lambda);

        self.single_qubit_rotation(target, u00, u01, u10, u11)
    }

    /// Single qubit rotation in the X-Y plane.
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `target` is a valid qubit indices (i.e., `< number of qubits`).
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn r1xy(&mut self, theta: f64, phi: f64, target: usize) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();

        // Calculate the matrix elements
        let r00 = Complex64::new(cos, 0.0); // cos(θ/2)
        let r01 = -Complex64::new(0.0, sin) * Complex64::from_polar(1.0, -phi); // -i sin(θ/2) e^(-iφ)
        let r10 = -Complex64::new(0.0, sin) * Complex64::from_polar(1.0, phi); // -i sin(θ/2) e^(iφ)
        let r11 = Complex64::new(cos, 0.0); // cos(θ/2)

        // Apply the single-qubit rotation using the matrix elements
        self.single_qubit_rotation(target, r00, r01, r10, r11)
    }

    /// Apply RXX(θ) = exp(-i θ XX/2) gate
    /// This implements evolution under the XX coupling between two qubits
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `qubit1` and `qubit2` are valid qubit indices (i.e., `< number of qubits`).
    /// - `qubit1 != qubit2`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn rxx(&mut self, theta: f64, qubit1: usize, qubit2: usize) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let neg_i_sin = Complex64::new(0.0, -sin); // -i*sin

        // Make sure qubit1 < qubit2 for consistent ordering
        let (q1, q2) = if qubit1 < qubit2 {
            (qubit1, qubit2)
        } else {
            (qubit2, qubit1)
        };

        for i in 0..self.state.len() {
            let bit1 = (i >> q1) & 1;
            let bit2 = (i >> q2) & 1;

            if bit1 == 0 && bit2 == 0 {
                let i01 = i ^ (1 << q2);
                let i10 = i ^ (1 << q1);
                let i11 = i ^ (1 << q1) ^ (1 << q2);

                let a00 = self.state[i];
                let a01 = self.state[i01];
                let a10 = self.state[i10];
                let a11 = self.state[i11];

                // Apply the correct RXX matrix
                self.state[i] = cos * a00 + neg_i_sin * a11;
                self.state[i01] = cos * a01 + neg_i_sin * a10;
                self.state[i10] = cos * a10 + neg_i_sin * a01;
                self.state[i11] = cos * a11 + neg_i_sin * a00;
            }
        }
        self
    }

    /// Apply RYY(θ) = exp(-i θ YY/2) gate
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `qubit1` and `qubit2` are valid qubit indices (i.e., `< number of qubits`).
    /// - `qubit1 != qubit2`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    #[inline]
    fn ryy(&mut self, theta: f64, q1: usize, q2: usize) -> &mut Self {
        let cos = (theta / 2.0).cos();
        let i_sin = Complex64::new(0.0, 1.0) * (theta / 2.0).sin();

        // No need to reorder q1 and q2 since we're using explicit masks
        let mask1 = 1 << q1;
        let mask2 = 1 << q2;

        for i in 0..self.state.len() {
            // Only process each set of 4 states once
            if (i & (mask1 | mask2)) == 0 {
                let i00 = i;
                let i01 = i | mask2;
                let i10 = i | mask1;
                let i11 = i | mask1 | mask2;

                let a00 = self.state[i00];
                let a01 = self.state[i01];
                let a10 = self.state[i10];
                let a11 = self.state[i11];

                self.state[i00] = cos * a00 + i_sin * a11;
                self.state[i01] = cos * a01 - i_sin * a10;
                self.state[i10] = cos * a10 - i_sin * a01;
                self.state[i11] = cos * a11 + i_sin * a00;
            }
        }
        self
    }

    /// Apply RZZ(θ) = exp(-i θ ZZ/2) gate
    ///
    /// This is an optimized implementation of the general two-qubit unitary:
    /// ```text
    /// RZZ(θ) = [[e^(-iθ/2),     0,          0,          0        ],
    ///           [0,          e^(iθ/2),      0,          0        ],
    ///           [0,             0,       e^(iθ/2),      0        ],
    ///           [0,             0,          0,       e^(-iθ/2)   ]]
    /// ```
    /// Optimized by taking advantage of the diagonal structure.
    ///
    /// # Safety
    ///
    /// This function assumes that:
    /// - `qubit1` and `qubit2` are valid qubit indices (i.e., `< number of qubits`).
    /// - `qubit1 != qubit2`.
    /// - These conditions must be ensured by the caller or a higher-level component.
    fn rzz(&mut self, theta: f64, qubit1: usize, qubit2: usize) -> &mut Self {
        // RZZ is diagonal in computational basis - just add phases
        for i in 0..self.state.len() {
            let bit1 = (i >> qubit1) & 1;
            let bit2 = (i >> qubit2) & 1;

            // Phase depends on parity of bits
            let phase = if bit1 ^ bit2 == 0 {
                // Same bits (00 or 11) -> e^(-iθ/2)
                Complex64::from_polar(1.0, -theta / 2.0)
            } else {
                // Different bits (01 or 10) -> e^(iθ/2)
                Complex64::from_polar(1.0, theta / 2.0)
            };

            self.state[i] *= phase;
        }
        self
    }
}

/// ## Key Invariants
/// - **Normalization**: The total probability (norm squared) of the state vector
///   must always be 1.
/// - **Unitarity**: All gate operations must preserve the norm of the state vector.
/// - **Phase Consistency**: States are compared up to a global phase.
///
/// ## Testing Strategy
/// - **Unit Tests**: Validate individual operations (e.g., `RX`, `RY`, `RZ`, `CX`, `CY`, etc.).
/// - **Compositional Tests**: Verify decompositions, commutation relations, and symmetry properties.
/// - **Edge Cases**: Test with boundary values (e.g., `θ = 0`, `θ = 2π`) and systems near memory limits.
/// - **Randomized Tests**: Evaluate probabilistic operations like measurement and ensure statistical validity.
/// - **Integration Tests**: Combine operations to ensure the overall system behaves as expected.
///
/// ## Test Organization
/// - Each gate or operation is tested independently for correctness.
/// - Helper functions like `assert_states_equal` are used to compare quantum states up to global phase.
/// - Failures provide clear diagnostic outputs for debugging, including mismatches and intermediate states.
#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, PI, TAU};

    use num_complex::Complex64;

    /// Compare two quantum states up to global phase.
    ///
    /// This function ensures that two state vectors represent the same quantum state,
    /// accounting for potential differences in global phase.
    ///
    /// # Arguments
    /// - `state1`: A reference to the first state vector.
    /// - `state2`: A reference to the second state vector.
    ///
    /// # Panics
    /// Panics if the states differ in norm or relative phase beyond a small numerical tolerance.
    fn assert_states_equal(state1: &[Complex64], state2: &[Complex64]) {
        const TOLERANCE: f64 = 1e-10;

        if state1[0].norm() < TOLERANCE && state2[0].norm() < TOLERANCE {
            // Both first components near zero, compare other components directly
            for (index, (a, b)) in state1.iter().zip(state2.iter()).enumerate() {
                assert!(
                    (a.norm() - b.norm()).abs() < TOLERANCE,
                    "States differ in magnitude at index {index}: {a} vs {b}"
                );
            }
        } else {
            // Get phase from the first pair of non-zero components
            let ratio = match state1
                .iter()
                .zip(state2.iter())
                .find(|(a, b)| a.norm() > TOLERANCE && b.norm() > TOLERANCE)
            {
                Some((a, b)) => b / a,
                None => panic!("States have no corresponding non-zero components"),
            };
            println!("Phase ratio between states: {ratio:?}");

            for (index, (a, b)) in state1.iter().zip(state2.iter()).enumerate() {
                assert!(
                    (a * ratio - b).norm() < TOLERANCE,
                    "States differ at index {index}: {a} vs {b} (adjusted with ratio {ratio:?}), diff = {}",
                    (a * ratio - b).norm()
                );
            }
        }
    }

    #[test]
    fn test_new_state() {
        // Verify the initial state is correctly set to |0>
        let q = StateVec::new(2);
        assert_eq!(q.state[0], Complex64::new(1.0, 0.0));
        for i in 1..4 {
            assert_eq!(q.state[i], Complex64::new(0.0, 0.0));
        }
    }

    #[test]
    fn test_rx() {
        // Test RX gate functionality
        let mut q = StateVec::new(1);

        // RX(π) should flip |0⟩ to -i|1⟩
        q.rx(PI, 0);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].norm() - 1.0).abs() < 1e-10);

        // RX(2π) should return to the initial state up to global phase
        let mut q = StateVec::new(1);
        q.rx(2.0 * PI, 0);
        assert!((q.state[0].norm() - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);
    }

    #[test]
    fn test_ry() {
        let mut q = StateVec::new(1);

        // RY(π) should flip |0⟩ to |1⟩
        q.ry(PI, 0);
        assert!(q.state[0].norm() < 1e-10); // Close to zero
        assert!((q.state[1].norm() - 1.0).abs() < 1e-10); // Magnitude 1 for |1⟩

        // Two RY(π) rotations should return to the initial state
        q.ry(PI, 0);
        assert!((q.state[0].norm() - 1.0).abs() < 1e-10); // Magnitude 1 for |0⟩
        assert!(q.state[1].norm() < 1e-10); // Close to zero
    }

    #[test]
    fn test_rz() {
        let mut q = StateVec::new(1);

        // RZ should only add phases, not change probabilities
        q.h(0); // Put qubit in superposition
        let probs_before: Vec<f64> = q.state.iter().map(num_complex::Complex::norm_sqr).collect();

        q.rz(FRAC_PI_2, 0); // Rotate Z by π/2
        let probs_after: Vec<f64> = q.state.iter().map(num_complex::Complex::norm_sqr).collect();

        for (p1, p2) in probs_before.iter().zip(probs_after.iter()) {
            assert!((p1 - p2).abs() < 1e-10); // Probabilities unchanged
        }
    }

    #[test]
    fn test_rx_step_by_step() {
        let mut q = StateVec::new(1);

        // Step 1: RX(0) should be identity
        q.rx(0.0, 0);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Step 2: RX(π) on |0⟩ should give -i|1⟩
        let mut q = StateVec::new(1);
        q.rx(PI, 0);
        println!("RX(π)|0⟩ = {:?}", q.state); // Debug output
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].im + 1.0).abs() < 1e-10);

        // Step 3: RX(π/2) on |0⟩ should give (|0⟩ - i|1⟩)/√2
        let mut q = StateVec::new(1);
        q.rx(FRAC_PI_2, 0);
        println!("RX(π/2)|0⟩ = {:?}", q.state); // Debug output
        let expected_amp = 1.0 / 2.0_f64.sqrt();
        assert!((q.state[0].re - expected_amp).abs() < 1e-10);
        assert!((q.state[1].im + expected_amp).abs() < 1e-10);
    }

    #[test]
    fn test_ry_step_by_step() {
        // Step 1: RY(0) should be identity
        let mut q = StateVec::new(1);
        q.ry(0.0, 0);
        println!("RY(0)|0⟩ = {:?}", q.state);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Step 2: RY(π) on |0⟩ should give |1⟩
        let mut q = StateVec::new(1);
        q.ry(PI, 0);
        println!("RY(π)|0⟩ = {:?}", q.state);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].re - 1.0).abs() < 1e-10);

        // Step 3: RY(π/2) on |0⟩ should give (|0⟩ + |1⟩)/√2
        let mut q = StateVec::new(1);
        q.ry(FRAC_PI_2, 0);
        println!("RY(π/2)|0⟩ = {:?}", q.state);
        let expected_amp = 1.0 / 2.0_f64.sqrt();
        assert!((q.state[0].re - expected_amp).abs() < 1e-10);
        assert!((q.state[1].re - expected_amp).abs() < 1e-10);

        // Step 4: RY(-π/2) on |0⟩ should give (|0⟩ - |1⟩)/√2
        let mut q = StateVec::new(1);
        q.ry(-FRAC_PI_2, 0);
        println!("RY(-π/2)|0⟩ = {:?}", q.state);
        assert!((q.state[0].re - expected_amp).abs() < 1e-10);
        assert!((q.state[1].re + expected_amp).abs() < 1e-10);
    }

    #[test]
    fn test_rz_step_by_step() {
        // Step 1: RZ(0) should be identity
        let mut q = StateVec::new(1);
        q.rz(0.0, 0);
        println!("RZ(0)|0⟩ = {:?}", q.state);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Step 2: RZ(π/2) on |+⟩ should give |+i⟩ = (|0⟩ + i|1⟩)/√2
        let mut q = StateVec::new(1);
        q.h(0); // Create |+⟩
        q.rz(FRAC_PI_2, 0);
        println!("RZ(π/2)|+⟩ = {:?}", q.state);
        let expected_amp = 1.0 / 2.0_f64.sqrt();
        assert!((q.state[0].norm() - expected_amp).abs() < 1e-10);
        assert!((q.state[1].norm() - expected_amp).abs() < 1e-10);
        // Check relative phase
        let ratio = q.state[1] / q.state[0];
        println!("Relative phase ratio = {ratio:?}");
        assert!(
            (ratio.im - 1.0).abs() < 1e-10,
            "Relative phase incorrect: ratio = {ratio}"
        );
        assert!(
            ratio.re.abs() < 1e-10,
            "Relative phase has unexpected real component: {}",
            ratio.re
        );

        // Step 3: Two RZ(π/2) operations should equal one RZ(π)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.rz(PI, 0);
        q2.rz(FRAC_PI_2, 0);
        q2.rz(FRAC_PI_2, 0);
        println!("RZ(π)|0⟩ vs RZ(π/2)RZ(π/2)|0⟩:");
        println!("q1 = {:?}", q1.state);
        println!("q2 = {:?}", q2.state);
        let ratio = q2.state[0] / q1.state[0];
        let phase = ratio.arg();
        println!("Phase difference between q2 and q1: {phase}");
        assert!(
            (ratio.norm() - 1.0).abs() < 1e-10,
            "Magnitudes differ: ratio = {ratio}"
        );
        // Don't check exact phase, just verify states are equal up to global phase
        assert!((q2.state[1] * q1.state[0] - q2.state[0] * q1.state[1]).norm() < 1e-10);
    }

    #[test]
    fn test_sq_rotation_commutation() {
        // RX and RY don't commute - verify RX(θ)RY(φ) ≠ RY(φ)RX(θ)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);

        let theta = FRAC_PI_3; // π/3
        let phi = FRAC_PI_4; // π/4

        // Apply in different orders
        q1.rx(theta, 0).ry(phi, 0);
        q2.ry(phi, 0).rx(theta, 0);

        println!("RY(π/4)RX(π/3)|0⟩ = {:?}", q1.state);
        println!("RX(π/3)RY(π/4)|0⟩ = {:?}", q2.state);

        // States should be different - check they're not equal up to global phase
        let ratio = q2.state[0] / q1.state[0];
        assert!((q2.state[1] / q1.state[1] - ratio).norm() > 1e-10);
    }

    #[test]
    fn test_sq_rotation_decompositions() {
        // H = RZ(-π)RY(-π/2)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);

        println!("Initial states:");
        println!("q1 = {:?}", q1.state);
        println!("q2 = {:?}", q2.state);

        q1.h(0); // Direct H
        println!("After H: q1 = {:?}", q1.state);

        // H via rotations - changed order and added negative sign to RZ angle
        q2.ry(-FRAC_PI_2, 0).rz(-PI, 0);
        println!("After RZ(-π)RY(-π/2): q2 = {:?}", q2.state);

        // Compare up to global phase by looking at ratios between components
        let ratio = q2.state[0] / q1.state[0];
        println!("Ratio = {ratio:?}");
        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            println!("Comparing {a} and {b}");
            assert!(
                (a * ratio - b).norm() < 1e-10,
                "States differ: {a} vs {b} (ratio: {ratio})"
            );
        }
    }

    #[test]
    fn test_sq_standard_gate_decompositions() {
        // Test S = RZ(π/2)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.sz(0);
        q2.rz(FRAC_PI_2, 0);
        println!("S|0⟩ = {:?}", q1.state);
        println!("RZ(π/2)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);

        // Test X = RX(π)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.x(0);
        q2.rx(PI, 0);
        println!("X|0⟩ = {:?}", q1.state);
        println!("RX(π)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);

        // Test Y = RY(π)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.y(0);
        q2.ry(PI, 0);
        println!("Y|0⟩ = {:?}", q1.state);
        println!("RY(π)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);

        // Test Z = RZ(π)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.z(0);
        q2.rz(PI, 0);
        println!("Z|0⟩ = {:?}", q1.state);
        println!("RZ(π)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);

        // Test √X = RX(π/2)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.sx(0);
        q2.rx(FRAC_PI_2, 0);
        println!("√X|0⟩ = {:?}", q1.state);
        println!("RX(π/2)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);

        // Test √Y = RY(π/2)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.sy(0);
        q2.ry(FRAC_PI_2, 0);
        println!("√Y|0⟩ = {:?}", q1.state);
        println!("RY(π/2)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);

        // Test S = TT as RZ(π/4)RZ(π/4)
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q2.rz(FRAC_PI_4, 0).rz(FRAC_PI_4, 0);
        q1.sz(0);
        println!("S|0⟩ = {:?}", q1.state);
        println!("T²|0⟩ = RZ(π/4)RZ(π/4)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);

        // Test H = RX(π)RY(π/2) decomposition
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);
        q1.h(0);
        q2.ry(FRAC_PI_2, 0).rx(PI, 0);
        println!("H|0⟩ = {:?}", q1.state);
        println!("RX(π)RY(π/2)|0⟩ = {:?}", q2.state);
        assert_states_equal(&q1.state, &q2.state);
    }

    #[test]
    fn test_rx_rotation_angle_relations() {
        // Test that RX(θ)RX(-θ) = I
        let mut q = StateVec::new(1);
        let theta = FRAC_PI_3;

        // Apply forward then reverse rotations
        q.rx(theta, 0).rx(-theta, 0);

        // Should get back to |0⟩ up to global phase
        assert!(q.state[1].norm() < 1e-10);
        assert!((q.state[0].norm() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ry_rotation_angle_relations() {
        // Test that RY(θ)RY(-θ) = I
        let mut q = StateVec::new(1);
        let theta = FRAC_PI_3;

        // Apply forward then reverse rotations
        q.ry(theta, 0).ry(-theta, 0);

        // Should get back to |0⟩ up to global phase
        assert!(q.state[1].norm() < 1e-10);
        assert!((q.state[0].norm() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rz_rotation_angle_relations() {
        // Test that RZ(θ)RZ(-θ) = I
        let mut q = StateVec::new(1);
        let theta = FRAC_PI_3;

        // Apply forward then reverse rotations
        q.rz(theta, 0).rz(-theta, 0);

        // Should get back to |0⟩ up to global phase
        assert!(q.state[1].norm() < 1e-10);
        assert!((q.state[0].norm() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_state_vec_u_vs_trait_u() {
        // Initialize state vectors with one qubit in the |0⟩ state.
        let mut state_vec_u = StateVec::new(1);
        let mut trait_u = StateVec::new(1);

        let theta = FRAC_PI_3;
        let phi = FRAC_PI_4;
        let lambda = FRAC_PI_6;

        // Apply `u` from the StateVec implementation.
        state_vec_u.u(theta, phi, lambda, 0);

        // Apply `u` from the ArbitraryRotationGateable trait.
        ArbitraryRotationGateable::u(&mut trait_u, theta, phi, lambda, 0);

        assert_states_equal(&state_vec_u.state, &trait_u.state);
    }

    #[test]
    fn test_r1xy_vs_trait_r1xy() {
        // Initialize state vectors with one qubit in the |0⟩ state.
        let mut state_vec_r1xy = StateVec::new(1);
        let mut trait_r1xy = StateVec::new(1);

        // Define angles for the test.
        let theta = FRAC_PI_3;
        let phi = FRAC_PI_4;

        // Apply the manual `r1xy` implementation.
        state_vec_r1xy.r1xy(theta, phi, 0);

        // Apply the `r1xy` implementation from the `ArbitraryRotationGateable` trait.
        ArbitraryRotationGateable::r1xy(&mut trait_r1xy, theta, phi, 0);

        // Use the `assert_states_equal` function to compare the states up to a global phase.
        assert_states_equal(&state_vec_r1xy.state, &trait_r1xy.state);
    }

    #[test]
    fn test_r1xy_vs_u() {
        let mut state_r1xy = StateVec::new(1);
        let mut state_u = StateVec::new(1);

        let theta = FRAC_PI_3;
        let phi = FRAC_PI_4;

        // Apply r1xy and equivalent u gates
        state_r1xy.r1xy(theta, phi, 0);
        state_u.u(theta, phi - FRAC_PI_2, FRAC_PI_2 - phi, 0);

        assert_states_equal(&state_r1xy.state, &state_u.state);
    }

    #[test]
    fn test_rz_vs_u() {
        let mut state_rz = StateVec::new(1);
        let mut state_u = StateVec::new(1);

        let theta = FRAC_PI_3;

        // Apply rz and u gates
        state_rz.rz(theta, 0);
        state_u.u(0.0, 0.0, theta, 0);

        assert_states_equal(&state_rz.state, &state_u.state);
    }

    #[test]
    fn test_u_decomposition() {
        let mut state_u = StateVec::new(1);
        let mut state_decomposed = StateVec::new(1);

        let theta = FRAC_PI_3;
        let phi = FRAC_PI_4;
        let lambda = FRAC_PI_6;

        // Apply U gate
        state_u.u(theta, phi, lambda, 0);

        // Apply the decomposed gates
        state_decomposed.rz(lambda, 0);
        state_decomposed.r1xy(theta, FRAC_PI_2, 0);
        state_decomposed.rz(phi, 0);

        // Assert that the states are equal
        assert_states_equal(&state_u.state, &state_decomposed.state);
    }

    #[test]
    fn test_x_vs_r1xy() {
        let mut state = StateVec::new(1);
        state.x(0);
        let state_after_x = state.clone();

        state.reset();
        state.r1xy(PI, 0.0, 0);
        let state_after_r1xy = state.clone();

        assert_states_equal(&state_after_x.state, &state_after_r1xy.state);
    }

    #[test]
    fn test_y_vs_r1xy() {
        let mut state = StateVec::new(1);
        state.y(0);
        let state_after_y = state.clone();

        state.reset();
        state.r1xy(PI, FRAC_PI_2, 0);
        let state_after_r1xy = state.clone();

        assert_states_equal(&state_after_y.state, &state_after_r1xy.state);
    }

    #[test]
    fn test_h_vs_r1xy_rz() {
        let mut state = StateVec::new(1);
        state.h(0); // Apply the H gate
        let state_after_h = state.clone();

        state.reset(); // Reset state to |0⟩
        state.r1xy(FRAC_PI_2, -FRAC_PI_2, 0).rz(PI, 0);
        let state_after_r1xy_rz = state.clone();

        assert_states_equal(&state_after_h.state, &state_after_r1xy_rz.state);
    }

    #[test]
    fn test_cx_decomposition() {
        let mut state_cx = StateVec::new(2);
        let mut state_decomposed = StateVec::new(2);

        let control = 0;
        let target = 1;

        // Apply CX gate
        state_cx.cx(control, target);

        // Apply the decomposed gates
        state_decomposed.r1xy(-FRAC_PI_2, FRAC_PI_2, target);
        state_decomposed.rzz(FRAC_PI_2, control, target);
        state_decomposed.rz(-FRAC_PI_2, control);
        state_decomposed.r1xy(FRAC_PI_2, PI, target);
        state_decomposed.rz(-FRAC_PI_2, target);

        // Assert that the states are equal
        assert_states_equal(&state_cx.state, &state_decomposed.state);
    }

    #[test]
    fn test_rxx_decomposition() {
        let mut state_rxx = StateVec::new(2);
        let mut state_decomposed = StateVec::new(2);

        let control = 0;
        let target = 1;

        // Apply RXX gate
        state_rxx.rxx(FRAC_PI_4, control, target);

        // Apply the decomposed gates
        state_decomposed.r1xy(FRAC_PI_2, FRAC_PI_2, control);
        state_decomposed.r1xy(FRAC_PI_2, FRAC_PI_2, target);
        state_decomposed.rzz(FRAC_PI_4, control, target);
        state_decomposed.r1xy(FRAC_PI_2, -FRAC_PI_2, control);
        state_decomposed.r1xy(FRAC_PI_2, -FRAC_PI_2, target);

        // Assert that the states are equal
        assert_states_equal(&state_rxx.state, &state_decomposed.state);
    }

    #[test]
    fn test_hadamard() {
        let mut q = StateVec::new(1);
        q.h(0);

        assert!((q.state[0].re - FRAC_1_SQRT_2).abs() < 1e-10);
        assert!((q.state[1].re - FRAC_1_SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn test_swap() {
        let mut q = StateVec::new(2);
        q.x(0);
        q.swap(0, 1);

        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[2].re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cx() {
        let mut q = StateVec::new(2);
        // Prep |+>
        q.h(0);
        q.cx(0, 1);

        // Should be in Bell state (|00> + |11>)/sqrt(2)
        let expected = 1.0 / 2.0_f64.sqrt();
        assert!((q.state[0].re - expected).abs() < 1e-10);
        assert!((q.state[3].re - expected).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);
        assert!(q.state[2].norm() < 1e-10);
    }

    #[test]
    fn test_cy() {
        let mut q = StateVec::new(2);

        // Create |+0⟩ state
        q.h(0);

        // Apply CY to get entangled state
        q.cy(0, 1);

        // Should be (|00⟩ + i|11⟩)/√2
        let expected = FRAC_1_SQRT_2;
        assert!((q.state[0].re - expected).abs() < 1e-10); // |00⟩ amplitude
        assert!(q.state[1].norm() < 1e-10); // |01⟩ amplitude
        assert!(q.state[2].norm() < 1e-10); // |10⟩ amplitude
        assert!((q.state[3].im - expected).abs() < 1e-10); // |11⟩ amplitude
    }

    #[test]
    fn test_cz() {
        let mut q = StateVec::new(2);

        // Create |++⟩ state
        q.h(0);
        q.h(1);

        // Apply CZ
        q.cz(0, 1);

        // Should be (|00⟩ + |01⟩ + |10⟩ - |11⟩)/2
        let expected = 0.5;
        assert!((q.state[0].re - expected).abs() < 1e-10); // |00⟩ amplitude
        assert!((q.state[1].re - expected).abs() < 1e-10); // |01⟩ amplitude
        assert!((q.state[2].re - expected).abs() < 1e-10); // |10⟩ amplitude
        assert!((q.state[3].re + expected).abs() < 1e-10); // |11⟩ amplitude
    }

    #[test]
    fn test_control_target_independence() {
        // Test that CY and CZ work regardless of which qubit is control/target
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);

        // Prepare same initial state
        q1.h(0);
        q1.h(1);
        q2.h(0);
        q2.h(1);

        // Apply gates with different control/target
        q1.cz(0, 1);
        q2.cz(1, 0);

        assert_states_equal(&q1.state, &q2.state);
    }

    #[test]
    fn test_x() {
        let mut q = StateVec::new(1);

        // Check initial state is |0>
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Test X on |0> -> |1>
        q.x(0);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].re - 1.0).abs() < 1e-10);

        // Test X on |1> -> |0>
        q.x(0);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Test X on superposition
        q.h(0);
        let initial_state = q.state.clone();
        q.x(0); // X|+> = |+>
        for (state, initial) in q.state.iter().zip(initial_state.iter()) {
            assert!((state - initial).norm() < 1e-10);
        }

        // Test X on second qubit of two-qubit system
        let mut q = StateVec::new(2);
        q.x(1);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[2].re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_y() {
        let mut q = StateVec::new(1);

        // Test Y on |0⟩ -> i|1⟩
        q.y(0);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1] - Complex64::i()).norm() < 1e-10);

        // Test Y on i|1⟩ -> |0⟩
        q.y(0);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Test Y on |+⟩
        let mut q = StateVec::new(1);
        q.h(0); // Create |+⟩
        q.y(0); // Should give i|-⟩
        let expected = FRAC_1_SQRT_2;
        assert!((q.state[0].im + expected).abs() < 1e-10);
        assert!((q.state[1].im - expected).abs() < 1e-10);
    }

    #[test]
    fn test_z() {
        let mut q = StateVec::new(1);

        // Test Z on |0⟩ -> |0⟩
        q.z(0);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Test Z on |1⟩ -> -|1⟩
        q.x(0); // Prepare |1⟩
        q.z(0);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].re + 1.0).abs() < 1e-10);

        // Test Z on |+⟩ -> |-⟩
        let mut q = StateVec::new(1);
        q.h(0); // Create |+⟩
        q.z(0); // Should give |-⟩
        let expected = FRAC_1_SQRT_2;
        assert!((q.state[0].re - expected).abs() < 1e-10);
        assert!((q.state[1].re + expected).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_relations() {
        let mut q1 = StateVec::new(1);
        let mut q2 = StateVec::new(1);

        // Store initial state
        let initial_state = q1.state.clone();

        // Test XYZ sequence
        q1.x(0);
        q1.y(0);
        q1.z(0);

        // XYZ = -iI, so state should be -i times initial state
        if initial_state[0].norm() > 1e-10 {
            let phase = q1.state[0] / initial_state[0];
            assert!((phase + Complex64::i()).norm() < 1e-10); // Changed to +Complex64::i()
        }

        // Test YZX sequence - should give same result
        q2.y(0);
        q2.z(0);
        q2.x(0);

        // Compare q1 and q2 up to global phase
        if q1.state[0].norm() > 1e-10 {
            let phase = q2.state[0] / q1.state[0];
            let phase_norm = phase.norm();
            assert!((phase_norm - 1.0).abs() < 1e-10);

            for (a, b) in q1.state.iter().zip(q2.state.iter()) {
                assert!((a * phase - b).norm() < 1e-10);
            }
        }
    }

    #[test]
    fn test_rxx() {
        // Test 1: RXX(π/2) on |00⟩ should give (|00⟩ - i|11⟩)/√2
        let mut q = StateVec::new(2);
        q.rxx(FRAC_PI_2, 0, 1);

        let expected = FRAC_1_SQRT_2;
        assert!((q.state[0].re - expected).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);
        assert!(q.state[2].norm() < 1e-10);
        assert!((q.state[3].im + expected).abs() < 1e-10);

        // Test 2: RXX(2π) should return to original state up to global phase
        let mut q = StateVec::new(2);
        q.h(0); // Create some initial state
        let initial = q.state.clone();
        q.rxx(TAU, 0, 1);

        // Compare up to global phase
        if q.state[0].norm() > 1e-10 {
            let phase = q.state[0] / initial[0];
            for (a, b) in q.state.iter().zip(initial.iter()) {
                assert!((a - b * phase).norm() < 1e-10);
            }
        }

        // Test 3: RXX(π) should flip |00⟩ to |11⟩ up to phase
        let mut q = StateVec::new(2);
        q.rxx(PI, 0, 1);

        // Should get -i|11⟩
        assert!(q.state[0].norm() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);
        assert!(q.state[2].norm() < 1e-10);
        assert!((q.state[3] - Complex64::new(0.0, -1.0)).norm() < 1e-10);
    }

    #[test]
    fn test_rxx_symmetry() {
        // Test that RXX is symmetric under exchange of qubits
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);

        // Prepare same non-trivial initial state
        q1.h(0);
        q1.h(1);
        q2.h(0);
        q2.h(1);

        // Apply RXX with different qubit orders
        q1.rxx(FRAC_PI_3, 0, 1);
        q2.rxx(FRAC_PI_3, 1, 0);

        // Results should be identical
        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!((a - b).norm() < 1e-10);
        }
    }

    #[test]
    fn test_ryy() {
        let expected = FRAC_1_SQRT_2;

        // Test all basis states for RYY(π/2)
        // |00⟩ -> (1/√2)|00⟩ - i(1/√2)|11⟩
        let mut q = StateVec::new(2);
        q.ryy(FRAC_PI_2, 0, 1);
        assert!((q.state[0].re - expected).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);
        assert!(q.state[2].norm() < 1e-10);
        assert!((q.state[3].im - expected).abs() < 1e-10);

        // |11⟩ -> i(1/√2)|00⟩ + (1/√2)|11⟩
        let mut q = StateVec::new(2);
        q.x(0).x(1); // Prepare |11⟩
        q.ryy(FRAC_PI_2, 0, 1);
        assert!((q.state[0].im - expected).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);
        assert!(q.state[2].norm() < 1e-10);
        assert!((q.state[3].re - expected).abs() < 1e-10);

        // |01⟩ -> (1/√2)|01⟩ + i(1/√2)|10⟩
        let mut q = StateVec::new(2);
        q.x(1); // Prepare |01⟩
        q.ryy(FRAC_PI_2, 0, 1);
        assert!(q.state[0].norm() < 1e-10);
        assert!(q.state[1].re.abs() < 1e-10);
        assert!((q.state[1].im + expected).abs() < 1e-10);
        assert!((q.state[2].re - expected).abs() < 1e-10);
        assert!(q.state[2].im.abs() < 1e-10);
        assert!(q.state[3].norm() < 1e-10);

        // |10⟩ -> (1/√2)|10⟩ + i(1/√2)|01⟩
        let mut q = StateVec::new(2);
        q.x(0); // Prepare |10⟩
        q.ryy(FRAC_PI_2, 0, 1);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].re - expected).abs() < 1e-10);
        assert!(q.state[1].im.abs() < 1e-10);
        assert!(q.state[2].re.abs() < 1e-10);
        assert!((q.state[2].im + expected).abs() < 1e-10);
        assert!(q.state[3].norm() < 1e-10);

        // Test properties

        // 1. Periodicity: RYY(2π) = I
        let mut q = StateVec::new(2);
        q.h(0); // Create non-trivial initial state
        let initial = q.state.clone();
        q.ryy(TAU, 0, 1);
        // Need to account for potential global phase
        if q.state[0].norm() > 1e-10 {
            let phase = q.state[0] / initial[0];
            for (a, b) in q.state.iter().zip(initial.iter()) {
                assert!(
                    (a - b * phase).norm() < 1e-10,
                    "Periodicity test failed: a={a}, b={b}"
                );
            }
        }

        // 2. Composition: RYY(θ₁)RYY(θ₂) = RYY(θ₁ + θ₂)
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);
        q1.h(0); // Create non-trivial initial state
        q2.h(0); // Same initial state
        q1.ryy(FRAC_PI_3, 0, 1).ryy(FRAC_PI_6, 0, 1);
        q2.ryy(FRAC_PI_2, 0, 1);
        // Compare up to global phase
        if q1.state[0].norm() > 1e-10 {
            let phase = q1.state[0] / q2.state[0];
            for (a, b) in q1.state.iter().zip(q2.state.iter()) {
                assert!(
                    (a - b * phase).norm() < 1e-10,
                    "Composition test failed: a={a}, b={b}"
                );
            }
        }

        // 3. Symmetry: RYY(θ,0,1) = RYY(θ,1,0)
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);
        q1.h(0).h(1); // Create non-trivial initial state
        q2.h(0).h(1); // Same initial state
        q1.ryy(FRAC_PI_3, 0, 1);
        q2.ryy(FRAC_PI_3, 1, 0);
        // States should be exactly equal (no phase difference)
        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!((a - b).norm() < 1e-10, "Symmetry test failed: a={a}, b={b}");
        }
    }

    #[test]
    fn test_ryy_qubit_order_invariance() {
        let theta = FRAC_PI_4;

        // Test on random initial states
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);
        q1.h(0).x(1); // Random state
        q2.h(0).x(1); // Same initial state

        q1.ryy(theta, 0, 1);
        q2.ryy(theta, 1, 0);

        // States should be exactly equal
        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!(
                (a - b).norm() < 1e-10,
                "Qubit order test failed: a={a}, b={b}"
            );
        }
    }

    #[test]
    fn test_ryy_large_system() {
        let theta = FRAC_PI_3;

        // Initialize a 5-qubit state
        let mut q = StateVec::new(5);
        q.h(0).h(1).h(2).h(3).h(4); // Superposition state

        // Apply RYY on qubits 2 and 4
        q.ryy(theta, 2, 4);

        // Ensure state vector normalization is preserved
        let norm: f64 = q.state.iter().map(num_complex::Complex::norm_sqr).sum();
        assert!(
            (norm - 1.0).abs() < 1e-10,
            "State normalization test failed: norm={norm}"
        );
    }

    fn assert_state_vectors_match(simulated: &[Complex64], expected: &[Complex64], tolerance: f64) {
        assert_eq!(
            simulated.len(),
            expected.len(),
            "State vectors must have the same length"
        );

        // Find the first non-zero entry in the expected vector
        let reference_index = expected
            .iter()
            .position(|&x| x.norm() > tolerance)
            .expect("Expected vector should have at least one non-zero element");

        // Compute the phase correction
        let phase = simulated[reference_index] / expected[reference_index];

        // Verify all elements match up to the global phase
        for (i, (sim, exp)) in simulated.iter().zip(expected.iter()).enumerate() {
            let corrected_exp = exp * phase;
            assert!(
                (sim - corrected_exp).norm() < tolerance,
                "Mismatch at index {i}: simulated = {sim:?}, expected = {exp:?}, corrected = {corrected_exp:?}"
            );
        }
    }

    #[test]
    fn test_ryy_edge_cases() {
        let mut q = StateVec::new(2);

        // Apply RYY gate
        q.ryy(PI, 0, 1);

        // Define the expected result for RYY(π)
        let expected = vec![
            Complex64::new(0.0, 0.0),  // |00⟩
            Complex64::new(0.0, 0.0),  // |01⟩
            Complex64::new(0.0, 0.0),  // |10⟩
            Complex64::new(-1.0, 0.0), // |11⟩
        ];

        // Compare simulated state vector to the expected result
        assert_state_vectors_match(&q.state, &expected, 1e-10);
    }

    #[test]
    fn test_ryy_global_phase() {
        let mut q = StateVec::new(2);

        q.ryy(PI, 0, 1);

        // Define the expected result for RYY(π)
        let expected = vec![
            Complex64::new(0.0, 0.0),  // |00⟩
            Complex64::new(0.0, 0.0),  // |01⟩
            Complex64::new(0.0, 0.0),  // |10⟩
            Complex64::new(-1.0, 0.0), // |11⟩
        ];

        // Compare states
        assert_states_equal(&q.state, &expected);
    }

    #[test]
    fn test_ryy_small_angles() {
        let theta = 1e-10; // Very small angle
        let mut q = StateVec::new(2);

        // Initialize |00⟩
        let initial = q.state.clone();
        q.ryy(theta, 0, 1);

        // Expect state to remain close to the initial state
        for (a, b) in q.state.iter().zip(initial.iter()) {
            assert!(
                (a - b).norm() < 1e-10,
                "Small angle test failed: a={a}, b={b}"
            );
        }
    }

    #[test]
    fn test_ryy_randomized() {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let theta = rng.gen_range(0.0..2.0 * PI);

        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);

        // Random initial state
        q1.h(0).h(1);
        q2.h(0).h(1);

        // Apply RYY with random qubit order
        q1.ryy(theta, 0, 1);
        q2.ryy(theta, 1, 0);

        // Compare states
        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!(
                (a - b).norm() < 1e-10,
                "Randomized test failed: a={a}, b={b}"
            );
        }
    }

    #[test]
    fn test_rzz() {
        // Test 1: RZZ(π) on (|00⟩ + |11⟩)/√2 should give itself
        let mut q = StateVec::new(2);
        // Create Bell state
        q.h(0);
        q.cx(0, 1);
        let initial = q.state.clone();

        q.rzz(PI, 0, 1);

        // Compare up to global phase
        if q.state[0].norm() > 1e-10 {
            let phase = q.state[0] / initial[0];
            for (a, b) in q.state.iter().zip(initial.iter()) {
                assert!((a - b * phase).norm() < 1e-10);
            }
        }

        // Test 2: RZZ(π/2) on |++⟩
        let mut q = StateVec::new(2);
        q.h(0);
        q.h(1);
        q.rzz(FRAC_PI_2, 0, 1);

        // e^(-iπ/4) = (1-i)/√2
        // e^(iπ/4) = (1+i)/√2
        let factor = 0.5; // 1/2 for the |++⟩ normalization
        let exp_minus_i_pi_4 = Complex64::new(1.0, -1.0) / (2.0_f64.sqrt());
        let exp_plus_i_pi_4 = Complex64::new(1.0, 1.0) / (2.0_f64.sqrt());

        assert!((q.state[0] - factor * exp_minus_i_pi_4).norm() < 1e-10); // |00⟩
        assert!((q.state[1] - factor * exp_plus_i_pi_4).norm() < 1e-10); // |01⟩
        assert!((q.state[2] - factor * exp_plus_i_pi_4).norm() < 1e-10); // |10⟩
        assert!((q.state[3] - factor * exp_minus_i_pi_4).norm() < 1e-10); // |11⟩
    }

    #[test]
    fn test_szz_equivalence() {
        // Test that SZZ is equivalent to RZZ(π/2)
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);

        // Create some non-trivial initial state
        q1.h(0);
        q2.h(0);

        // Compare direct SZZ vs RZZ(π/2)
        q1.szz(0, 1);
        q2.rzz(FRAC_PI_2, 0, 1);

        assert_states_equal(q1.state(), q2.state());

        // Also verify decomposition matches
        let mut q3 = StateVec::new(2);
        q3.h(0); // Same initial state
        q3.h(0).h(1).sxx(0, 1).h(0).h(1);

        assert_states_equal(q1.state(), q3.state());
    }

    #[test]
    fn test_szz_trait_equivalence() {
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);

        // Create some non-trivial initial state
        q1.h(0);
        q2.h(0);

        // Compare CliffordGateable trait szz vs ArbitraryRotationGateable trait rzz(π/2)
        CliffordGateable::<usize>::szz(&mut q1, 0, 1);
        ArbitraryRotationGateable::<usize>::rzz(&mut q2, PI / 2.0, 0, 1);

        assert_states_equal(q1.state(), q2.state());
    }

    #[test]
    fn test_rotation_symmetries() {
        // Test that all rotations are symmetric under exchange of qubits
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);

        // Prepare same non-trivial initial state
        q1.h(0);
        q1.h(1);
        q2.h(0);
        q2.h(1);

        let theta = PI / 3.0;

        // Test RYY symmetry
        q1.ryy(theta, 0, 1);
        q2.ryy(theta, 1, 0);

        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!((a - b).norm() < 1e-10);
        }

        // Test RZZ symmetry
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);
        q1.h(0);
        q1.h(1);
        q2.h(0);
        q2.h(1);

        q1.rzz(theta, 0, 1);
        q2.rzz(theta, 1, 0);

        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!((a - b).norm() < 1e-10);
        }
    }

    #[test]
    fn test_mz() {
        // Test 1: Measuring |0> state
        let mut q = StateVec::new(1);
        let result = q.mz(0);
        assert!(!result.outcome);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Test 2: Measuring |1> state
        let mut q = StateVec::new(1);
        q.x(0);
        let result = q.mz(0);
        assert!(result.outcome);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].re - 1.0).abs() < 1e-10);

        // Test 3: Measuring superposition state multiple times
        let mut zeros = 0;
        let trials = 1000;

        for _ in 0..trials {
            let mut q = StateVec::new(1);
            q.h(0);
            let result = q.mz(0);
            if !result.outcome {
                zeros += 1;
            }
        }

        // Check if measurements are roughly equally distributed
        let ratio = f64::from(zeros) / f64::from(trials);
        assert!((ratio - 0.5).abs() < 0.1); // Should be close to 0.5...

        // Test 4: Measuring one qubit of a Bell state
        let mut q = StateVec::new(2);
        q.h(0);
        q.cx(0, 1);

        // Measure first qubit
        let result1 = q.mz(0);
        // Measure second qubit - should match first
        let result2 = q.mz(1);
        assert_eq!(result1.outcome, result2.outcome);
    }

    #[test]
    fn test_reset() {
        let mut q = StateVec::new(1);

        q.h(0);
        assert!((q.state[0].re - FRAC_1_SQRT_2).abs() < 1e-10);
        assert!((q.state[1].re - FRAC_1_SQRT_2).abs() < 1e-10);

        q.pz(0);

        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);
    }

    #[test]
    fn test_reset_multiple_qubits() {
        let mut q = StateVec::new(2);

        q.h(0);
        q.cx(0, 1);

        q.pz(0);

        let prob_0 = q.state[0].norm_sqr() + q.state[2].norm_sqr();
        let prob_1 = q.state[1].norm_sqr() + q.state[3].norm_sqr();

        assert!((prob_0 - 1.0).abs() < 1e-10);
        assert!(prob_1 < 1e-10);
    }

    #[test]
    fn test_single_qubit_rotation() {
        let mut q = StateVec::new(1);

        // Test 1: Hadamard gate
        let h00 = Complex64::new(FRAC_1_SQRT_2, 0.0);
        let h01 = Complex64::new(FRAC_1_SQRT_2, 0.0);
        let h10 = Complex64::new(FRAC_1_SQRT_2, 0.0);
        let h11 = Complex64::new(-FRAC_1_SQRT_2, 0.0);

        q.single_qubit_rotation(0, h00, h01, h10, h11);
        assert!((q.state[0].re - FRAC_1_SQRT_2).abs() < 1e-10);
        assert!((q.state[1].re - FRAC_1_SQRT_2).abs() < 1e-10);

        // Test 2: X gate
        let mut q = StateVec::new(1);
        let x00 = Complex64::new(0.0, 0.0);
        let x01 = Complex64::new(1.0, 0.0);
        let x10 = Complex64::new(1.0, 0.0);
        let x11 = Complex64::new(0.0, 0.0);

        q.single_qubit_rotation(0, x00, x01, x10, x11);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].re - 1.0).abs() < 1e-10);

        // Test 3: Phase gate
        let mut q = StateVec::new(1);
        let p00 = Complex64::new(1.0, 0.0);
        let p01 = Complex64::new(0.0, 0.0);
        let p10 = Complex64::new(0.0, 0.0);
        let p11 = Complex64::new(0.0, 1.0);

        q.single_qubit_rotation(0, p00, p01, p10, p11);
        assert!((q.state[0].re - 1.0).abs() < 1e-10);
        assert!(q.state[1].norm() < 1e-10);

        // Test 4: Y gate using unitary
        let mut q = StateVec::new(1);
        let y00 = Complex64::new(0.0, 0.0);
        let y01 = Complex64::new(0.0, -1.0);
        let y10 = Complex64::new(0.0, 1.0);
        let y11 = Complex64::new(0.0, 0.0);

        q.single_qubit_rotation(0, y00, y01, y10, y11);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].im - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sq_rotation_edge_cases() {
        let mut q = StateVec::new(1);

        // Test RX(0): Should be identity
        let initial = q.state.clone();
        q.rx(0.0, 0);
        assert_states_equal(&q.state, &initial);

        // Test RX(2π): Should also be identity up to global phase
        q.rx(2.0 * PI, 0);
        assert_states_equal(&q.state, &initial);

        // Test RY(0): Should be identity
        q.ry(0.0, 0);
        assert_states_equal(&q.state, &initial);

        // Test RY(2π): Should also be identity up to global phase
        q.ry(2.0 * PI, 0);
        assert_states_equal(&q.state, &initial);

        // Test RZ(0): Should be identity
        q.rz(0.0, 0);
        assert_states_equal(&q.state, &initial);

        // Test RZ(2π): Should also be identity up to global phase
        q.rz(2.0 * PI, 0);
        assert_states_equal(&q.state, &initial);
    }

    #[test]
    fn test_unitary_properties() {
        let mut q = StateVec::new(1);

        // Create random state with Hadamard
        q.h(0);

        // Apply Z gate as unitary
        let z00 = Complex64::new(1.0, 0.0);
        let z01 = Complex64::new(0.0, 0.0);
        let z10 = Complex64::new(0.0, 0.0);
        let z11 = Complex64::new(-1.0, 0.0);

        let initial = q.state.clone();
        q.single_qubit_rotation(0, z00, z01, z10, z11);

        // Check normalization is preserved
        let norm: f64 = q.state.iter().map(num_complex::Complex::norm_sqr).sum();
        assert!((norm - 1.0).abs() < 1e-10);

        // Apply Z again - should get back original state
        q.single_qubit_rotation(0, z00, z01, z10, z11);

        for (a, b) in q.state.iter().zip(initial.iter()) {
            assert!((a - b).norm() < 1e-10);
        }
    }

    #[test]
    fn test_u_special_cases() {
        // Test 1: U(π, 0, π) should be X gate
        let mut q = StateVec::new(1);
        q.u(PI, 0.0, PI, 0);
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[1].re - 1.0).abs() < 1e-10);

        // Test 2: Hadamard gate
        // H = U(π/2, 0, π)
        let mut q = StateVec::new(1);
        q.u(PI / 2.0, 0.0, PI, 0);
        assert!((q.state[0].re - FRAC_1_SQRT_2).abs() < 1e-10);
        assert!((q.state[1].re - FRAC_1_SQRT_2).abs() < 1e-10);

        // Test 3: U(0, 0, π) should be Z gate
        let mut q = StateVec::new(1);
        q.h(0); // First put in superposition
        let initial = q.state.clone();
        q.u(0.0, 0.0, PI, 0);
        assert!((q.state[0] - initial[0]).norm() < 1e-10);
        assert!((q.state[1] + initial[1]).norm() < 1e-10);

        // Additional test: U3(π/2, π/2, -π/2) should be S†H
        let mut q = StateVec::new(1);
        q.u(PI / 2.0, PI / 2.0, -PI / 2.0, 0);
        // This creates the state (|0⟩ + i|1⟩)/√2
        assert!((q.state[0].re - FRAC_1_SQRT_2).abs() < 1e-10);
        assert!((q.state[1].im - FRAC_1_SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn test_u_composition() {
        let mut q1 = StateVec::new(1);
        let q2 = StateVec::new(1);

        // Two U gates that should multiply to identity
        q1.u(PI / 3.0, PI / 4.0, PI / 6.0, 0);
        q1.u(-PI / 3.0, -PI / 6.0, -PI / 4.0, 0);

        // Compare with initial state
        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!((a - b).norm() < 1e-10);
        }
    }

    #[test]
    fn test_u_arbitrary() {
        let mut q = StateVec::new(1);

        // Apply some arbitrary rotation
        let theta = PI / 5.0;
        let phi = PI / 7.0;
        let lambda = PI / 3.0;
        q.u(theta, phi, lambda, 0);

        // Verify normalization is preserved
        let norm: f64 = q.state.iter().map(num_complex::Complex::norm_sqr).sum();
        assert!((norm - 1.0).abs() < 1e-10);

        // Verify expected amplitudes
        let expected_0 = (theta / 2.0).cos();
        assert!((q.state[0].re - expected_0).abs() < 1e-10);

        let expected_1_mag = (theta / 2.0).sin();
        assert!((q.state[1].norm() - expected_1_mag).abs() < 1e-10);
    }

    #[test]
    fn test_two_qubit_unitary_cnot() {
        // Test that we can implement CNOT using the general unitary
        let mut q1 = StateVec::new(2);
        let mut q2 = StateVec::new(2);

        // CNOT matrix
        let cnot = [
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
        ];

        // Create Bell state using both methods
        q1.h(0);
        q1.cx(0, 1);

        q2.h(0);
        q2.two_qubit_unitary(0, 1, cnot);

        // Compare results
        for (a, b) in q1.state.iter().zip(q2.state.iter()) {
            assert!((a - b).norm() < 1e-10);
        }
    }

    #[test]
    fn test_two_qubit_unitary_swap() {
        // Test SWAP gate
        let mut q = StateVec::new(2);

        // Prepare |10⟩ state
        q.x(0);

        // SWAP matrix
        let swap = [
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
            ],
        ];

        q.two_qubit_unitary(0, 1, swap);

        // Should be in |01⟩ state
        assert!(q.state[0].norm() < 1e-10);
        assert!((q.state[2].re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_two_qubit_unitary_properties() {
        let mut q = StateVec::new(2);

        // Create a non-trivial state
        q.h(0);
        q.h(1);

        // iSWAP matrix
        let iswap = [
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 1.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 1.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
            ],
        ];

        q.two_qubit_unitary(0, 1, iswap);

        // Verify normalization is preserved
        let norm: f64 = q.state.iter().map(num_complex::Complex::norm_sqr).sum();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_single_qubit_locality() {
        // Test on 3 qubit system that gates only affect their target
        let mut q = StateVec::new(3);

        // Prepare state |+⟩|0⟩|0⟩
        q.h(0); // Affects least significant bit

        // Apply X to qubit 2 (most significant bit)
        q.x(2);

        // Check that qubit 0 is still in |+⟩ state
        // When qubit 2 is |1⟩, check LSB still shows |+⟩
        assert!((q.state[4].re - FRAC_1_SQRT_2).abs() < 1e-10); // |100⟩
        assert!((q.state[5].re - FRAC_1_SQRT_2).abs() < 1e-10); // |101⟩
    }

    #[test]
    fn test_bit_indexing() {
        let mut q = StateVec::new(3);

        println!("Initial state (|000⟩):");
        for i in 0..8 {
            println!("  {:03b}: {:.3}", i, q.state[i]);
        }

        // Put |+⟩ on qubit 0 (LSB)
        q.h(0);

        println!("\nAfter H on qubit 0:");
        for i in 0..8 {
            println!("  {:03b}: {:.3}", i, q.state[i]);
        }

        // Check state is |+⟩|0⟩|0⟩
        // Only indices that differ in LSB (qubit 0) should be FRAC_1_SQRT_2
        for i in 0..8 {
            let qubit0 = i & 1;
            let qubit1 = (i >> 1) & 1;
            let qubit2 = (i >> 2) & 1;

            let expected = if qubit1 == 0 && qubit2 == 0 {
                FRAC_1_SQRT_2
            } else {
                0.0
            };

            if (q.state[i].re - expected).abs() >= 1e-10 {
                println!("\nMismatch at index {i}: {i:03b}");
                println!("Qubit values: q2={qubit2}, q1={qubit1}, q0={qubit0}");
                println!("Expected {}, got {}", expected, q.state[i].re);
            }
            assert!((q.state[i].re - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_two_qubit_locality() {
        let mut q = StateVec::new(4);

        println!("Initial state:");
        for i in 0..16 {
            println!("  {:04b}: {:.3}", i, q.state[i]);
        }

        // Prepare |+⟩ on qubit 0 (LSB)
        q.h(0);

        println!("\nAfter H on qubit 0:");
        for i in 0..16 {
            println!("  {:04b}: {:.3}", i, q.state[i]);
        }

        // Apply CX between qubits 2,3
        q.cx(2, 3);

        println!("\nAfter CX on qubits 2,3:");
        for i in 0..16 {
            println!("  {:04b}: {:.3}", i, q.state[i]);

            // Extract qubit values
            // let _q0 = i & 1;
            let q1 = (i >> 1) & 1;
            let q2 = (i >> 2) & 1;
            let q3 = (i >> 3) & 1;

            // Only states with q0=0 or q0=1 and q1=q2=q3=0 should have amplitude
            let expected = if q1 == 0 && q2 == 0 && q3 == 0 {
                FRAC_1_SQRT_2
            } else {
                0.0
            };

            if (q.state[i].re - expected).abs() >= 1e-10 {
                println!("Mismatch at {i:04b}");
                println!("Expected {}, got {}", expected, q.state[i].re);
            }
            assert!((q.state[i].re - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_two_qubit_gate_locality() {
        let mut q = StateVec::new(3);

        // Prepare state |+⟩|0⟩|0⟩
        q.h(0);

        // Apply CX on qubits 1 and 2 (no effect on qubit 0)
        q.cx(1, 2);

        // Qubit 0 should remain in superposition
        let expected_amp = 1.0 / 2.0_f64.sqrt();
        assert!((q.state[0].re - expected_amp).abs() < 1e-10);
        assert!((q.state[1].re - expected_amp).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_locality() {
        let mut q = StateVec::new(3);

        println!("Initial state:");
        for i in 0..8 {
            println!("  {:03b}: {:.3}", i, q.state[i]);
        }

        // Prepare |+⟩ on qubit 0 (LSB)
        q.h(0);

        println!("\nAfter H on qubit 0:");
        for i in 0..8 {
            println!("  {:03b}: {:.3}", i, q.state[i]);
        }

        // Apply rotation to qubit 1
        q.rx(PI / 2.0, 1);

        println!("\nAfter RX on qubit 1:");
        for i in 0..8 {
            println!("  {:03b}: {:.3}", i, q.state[i]);
        }

        // Check each basis state contribution
        for i in 0..8 {
            let expected = FRAC_1_SQRT_2;
            if (q.state[i].norm() - expected).abs() >= 1e-10 {
                println!("\nMismatch at index {i}: {i:03b}");
                println!("Expected norm {}, got {}", expected, q.state[i].norm());
            }
        }
    }

    #[test]
    fn test_large_system() {
        // Test with a large number of qubits to ensure robustness.
        let num_qubits = 20; // 20 qubits => 2^20 amplitudes (~1M complex numbers)
        let mut q = StateVec::new(num_qubits);

        // Apply Hadamard to the first qubit
        q.h(0);

        // Check normalization and amplitudes for |0...0> and |1...0>
        let expected_amp = 1.0 / (2.0_f64.sqrt());
        assert!((q.state[0].norm() - expected_amp).abs() < 1e-10);
        assert!((q.state[1].norm() - expected_amp).abs() < 1e-10);

        // Ensure all other amplitudes remain zero
        for i in 2..q.state.len() {
            assert!(q.state[i].norm() < 1e-10);
        }
    }

    #[test]
    fn test_measurement_consistency() {
        let mut q = StateVec::new(1);

        // Put qubit in |1⟩ state
        q.x(0);

        // Measure twice - result should be the same
        let result1 = q.mz(0);
        let result2 = q.mz(0);

        assert!(result1.outcome);
        assert!(result2.outcome);
    }

    #[test]
    fn test_measurement_on_entangled_state() {
        let mut q = StateVec::new(2);

        // Create Bell state (|00⟩ + |11⟩) / sqrt(2)
        q.h(0);
        q.cx(0, 1);

        // Measure the first qubit
        let result1 = q.mz(0);

        // Measure the second qubit - should match the first
        let result2 = q.mz(1);

        assert_eq!(result1.outcome, result2.outcome);
    }
}
