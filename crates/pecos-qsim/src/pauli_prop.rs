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

#![allow(unused_variables)]

use super::clifford_gateable::{CliffordGateable, MeasurementResult};
use crate::quantum_simulator::QuantumSimulator;
use core::marker::PhantomData;
use pecos_core::{IndexableElement, Set, VecSet};

// TODO: Allow for the use of sets of elements of types other than usize

/// Type alias for the most common use case of `PauliProp` with standard vectors
#[expect(clippy::module_name_repetitions)]
pub type StdPauliProp = PauliProp<VecSet<usize>, usize>;

/// A simulator that tracks how Pauli operators transform under Clifford operations.
///
/// # Overview
/// The `PauliProp` simulator efficiently tracks the evolution of Pauli operators (X, Y, Z)
/// through Clifford quantum operations without maintaining the full quantum state. This makes
/// it particularly useful for:
/// - Simulating Pauli noise propagation in quantum circuits
/// - Tracking the evolution of Pauli observables
/// - Analyzing stabilizer states
/// - Verifying Clifford circuit implementations
///
/// # State Representation
/// The simulator maintains two sets to track Pauli operators:
/// - `xs`: Records qubits with X Pauli operators
/// - `zs`: Records qubits with Z Pauli operators
///
/// Y operators are implicitly represented by qubits present in both sets since Y = iXZ.
/// The sign/phase of the operators is not tracked as it's often not relevant for the
/// intended use cases.
///
/// # Type Parameters
/// - `T`: The set type used to store qubit indices (e.g., `VecSet`\<usize\>)
/// - `E`: The element type used for qubit indices (e.g., usize)
///
/// # Example
/// ```rust
/// use pecos_qsim::{StdPauliProp, CliffordGateable};
///
/// let mut sim = StdPauliProp::new();
/// sim.add_x(0);  // Track an X on qubit 0
/// sim.h(0);         // Apply Hadamard - transforms X to Z
/// assert!(sim.contains_z(0));  // Verify qubit 0 now has Z
/// ```
///
/// # Performance Characteristics
/// - Space complexity: O(n) where n is the number of qubits with non-identity operators
/// - Time complexity: O(1) for most gates
///
/// # References
/// - Gottesman, "The Heisenberg Representation of Quantum Computers"
///   <https://arxiv.org/abs/quant-ph/9807006>
#[derive(Clone, Debug)]
pub struct PauliProp<T, E>
where
    T: for<'a> Set<'a, Element = E>,
    E: IndexableElement,
{
    xs: T,
    zs: T,
    _marker: PhantomData<E>,
}

// TODO: Optionally track the sign of the operator

impl<T, E> Default for PauliProp<T, E>
where
    E: IndexableElement,
    T: for<'a> Set<'a, Element = E>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E> PauliProp<T, E>
where
    E: IndexableElement,
    T: for<'a> Set<'a, Element = E>,
{
    /// Creates a new `PauliProp` simulator for the specified number of qubits.
    ///
    /// The simulator is initialized with no Pauli operators as the user needs to specify what
    /// observables to track.
    ///
    /// # Arguments
    /// * `num_qubits` - The total number of qubits to simulate
    ///
    /// # Returns
    /// A new `PauliProp` instance configured for the specified number of qubits
    #[must_use]
    pub fn new() -> Self {
        PauliProp {
            xs: T::new(),
            zs: T::new(),
            _marker: PhantomData,
        }
    }
}

impl<T, E> QuantumSimulator for PauliProp<T, E>
where
    E: IndexableElement,
    T: for<'a> Set<'a, Element = E>,
{
    /// Resets the state by clearing all Pauli all tracked X and Z operators.
    ///
    /// # Returns
    /// * `&mut Self` - Returns self for method chaining
    #[inline]
    fn reset(&mut self) -> &mut Self {
        self.xs.clear();
        self.zs.clear();
        self
    }
}

