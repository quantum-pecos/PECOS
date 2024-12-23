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

use super::quantum_simulator::QuantumSimulator;
use pecos_core::IndexableElement;

pub struct MeasurementResult {
    pub outcome: bool,
    pub is_deterministic: bool,
}

/// A simulator trait for quantum systems that implement Clifford operations.
///
/// # Overview
/// The Clifford group is a set of quantum operations that map Pauli operators to Pauli operators
/// under conjugation. A Clifford operation C transforms a Pauli operator P as:
/// ```text
/// C P C† = P'
/// ```
/// where P' is another Pauli operator (possibly with a phase ±1 or ±i).
///
/// # Gate Set
/// This trait provides:
///
/// ## Single-qubit gates
/// - Pauli gates (X, Y, Z)
/// - Hadamard (H) and variants (H2-H6)
/// - Phase gates (SX, SY, SZ) and their adjoints
/// - Face (F) gates and variants (F, F2-F4) and their adjoints
///
/// ## Two-qubit gates
/// - CNOT (CX)
/// - Controlled-Y (CY)
/// - Controlled-Z (CZ)
/// - SWAP
/// - √XX, √YY, √ZZ and their adjoints
/// - G (a two-qubit Clifford)
///
/// ## Measurements and Preparations
/// - Measurements in X, Y, Z bases (including ± variants)
/// - State preparations in X, Y, Z bases (including ± variants)
///
/// # Type Parameters
/// - `T`: An indexable element type that can convert between qubit indices and usizes
///
/// # Gate Transformations
/// Gates transform Pauli operators according to their Heisenberg representation. For example:
///
/// Hadamard (H):
/// ```text
/// X → Z
/// Z → X
/// Y → -Y
/// ```
///
/// CNOT (with control c and target t):
/// ```text
/// Xc⊗It → Xc⊗Xt
/// Ic⊗Xt → Ic⊗Xt
/// Zc⊗It → Zc⊗It
/// Ic⊗Zt → Zc⊗Zt
/// ```
///
/// # Measurement Semantics
/// - Measurements return a `MeasurementResult` containing:
///   - outcome: true for +1 eigenstate, false for -1 eigenstate
///   - deterministic: true if state was already in an eigenstate
///
/// # Examples
/// ```rust
/// use pecos_qsim::{CliffordGateable, StdSparseStab};
/// let mut sim = StdSparseStab::new(2);
///
/// // Create Bell state
/// sim.h(0).cx(0, 1);
///
/// // Measure in Z basis
/// let outcome = sim.mz(0);
/// ```
///
/// # Required Implementations
/// When implementing this trait, the following methods must be provided:
/// - `sz()`: Square root of Z gate (S or P gate)
/// - `h()`: Hadamard gate
/// - `cx()`: Controlled-NOT gate
/// - `mz()`: Z-basis measurement
///
/// All other operations have default implementations in terms of these basic gates.
/// Implementors may override any default implementation for efficiency.
///
/// # References
/// - Gottesman, "The Heisenberg Representation of Quantum Computers"
///   <https://arxiv.org/abs/quant-ph/9807006>
#[expect(clippy::min_ident_chars)]
pub trait CliffordGateable<T: IndexableElement>: QuantumSimulator {
    /// Applies the identity gate (I) to the specified qubit.
    ///
    /// The identity gate leaves the state unchanged.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → X
    /// Y → Y
    /// Z → Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// I = [[1, 0],
    ///      [0, 1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.identity(0); // State remains unchanged
    /// ```
    #[inline]
    fn identity(&mut self, _q: T) -> &mut Self {
        self
    }

    /// Applies a Pauli X (NOT) gate to the specified qubit.
    ///
    /// The X gate is equivalent to a classical NOT operation in the computational basis.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → X
    /// Y → -Y
    /// Z → -Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// X = [[0, 1],
    ///      [1, 0]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.x(0)   // Apply X gate to qubit 0
    ///    .h(0);  // Then apply H gate
    /// ```
    #[inline]
    fn x(&mut self, q: T) -> &mut Self {
        self.h(q).z(q).h(q)
    }

    /// Applies a Pauli Y gate to the specified qubit.
    ///
    /// The Y gate is a rotation by π radians around the Y axis of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -X
    /// Y → Y
    /// Z → -Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// Y = [[ 0, -i],
    ///      [+i,  0]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.y(0)   // Apply Y gate to qubit 0
    ///    .h(0);  // Then apply H gate
    /// ```
    #[inline]
    fn y(&mut self, q: T) -> &mut Self {
        self.z(q).x(q)
    }

    /// Applies a Pauli Z gate to the specified qubit.
    ///
    /// The Z gate applies a phase flip in the computational basis.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -X
    /// Y → -Y
    /// Z → Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// Z = [[1,  0],
    ///      [0, -1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.z(0)   // Apply Z gate to qubit 0
    ///    .h(0);  // Then apply H gate
    /// ```
    #[inline]
    fn z(&mut self, q: T) -> &mut Self {
        self.sz(q).sz(q)
    }