impl<T, E> PauliProp<T, E>
where
    T: for<'a> Set<'a, Element = E>,
    E: IndexableElement,
{
    /// Checks if the specified qubit has an X operator.
    ///
    /// # Arguments
    /// * `item` - The qubit index to check
    ///
    /// # Returns
    /// `true` if an X operator is present on the qubit
    #[inline]
    pub fn contains_x(&self, item: E) -> bool {
        self.xs.contains(&item)
    }

    /// Checks if the specified qubit has a Z operator.
    ///
    /// # Arguments
    /// * `item` - The qubit index to check
    ///
    /// # Returns
    /// `true` if a Z operator is present on the qubit
    #[inline]
    pub fn contains_z(&self, item: E) -> bool {
        self.zs.contains(&item)
    }

    /// Checks if the specified qubit has a Y operator.
    ///
    /// Since Y = iXZ, this checks for the presence of both X and Z operators.
    ///
    /// # Arguments
    /// * `item` - The qubit index to check
    ///
    /// # Returns
    /// `true` if both X and Z operators are present on the qubit
    #[inline]
    pub fn contains_y(&self, item: E) -> bool {
        self.contains_x(item) && self.contains_z(item)
    }

    /// Adds an X Pauli operator to be tracked to the specified qubit
    ///
    /// If the qubit already has:
    /// - No operator: Adds X
    /// - X operator: Removes X
    /// - Z operator: Creates Y (iXZ)
    /// - Y operator: Creates Z
    ///
    /// # Arguments
    /// * `item` - The qubit index to add the X operator to
    #[inline]
    pub fn add_x(&mut self, item: E) {
        self.xs.symmetric_difference_item_update(&item);
    }

    /// Adds a Z operator to the specified qubit.
    ///
    /// If the qubit already has:
    /// - No operator: Adds Z
    /// - Z operator: Removes Z
    /// - X operator: Creates Y (iXZ)
    /// - Y operator: Creates X
    ///
    /// # Arguments
    /// * `item` - The qubit index to add the Z operator to
    #[inline]
    pub fn add_z(&mut self, item: E) {
        self.zs.symmetric_difference_item_update(&item);
    }

    /// Adds a Y operator to the specified qubit.
    ///
    /// Since Y = iXZ, this adds both X and Z operators to the qubit.
    ///
    /// If the qubit already has:
    /// - No operator: Creates Y (Creates X and Z)
    /// - X operator: Removes X, Creates Z
    /// - Z operator: Removes Z, Creates X
    /// - Y operator: Removes X and Z
    ///
    /// # Arguments
    /// * `item` - The qubit index to add the Y operator to
    #[inline]
    pub fn add_y(&mut self, item: E) {
        self.add_x(item);
        self.add_z(item);
    }
}

impl<T, E> CliffordGateable<E> for PauliProp<T, E>
where
    T: for<'a> Set<'a, Element = E>,
    E: IndexableElement,
{
    /// Applies the square root of Z gate (SZ or S gate) to the specified qubit.
    ///
    /// The SZ gate transforms Pauli operators as follows:
    /// ```text
    /// X -> Y
    /// Y -> -X
    /// Z -> Z
    /// ```
    ///
    /// Implementation: If the qubit has an X operator, toggle its Z operator
    ///
    /// # Arguments
    /// * `q` - The target qubit
    ///
    /// # Returns
    /// * `&mut Self` - Returns self for method chaining
    #[inline]
    fn sz(&mut self, q: E) -> &mut Self {
        if self.contains_x(q) {
            self.add_z(q);
        }
        self
    }

    /// Applies the Hadamard (H) gate to the specified qubit.
    ///
    /// The H gate transforms Pauli operators as follows:
    /// ```text
    /// X -> Z
    /// Z -> X
    /// Y -> -Y
    /// ```
    ///
    /// Implementation:
    /// - For X or Z: Swap between X and Z sets
    /// - For Y: Leave unchanged (Y transforms to -Y)
    ///
    /// # Arguments
    /// * `q` - The target qubit
    ///
    /// # Returns
    /// * `&mut Self` - Returns self for method chaining
    #[inline]
    #[expect(clippy::similar_names)]
    fn h(&mut self, q: E) -> &mut Self {
        let in_xs = self.contains_x(q);
        let in_zs = self.contains_z(q);

        if in_xs && in_zs {
        } else if in_xs {
            self.xs.remove(&q);
            self.zs.insert(q);
        } else if in_zs {
            self.zs.remove(&q);
            self.xs.insert(q);
        }
        self
    }

    /// Applies the controlled-X (CX) gate between two qubits
    ///
    /// The CX gate transforms Pauli operators as follows:
    /// ```text
    /// XI -> XX  (X on control propagates to target)
    /// IX -> IX  (X on target unchanged)
    /// ZI -> ZI  (Z on control unchanged)
    /// IZ -> ZZ  (Z on target propagates to control)
    /// ```
    ///
    /// Implementation:
    /// - If control has X: Toggle X on target
    /// - If target has Z: Toggle Z on control
    ///
    /// # Arguments
    /// * `q1` - The control qubit
    /// * `q2` - The target qubit
    ///
    /// # Returns
    /// * `&mut Self` - Returns self for method chaining
    #[inline]
    fn cx(&mut self, q1: E, q2: E) -> &mut Self {
        if self.contains_x(q1) {
            self.add_x(q2);
        }
        if self.contains_z(q2) {
            self.add_z(q1);
        }
        self
    }

    /// Performs a Z-basis measurement on the specified qubit.
    ///
    /// This simulates the effect of Pauli operators on measurement due to propagation.
    /// The outcome indicates whether an X operator has propagated to the measured
    /// qubit, which would flip the measurement result in the Z basis.
    ///
    /// Note: The outcomes are not actual measurements of the state but detect only if introduced
    /// operators might flip the value of measures and only correspond to valid measurements if they
    /// are originally deterministic.
    ///
    /// # Arguments
    /// * `q` - The qubit to measure
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if an X operator is present (measurement flipped)
    ///   - `is_deterministic`: always true for this simulator
    #[inline]
    fn mz(&mut self, q: E) -> MeasurementResult {
        let outcome = self.contains_x(q);
        MeasurementResult {
            outcome,
            is_deterministic: true,
        }
    }
}