    /// Applies a square root of X (SX) gate to the specified qubit.
    ///
    /// The SX gate is equivalent to a rotation by π/2 radians around the X axis
    /// of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → X
    /// Y → -Z
    /// Z → Y
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SX = 1/2 [[1+i, 1-i],
    ///           [1-i, 1+i]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.sx(0)   // Apply SX gate to qubit 0
    ///    .h(0);   // Then apply H gate
    /// ```
    #[inline]
    fn sx(&mut self, q: T) -> &mut Self {
        self.h(q).sz(q).h(q)
    }

    /// Applies the adjoint (inverse) of the square root of X gate.
    ///
    /// The SX† gate is equivalent to a rotation by -π/2 radians around the X axis
    /// of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → X
    /// Y → Z
    /// Z → -Y
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SX† = 1/2 [[1-i, 1+i],
    ///            [1+i, 1-i]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.sxdg(0)   // Apply SX† gate to qubit 0
    ///    .h(0);     // Then apply H gate
    /// ```
    #[inline]
    fn sxdg(&mut self, q: T) -> &mut Self {
        self.h(q).szdg(q).h(q)
    }

    /// Applies a square root of Y (SY) gate to the specified qubit. The SY gate is equivalent to a
    /// rotation by π/2 radians around the Y axis of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Z
    /// Y → Y
    /// Z → X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SY = 1/√2 [[1,  -1],
    ///            [1,   1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.sy(0)   // Apply SY gate to qubit 0
    ///    .h(0);   // Then apply H gate
    /// ```
    #[inline]
    fn sy(&mut self, q: T) -> &mut Self {
        self.h(q).x(q)
    }

    /// Applies the adjoint (inverse) of the square root of Y gate. The SY† gate is equivalent to a
    /// rotation by -π/2 radians around the Y axis of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Z
    /// Y → Y
    /// Z → -X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SY† = 1/√2 [[ 1,  1],
    ///            [-1,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.sydg(0)   // Apply SY† gate to qubit 0
    ///    .h(0);     // Then apply H gate
    /// ```
    #[inline]
    fn sydg(&mut self, q: T) -> &mut Self {
        self.x(q).h(q)
    }

    /// Applies a square root of Z (SZ) gate to the specified qubit. The SZ gate (also known as the
    /// S gate) is equivalent to a rotation by π/2 radians around the Z axis of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Y
    /// Y → -X
    /// Z → Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SZ = [[1, 0],
    ///       [0, i]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.sz(0)   // Apply SZ gate to qubit 0
    ///    .h(0);   // Then apply H gate
    /// ```
    fn sz(&mut self, q: T) -> &mut Self;

    /// Applies the adjoint (inverse) of the square root of Z gate. The SZ† gate is equivalent to a
    /// rotation by -π/2 radians around the Z axis of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Y
    /// Y → X
    /// Z → Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SZ† = [[1,  0],
    ///        [0, -i]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// sim.szdg(0)   // Apply SZ† gate to qubit 0
    ///    .h(0);     // Then apply H gate
    /// ```
    #[inline]
    fn szdg(&mut self, q: T) -> &mut Self {
        self.z(q).sz(q)
    }

    /// Applies the Hadamard gate (H or H1) to the specified qubit. The Hadamard gate creates an
    /// equal superposition of basis states and is fundamental to many quantum algorithms.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Z
    /// Y → -Y
    /// Z → X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// H = 1/√2 [[1,  1],
    ///           [1, -1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)     // Apply H gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    fn h(&mut self, q: T) -> &mut Self;

    /// Applies the H2 variant of the Hadamard gate to the specified qubit. H2 transforms between
    /// complementary measurement bases with an additional
    /// negative sign.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Z
    /// Y → -Y
    /// Z → -X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// H2 = 1/√2 [[ 1, -1],
    ///            [-1,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h2(0)    // Apply H2 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn h2(&mut self, q: T) -> &mut Self {
        self.sy(q).z(q)
    }

    /// Applies the H3 variant of the Hadamard gate to the specified qubit. H3 performs a basis
    /// transformation in the XY plane of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Y
    /// Y → X
    /// Z → -Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// H3 = 1/√2 [[1,  i],
    ///            [i,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h3(0)    // Apply H3 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn h3(&mut self, q: T) -> &mut Self {
        self.sz(q).y(q)
    }

    /// Applies the H4 variant of the Hadamard gate to the specified qubit. H4 combines an XY-plane
    /// rotation with negative signs.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Y
    /// Y → -X
    /// Z → -Z
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// H4 = 1/√2 [[ 1, -i],
    ///            [-i,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h4(0)    // Apply H4 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn h4(&mut self, q: T) -> &mut Self {
        self.sz(q).x(q)
    }

    /// Applies the H5 variant of the Hadamard gate to the specified qubit. H5 performs a basis
    /// transformation in the YZ plane of the Bloch sphere.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -X
    /// Y → Z
    /// Z → Y
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// H5 = 1/√2 [[-1,  1],
    ///            [ 1,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h5(0)    // Apply H5 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn h5(&mut self, q: T) -> &mut Self {
        self.sx(q).z(q)
    }

    /// Applies the H6 variant of the Hadamard gate to the specified qubit. H6 combines a YZ-plane
    /// rotation with negative signs.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -X
    /// Y → -Z
    /// Z → -Y
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// H6 = 1/√2 [[-1, -1],
    ///            [-1,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h6(0)    // Apply H6 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn h6(&mut self, q: T) -> &mut Self {
        self.sx(q).y(q)
    }

    /// Applies the Face gate (F or F1) to the specified qubit. The Face gate performs a cyclic
    /// permutation of the Pauli operators.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Y
    /// Y → Z
    /// Z → X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F = 1/√2 [[1,  -i],
    ///           [i,   1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.f(0)     // Apply F gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn f(&mut self, q: T) -> &mut Self {
        self.sx(q).sz(q)
    }

    /// Applies the adjoint of the Face gate (F† or F1†) to the specified qubit. F† performs a
    /// counter-clockwise cyclic permutation of the Pauli operators.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Z
    /// Y → X
    /// Z → Y
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F† = 1/√2 [[1,   i],
    ///            [-i,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.fdg(0)   // Apply F† gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn fdg(&mut self, q: T) -> &mut Self {
        self.szdg(q).sxdg(q)
    }

    /// Applies the F2 variant of the Face gate to the specified qubit. F2 performs a cyclic
    /// permutation of the Pauli operators with one negative sign.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Z
    /// Y → -X
    /// Z → Y
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F2 = 1/√2 [[-1,  -i],
    ///            [-i,   1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.f2(0)    // Apply F2 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn f2(&mut self, q: T) -> &mut Self {
        self.sxdg(q).sy(q)
    }

    /// Applies the adjoint of the F2 gate (F2†) to the specified qubit. F2† performs a cyclic
    /// permutation with one negative sign in reverse.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Y
    /// Y → Z
    /// Z → -X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F2† = 1/√2 [[-1,   i],
    ///            [ i,   1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.f2dg(0)  // Apply F2† gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn f2dg(&mut self, q: T) -> &mut Self {
        self.sydg(q).sx(q)
    }

    /// Applies the F3 variant of the Face gate to the specified qubit. F3 performs a cyclic
    /// permutation with two negative signs.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Y
    /// Y → -Z
    /// Z → -X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F3 = 1/√2 [[ 1,  -i],
    ///            [-i,  -1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.f3(0)    // Apply F3 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn f3(&mut self, q: T) -> &mut Self {
        self.sxdg(q).sz(q)
    }

    /// Applies the adjoint of the F3 gate (F3†) to the specified qubit. F3† performs a cyclic
    /// permutation with two negative signs in reverse.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Z
    /// Y → X
    /// Z → -Y
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F3† = 1/√2 [[ 1,   i],
    ///            [ i,  -1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.f3dg(0)  // Apply F3† gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn f3dg(&mut self, q: T) -> &mut Self {
        self.szdg(q).sx(q)
    }

    /// Applies the F4 variant of the Face gate to the specified qubit. F4 performs a cyclic
    /// permutation with three negative signs.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → Z
    /// Y → -Z
    /// Z → -X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F4 = 1/√2 [[-i,  -1],
    ///            [ 1,  -i]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.f4(0)    // Apply F4 gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn f4(&mut self, q: T) -> &mut Self {
        self.sz(q).sx(q)
    }

    /// Applies the adjoint of the F4 gate (F4†) to the specified qubit. F4† performs a reverse
    /// cyclic permutation of the Pauli operators.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// X → -Y
    /// Y → Z
    /// Z → -X
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// F4† = 1/√2 [[ i,   1],
    ///            [-1,   i]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.f4dg(0)  // Apply F4† gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate
    /// ```
    #[inline]
    fn f4dg(&mut self, q: T) -> &mut Self {
        self.sxdg(q).szdg(q)
    }

    /// Applies a controlled-X (CNOT) operation between two qubits. The CX gate flips the target
    /// qubit if the control qubit is in state |1⟩.
    ///
    /// # Arguments
    /// * `q1` - Control qubit index.
    /// * `q2` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → XX
    /// IX → IX
    /// ZI → ZI
    /// IZ → ZZ
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// CX = [[1, 0, 0, 0],
    ///       [0, 1, 0, 0],
    ///       [0, 0, 0, 1],
    ///       [0, 0, 1, 0]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)     // Apply H gate to qubit 0
    ///    .cx(0,1); // Then apply CX gate between qubits 0 and 1
    /// ```
    fn cx(&mut self, q1: T, q2: T) -> &mut Self;

    /// Applies a controlled-Y operation between two qubits. The CY gate applies a Y operation on
    /// the target qubit if the control qubit is in state |1⟩.
    ///
    /// # Arguments
    /// * `q1` - Control qubit index.
    /// * `q2` - Target qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → XY
    /// IX → IX
    /// ZI → ZI
    /// IZ → ZZ
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// CY = [[1,  0,  0,  0],
    ///       [0,  1,  0,  0],
    ///       [0,  0,  0, -i],
    ///       [0,  0, +i,  0]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)     // Apply H gate to qubit 0
    ///    .cy(0,1); // Then apply CY gate between qubits 0 and 1
    /// ```
    #[inline]
    fn cy(&mut self, q1: T, q2: T) -> &mut Self {
        self.sz(q2).cx(q1, q2).szdg(q2)
    }

    /// Applies a controlled-Z operation between two qubits. The CZ gate applies a phase of -1 when
    /// both qubits are in state |1⟩.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → XZ
    /// IX → ZX
    /// ZI → ZI
    /// IZ → IZ
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// CZ = [[1,  0,  0,  0],
    ///       [0,  1,  0,  0],
    ///       [0,  0,  1,  0],
    ///       [0,  0,  0, -1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)     // Apply H gate to qubit 0
    ///    .cz(0,1); // Then apply CZ gate between qubits 0 and 1
    /// ```
    #[inline]
    fn cz(&mut self, q1: T, q2: T) -> &mut Self {
        self.h(q2).cx(q1, q2).h(q2)
    }

    /// Applies a square root of XX (SXX) operation between two qubits. The SXX gate implements
    /// evolution under XX coupling for time π/4.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → XI
    /// IX → IX
    /// ZI → -YX
    /// IZ → -XY
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SXX = 1/√2 [[1,  0,  0, -i],
    ///             [0,  1, -i,  0],
    ///             [0, -i,  1,  0],
    ///             [-i, 0,  0,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)      // Apply H gate to qubit 0
    ///    .sxx(0,1); // Then apply SXX gate between qubits 0 and 1
    /// ```
    #[inline]
    fn sxx(&mut self, q1: T, q2: T) -> &mut Self {
        self.sx(q1).sx(q2).sydg(q1).cx(q1, q2).sy(q1)
    }

    /// Applies the adjoint of the square root of XX operation. The SXX† gate implements reverse
    /// evolution under XX coupling.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → XI
    /// IX → IX
    /// ZI → YX
    /// IZ → XY
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SXX† = 1/√2 [[1,  0,  0,  i],
    ///              [0,  1,  i,  0],
    ///              [0,  i,  1,  0],
    ///              [i,  0,  0,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)         // Apply H gate to qubit 0
    ///    .sxxdg(0,1); // Then apply SXX† gate between qubits 0 and 1
    /// ```
    #[inline]
    fn sxxdg(&mut self, q1: T, q2: T) -> &mut Self {
        self.x(q1).x(q2).sxx(q1, q2)
    }

    /// Applies a square root of YY (SYY) operation between two qubits. The SYY gate implements
    /// evolution under YY coupling for time π/4.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → -ZY
    /// IX → -YZ
    /// ZI → XY
    /// IZ → YX
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SYY = 1/√2 [[1,  0,   0, -i],
    ///             [0, -i,   1,  0],
    ///             [0,  1,  -i,  0],
    ///             [-i, 0,   0,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)      // Apply H gate to qubit 0
    ///    .syy(0,1); // Then apply SYY gate between qubits 0 and 1
    /// ```
    #[inline]
    fn syy(&mut self, q1: T, q2: T) -> &mut Self {
        self.szdg(q1).szdg(q2).sxx(q1, q2).sz(q1).sz(q2)
    }

    /// Applies the adjoint of the square root of YY operation. The SYY† gate implements reverse
    /// evolution under YY coupling.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → ZY
    /// IX → YZ
    /// ZI → -XY
    /// IZ → -YX
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SYY† = 1/√2 [[1,  0,  0,  i],
    ///              [0,  i,  1,  0],
    ///              [0,  1,  i,  0],
    ///              [i,  0,  0,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)         // Apply H gate to qubit 0
    ///    .syydg(0,1); // Then apply SYY† gate between qubits 0 and 1
    /// ```
    #[inline]
    fn syydg(&mut self, q1: T, q2: T) -> &mut Self {
        self.y(q1).y(q2).syy(q1, q2)
    }

    /// Applies a square root of ZZ (SZZ) operation between two qubits. The SZZ gate implements
    /// evolution under ZZ coupling for time π/4.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → YZ
    /// IX → ZY
    /// ZI → ZI
    /// IZ → IZ
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SZZ = e^(-iπ/4) [[1,  0,  0,  0],
    ///                  [0, -i,  0,  0],
    ///                  [0,  0, -i,  0],
    ///                  [0,  0,  0,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)      // Apply H gate to qubit 0
    ///    .szz(0,1); // Then apply SZZ gate between qubits 0 and 1
    /// ```
    #[inline]
    fn szz(&mut self, q1: T, q2: T) -> &mut Self {
        self.h(q1).h(q2).sxx(q1, q2).h(q1).h(q2)
    }

    /// Applies the adjoint of the square root of ZZ operation. The SZZ† gate implements
    /// reverse evolution under ZZ coupling.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → -YZ
    /// IX → -ZY
    /// ZI → ZI
    /// IZ → IZ
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SZZ† = e^(iπ/4) [[1,  0,  0,  0],
    ///                  [0,  i,  0,  0],
    ///                  [0,  0,  i,  0],
    ///                  [0,  0,  0,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)         // Apply H gate to qubit 0
    ///    .szzdg(0,1); // Then apply SZZ† gate between qubits 0 and 1
    /// ```
    #[inline]
    fn szzdg(&mut self, q1: T, q2: T) -> &mut Self {
        self.z(q1).z(q2).szz(q1, q2)
    }

    /// Applies the SWAP operation between two qubits. The SWAP gate exchanges the quantum
    /// states of two qubits.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → IX
    /// IX → XI
    /// ZI → IZ
    /// IZ → ZI
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// SWAP = [[1, 0, 0, 0],
    ///         [0, 0, 1, 0],
    ///         [0, 1, 0, 0],
    ///         [0, 0, 0, 1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)       // Apply H gate to qubit 0
    ///    .swap(0,1); // Then swap qubits 0 and 1
    /// ```
    #[inline]
    fn swap(&mut self, q1: T, q2: T) -> &mut Self {
        self.cx(q1, q2).cx(q2, q1).cx(q1, q2)
    }

    /// Applies the iSWAP two-qubit Clifford operation. The iSWAP gate swaps states with an
    /// additional i phase on the swapped states.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → -ZY
    /// IX → YZ
    /// ZI → IZ
    /// IZ → ZI
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// iSWAP = [[1, 0,  0,  0],
    ///          [0, 0,  i,  0],
    ///          [0, i,  0,  0],
    ///          [0, 0,  0,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)        // Apply H gate to qubit 0
    ///    .iswap(0,1); // Then apply iSWAP gate between qubits 0 and 1
    /// ```
    fn iswap(&mut self, q1: T, q2: T) -> &mut Self {
        self.sz(q1).sz(q2).h(q1).cx(q1, q2).cx(q2, q1).h(q2)
    }

    /// Applies the G two-qubit Clifford operation. G is a symmetric two-qubit operation that
    /// implements a particular permutation of single-qubit Paulis.
    ///
    /// # Arguments
    /// * `q1` - First qubit index.
    /// * `q2` - Second qubit index.
    ///
    /// # Pauli Transformation
    /// ```text
    /// XI → IX
    /// IX → XI
    /// ZI → XZ
    /// IZ → ZX
    /// ```
    ///
    /// # Matrix Representation
    /// ```text
    /// G = 1/2 [[1,  1,  1, -1],
    ///          [1, -1,  1,  1],
    ///          [1,  1, -1,  1],
    ///          [-1, 1,  1,  1]]
    /// ```
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.h(0)     // Apply H gate to qubit 0
    ///    .g(0,1);  // Then apply G gate between qubits 0 and 1
    /// ```
    #[inline]
    fn g(&mut self, q1: T, q2: T) -> &mut Self {
        self.cz(q1, q2).h(q1).h(q2).cz(q1, q2)
    }

    /// Measures the +X Pauli operator, projecting to the measured eigenstate.
    ///
    /// Projects the state into either the |+⟩ or |-⟩ eigenstate based on the
    /// measurement outcome.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if projected to |-⟩, false if projected to |+⟩
    ///   - `is_deterministic`: true if state was already in an X eigenstate
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// |ψ⟩ → |+⟩ with probability |⟨+|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |-⟩ with probability |⟨-|ψ⟩|²  (outcome = true)
    /// ```
    /// Where |±⟩ = (|0⟩ ± |1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `mpx(q)` to measure and force preparation into |+⟩ state
    /// * Use `px(q)` for direct preparation of |+⟩ state
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mx(0);    // Measure in X basis
    /// // State is now either |+⟩ or |-⟩ depending on outcome
    /// ```
    #[inline]
    fn mx(&mut self, q: T) -> MeasurementResult {
        self.h(q);
        let meas = self.mz(q);
        self.h(q);

        meas
    }

    /// Measures the -X Pauli operator, projecting to the measured eigenstate.
    ///
    /// Projects the state into either the |+⟩ or |-⟩ eigenstate based on the
    /// measurement outcome.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if projected to |+⟩, false if projected to |-⟩
    ///   - `is_deterministic`: true if state was already in an X eigenstate
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// |ψ⟩ → |-⟩ with probability |⟨-|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |+⟩ with probability |⟨+|ψ⟩|²  (outcome = true)
    /// ```
    /// Where |±⟩ = (|0⟩ ± |1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `mpnx(q)` to measure and force preparation into |-⟩ state
    /// * Use `pnx(q)` for direct preparation of |-⟩ state
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mnx(0);    // Measure in -X basis
    /// // State is now either |+⟩ or |-⟩ depending on outcome
    /// ```
    #[inline]
    fn mnx(&mut self, q: T) -> MeasurementResult {
        self.h(q).x(q);
        let meas = self.mz(q);
        self.x(q).h(q);

        meas
    }

    /// Measures the +Y Pauli operator, projecting to the measured eigenstate.
    ///
    /// Projects the state into either the |+i⟩ or |-i⟩ eigenstate based on the
    /// measurement outcome.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if projected to |-i⟩, false if projected to |+i⟩
    ///   - `is_deterministic`: true if state was already in a Y eigenstate
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// |ψ⟩ → |+i⟩ with probability |⟨+i|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |-i⟩ with probability |⟨-i|ψ⟩|²  (outcome = true)
    /// ```
    /// Where |±i⟩ = (|0⟩ ± i|1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `mpy(q)` to measure and force preparation into |+i⟩ state
    /// * Use `py(q)` for direct preparation of |+i⟩ state
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.my(0);    // Measure in Y basis
    /// // State is now either |+i⟩ or |-i⟩ depending on outcome
    /// ```
    #[inline]
    fn my(&mut self, q: T) -> MeasurementResult {
        self.sx(q);
        let meas = self.mz(q);
        self.sxdg(q);

        meas
    }

    /// Measures the -Y Pauli operator, projecting to the measured eigenstate.
    ///
    /// Projects the state into either the |+i⟩ or |-i⟩ eigenstate based on the
    /// measurement outcome.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if projected to |+i⟩, false if projected to |-i⟩
    ///   - `is_deterministic`: true if state was already in a Y eigenstate
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// |ψ⟩ → |-i⟩ with probability |⟨-i|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |+i⟩ with probability |⟨+i|ψ⟩|²  (outcome = true)
    /// ```
    /// Where |±i⟩ = (|0⟩ ± i|1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `mpny(q)` to measure and force preparation into |-i⟩ state
    /// * Use `pny(q)` for direct preparation of |-i⟩ state
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mny(0);    // Measure in -Y basis
    /// // State is now either |+i⟩ or |-i⟩ depending on outcome
    /// ```
    #[inline]
    fn mny(&mut self, q: T) -> MeasurementResult {
        // -Y -> +Z
        self.sxdg(q);
        let meas = self.mz(q);
        // +Z -> -Y
        self.sx(q);

        meas
    }

    /// Measures the +Z Pauli operator, projecting to the measured eigenstate.
    ///
    /// Projects the state into either the |0⟩ or |1⟩ eigenstate based on the
    /// measurement outcome. This is the standard computational basis measurement.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if projected to |1⟩, false if projected to |0⟩
    ///   - `is_deterministic`: true if state was already in a Z eigenstate
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// |ψ⟩ → |0⟩ with probability |⟨0|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |1⟩ with probability |⟨1|ψ⟩|²  (outcome = true)
    /// ```
    ///
    /// # Related Operations
    /// * Use `mpz(q)` to measure and force preparation into |0⟩ state
    /// * Use `pz(q)` for direct preparation of |0⟩ state
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mz(0);    // Measure in Z basis
    /// // State is now either |0⟩ or |1⟩ depending on outcome
    /// ```
    fn mz(&mut self, q: T) -> MeasurementResult;

    /// Measures the -Z Pauli operator, projecting to the measured eigenstate.
    ///
    /// Projects the state into either the |0⟩ or |1⟩ eigenstate based on the
    /// measurement outcome.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if projected to |0⟩, false if projected to |1⟩
    ///   - `is_deterministic`: true if state was already in a Z eigenstate
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// |ψ⟩ → |1⟩ with probability |⟨1|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |0⟩ with probability |⟨0|ψ⟩|²  (outcome = true)
    /// ```
    ///
    /// # Related Operations
    /// * Use `mpnz(q)` to measure and force preparation into |1⟩ state
    /// * Use `pnz(q)` for direct preparation of |1⟩ state
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mnz(0);    // Measure in -Z basis
    /// // State is now either |0⟩ or |1⟩ depending on outcome
    /// ```
    #[inline]
    fn mnz(&mut self, q: T) -> MeasurementResult {
        self.x(q);
        let meas = self.mz(q);
        self.x(q);

        meas
    }

    /// Prepares a qubit in the +1 eigenstate of the +X operator. Equivalent to preparing
    /// |+X⟩ = |+⟩ = (|0⟩ + |1⟩)/√2.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Related Operations
    /// * Use `mx(q)` to measure in the X basis
    /// * Use `mpx(q)` to measure and prepare in the same eigenstate
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.px(0)     // Prepare qubit 0 in |+X⟩ state
    ///    .cx(0,1);  // Then apply CX gate
    /// ```
    #[inline]
    fn px(&mut self, q: T) -> &mut Self {
        self.mpx(q);
        self
    }

    /// Prepares the qubit in the +1 eigenstate of -X. Equivalent to preparing
    /// |-X⟩ = |-⟩ = (|0⟩ - |1⟩)/√2.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Related Operations
    /// * Use `mnx(q)` to measure in the -X basis
    /// * Use `mpnx(q)` to measure and prepare in the same eigenstate
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.pnx(0)    // Prepare qubit 0 in |-X⟩ state
    ///    .cx(0,1);  // Then apply CX gate
    /// ```
    #[inline]
    fn pnx(&mut self, q: T) -> &mut Self {
        self.mpnx(q);
        self
    }

    /// Prepares the qubit in the +1 eigenstate of +Y. Equivalent to preparing
    /// |+Y⟩ = |+i⟩ = (|0⟩ + i|1⟩)/√2.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Related Operations
    /// * Use `my(q)` to measure in the Y basis
    /// * Use `mpy(q)` to measure and prepare in the same eigenstate
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.py(0)     // Prepare qubit 0 in |+Y⟩ state
    ///    .cx(0,1);  // Then apply CX gate
    /// ```
    #[inline]
    fn py(&mut self, q: T) -> &mut Self {
        self.mpy(q);
        self
    }

    /// Prepares the qubit in the +1 eigenstate of -Y. Equivalent to preparing
    /// |-Y⟩ = |-i⟩ = (|0⟩ - i|1⟩)/√2.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Related Operations
    /// * Use `mny(q)` to measure in the -Y basis
    /// * Use `mpny(q)` to measure and prepare in the same eigenstate
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.pny(0)    // Prepare qubit 0 in |-Y⟩ state
    ///    .cx(0,1);  // Then apply CX gate
    /// ```
    #[inline]
    fn pny(&mut self, q: T) -> &mut Self {
        self.mpny(q);
        self
    }

    /// Prepares the qubit in the +1 eigenstate of +Z. Equivalent to preparing |+Z⟩ = |0⟩.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Related Operations
    /// * Use `mz(q)` to measure in the Z basis
    /// * Use `mpz(q)` to measure and prepare in the same eigenstate
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.pz(0)     // Prepare qubit 0 in |0⟩ state
    ///    .cx(0,1);  // Then apply CX gate
    /// ```
    #[inline]
    fn pz(&mut self, q: T) -> &mut Self {
        self.mpz(q);
        self
    }

    /// Prepares the qubit in the +1 eigenstate of -Z. Equivalent to preparing |-Z⟩ = |1⟩.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `&mut Self` - Returns the simulator for method chaining.
    ///
    /// # Related Operations
    /// * Use `mnz(q)` to measure in the -Z basis
    /// * Use `mpnz(q)` to measure and prepare in the same eigenstate
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(2);
    /// sim.pnz(0)    // Prepare qubit 0 in |1⟩ state
    ///    .cx(0,1);  // Then apply CX gate
    /// ```
    #[inline]
    fn pnz(&mut self, q: T) -> &mut Self {
        self.mpnz(q);
        self
    }

    /// Both measures +X and prepares the qubit in the |+⟩ state.
    ///
    /// After measurement, unlike `mx()` which projects to the measured eigenstate,
    /// this operation always prepares the |+⟩ state regardless of measurement outcome.
    /// The operation combines measurement with deterministic state preparation.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if Z correction was needed
    ///   - `is_deterministic`: true if state was already |+⟩
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// First measures X:
    /// |ψ⟩ → |+⟩ with probability |⟨+|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |-⟩ with probability |⟨-|ψ⟩|²  (outcome = true)
    ///
    /// Then applies correction if needed:
    /// |-⟩ → Z|-⟩ = |+⟩
    /// ```
    /// Final state is always |+⟩ = (|0⟩ + |1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `mx(q)` to measure and project to the measured eigenstate
    /// * Use `px(q)` for state preparation without measurement
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mpx(0);    // Measure X and prepare |+⟩ state
    /// // State is now |+⟩ regardless of measurement outcome
    /// ```
    #[inline]
    fn mpx(&mut self, q: T) -> MeasurementResult {
        let result = self.mx(q);
        if result.outcome {
            self.z(q);
        }
        result
    }

    /// Both measures -X and prepares the qubit in the |-⟩ state.
    ///
    /// After measurement, unlike `mnx()` which projects to the measured eigenstate,
    /// this operation always prepares the |-⟩ state regardless of measurement outcome.
    /// The operation combines measurement with deterministic state preparation.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if Z correction was needed
    ///   - `is_deterministic`: true if state was already |-⟩
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// First measures -X:
    /// |ψ⟩ → |-⟩ with probability |⟨-|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |+⟩ with probability |⟨+|ψ⟩|²  (outcome = true)
    ///
    /// Then applies correction if needed:
    /// |+⟩ → Z|+⟩ = |-⟩
    /// ```
    /// Final state is always |-⟩ = (|0⟩ - |1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `mnx(q)` to measure and project to the measured eigenstate
    /// * Use `pnx(q)` for state preparation without measurement
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mpnx(0);    // Measure -X and prepare |-⟩ state
    /// // State is now |-⟩ regardless of measurement outcome
    /// ```
    #[inline]
    fn mpnx(&mut self, q: T) -> MeasurementResult {
        let result = self.mnx(q);
        if result.outcome {
            self.z(q);
        }
        result
    }

    /// Both measures +Y and prepares the qubit in the |+i⟩ state.
    ///
    /// After measurement, unlike `my()` which projects to the measured eigenstate,
    /// this operation always prepares the |+i⟩ state regardless of measurement outcome.
    /// The operation combines measurement with deterministic state preparation.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if Z correction was needed
    ///   - `is_deterministic`: true if state was already |+i⟩
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// First measures Y:
    /// |ψ⟩ → |+i⟩ with probability |⟨+i|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |-i⟩ with probability |⟨-i|ψ⟩|²  (outcome = true)
    ///
    /// Then applies correction if needed:
    /// |-i⟩ → Z|-i⟩ = |+i⟩
    /// ```
    /// Final state is always |+i⟩ = (|0⟩ + i|1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `my(q)` to measure and project to the measured eigenstate
    /// * Use `py(q)` for state preparation without measurement
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mpy(0);    // Measure Y and prepare |+i⟩ state
    /// // State is now |+i⟩ regardless of measurement outcome
    /// ```
    #[inline]
    fn mpy(&mut self, q: T) -> MeasurementResult {
        let result = self.my(q);
        if result.outcome {
            self.z(q);
        }
        result
    }

    /// Both measures -Y and prepares the qubit in the |-i⟩ state.
    ///
    /// After measurement, unlike `mny()` which projects to the measured eigenstate,
    /// this operation always prepares the |-i⟩ state regardless of measurement outcome.
    /// The operation combines measurement with deterministic state preparation.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if Z correction was needed
    ///   - `is_deterministic`: true if state was already |-i⟩
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// First measures -Y:
    /// |ψ⟩ → |-i⟩ with probability |⟨-i|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |+i⟩ with probability |⟨+i|ψ⟩|²  (outcome = true)
    ///
    /// Then applies correction if needed:
    /// |+i⟩ → Z|+i⟩ = |-i⟩
    /// ```
    /// Final state is always |-i⟩ = (|0⟩ - i|1⟩)/√2.
    ///
    /// # Related Operations
    /// * Use `mny(q)` to measure and project to the measured eigenstate
    /// * Use `pny(q)` for state preparation without measurement
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mpny(0);    // Measure -Y and prepare |-i⟩ state
    /// // State is now |-i⟩ regardless of measurement outcome
    /// ```
    #[inline]
    fn mpny(&mut self, q: T) -> MeasurementResult {
        let result = self.mny(q);
        if result.outcome {
            self.z(q);
        }
        result
    }

    /// Both measures +Z and prepares the qubit in the |0⟩ state.
    ///
    /// After measurement, unlike `mz()` which projects to the measured eigenstate,
    /// this operation always prepares the |0⟩ state regardless of measurement outcome.
    /// The operation combines measurement with deterministic state preparation.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if X correction was needed
    ///   - `is_deterministic`: true if state was already |0⟩
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// First measures Z:
    /// |ψ⟩ → |0⟩ with probability |⟨0|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |1⟩ with probability |⟨1|ψ⟩|²  (outcome = true)
    ///
    /// Then applies correction if needed:
    /// |1⟩ → X|1⟩ = |0⟩
    /// ```
    /// Final state is always |0⟩.
    ///
    /// # Related Operations
    /// * Use `mz(q)` to measure and project to the measured eigenstate
    /// * Use `pz(q)` for state preparation without measurement
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mpz(0);    // Measure Z and prepare |0⟩ state
    /// // State is now |0⟩ regardless of measurement outcome
    /// ```
    #[inline]
    fn mpz(&mut self, q: T) -> MeasurementResult {
        let result = self.mz(q);
        if result.outcome {
            self.x(q);
        }
        result
    }

    /// Both measures -Z and prepares the qubit in the |1⟩ state.
    ///
    /// After measurement, unlike `mnz()` which projects to the measured eigenstate,
    /// this operation always prepares the |1⟩ state regardless of measurement outcome.
    /// The operation combines measurement with deterministic state preparation.
    ///
    /// # Arguments
    /// * `q` - Target qubit index.
    ///
    /// # Returns
    /// * `MeasurementResult` containing:
    ///   - `outcome`: true if X correction was needed
    ///   - `is_deterministic`: true if state was already |1⟩
    ///
    /// # Mathematical Details
    /// The operation performs:
    /// ```text
    /// First measures -Z:
    /// |ψ⟩ → |1⟩ with probability |⟨1|ψ⟩|²  (outcome = false)
    /// |ψ⟩ → |0⟩ with probability |⟨0|ψ⟩|²  (outcome = true)
    ///
    /// Then applies correction if needed:
    /// |0⟩ → X|0⟩ = |1⟩
    /// ```
    /// Final state is always |1⟩.
    ///
    /// # Related Operations
    /// * Use `mnz(q)` to measure and project to the measured eigenstate
    /// * Use `pnz(q)` for state preparation without measurement
    ///
    /// # Examples
    /// ```rust
    /// use pecos_qsim::{CliffordGateable, StdSparseStab};
    /// let mut sim = StdSparseStab::new(1);
    /// let result = sim.mpnz(0);    // Measure -Z and prepare |1⟩ state
    /// // State is now |1⟩ regardless of measurement outcome
    /// ```
    #[inline]
    fn mpnz(&mut self, q: T) -> MeasurementResult {
        let result = self.mnz(q);
        if result.outcome {
            self.x(q);
        }
        result
    }
}
