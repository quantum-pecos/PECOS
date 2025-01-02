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

use crate::CliffordGateable;
use pecos_core::IndexableElement;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

/// A trait for implementing arbitrary rotation gates on a quantum system.
///
/// This trait extends [`CliffordGateable`] and provides methods for applying
/// single-qubit and two-qubit rotation gates around various axes.
///
/// # Type Parameters
/// - `T`: A type implementing [`IndexableElement`], representing the indices
///   of qubits within the quantum system.
///
/// # Note
/// Most of the methods in this trait have default implementations. However, the
/// following methods are the minimum methods that must be implemented to utilize the trait:
/// - `rx`: Rotation around the X-axis.
/// - `rz`: Rotation around the Z-axis.
/// - `rzz`: Two-qubit rotation around the ZZ-axis.
pub trait ArbitraryRotationGateable<T: IndexableElement>: CliffordGateable<T> {
    /// Applies a rotation around the X-axis by an angle `theta`.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle in radians.
    /// - `q`: The target qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    fn rx(&mut self, theta: f64, q: T) -> &mut Self;

    /// Applies a rotation around the Y-axis by an angle `theta`.
    ///
    /// By default, this is implemented in terms of `sz`, `rx`, and `szdg` gates.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle in radians.
    /// - `q`: The target qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    #[inline]
    fn ry(&mut self, theta: f64, q: T) -> &mut Self {
        self.sz(q).rx(theta, q).szdg(q)
    }

    /// Applies a rotation around the Z-axis by an angle `theta`.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle in radians.
    /// - `q`: The target qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    fn rz(&mut self, theta: f64, q: T) -> &mut Self;

    /// Applies a general single-qubit unitary gate with the specified parameters.
    ///
    /// By default, this is implemented in terms of `rz` and `ry` gates.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle around the Y-axis in radians.
    /// - `phi`: The first Z-axis rotation angle in radians.
    /// - `lambda`: The second Z-axis rotation angle in radians.
    /// - `q`: The target qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    #[inline]
    fn u(&mut self, theta: f64, phi: f64, lambda: f64, q: T) -> &mut Self {
        self.rz(lambda, q).ry(theta, q).rz(phi, q)
    }

    /// Applies an XY-plane rotation gate with a specified angle and axis.
    ///
    /// By default, this is implemented in terms of `rz` and `ry` gates.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle in radians.
    /// - `phi`: The axis angle in radians.
    /// - `q`: The target qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    #[inline]
    fn r1xy(&mut self, theta: f64, phi: f64, q: T) -> &mut Self {
        self.rz(-phi + FRAC_PI_2, q)
            .ry(theta, q)
            .rz(phi - FRAC_PI_2, q)
    }

    /// Applies the T gate (π/8 rotation around Z-axis).
    ///
    /// # Parameters
    /// - `q`: The target qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    #[inline]
    fn t(&mut self, q: T) -> &mut Self {
        self.rz(FRAC_PI_4, q)
    }

    /// Applies the T† (T-dagger) gate (−π/8 rotation around Z-axis).
    ///
    /// # Parameters
    /// - `q`: The target qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    #[inline]
    fn tdg(&mut self, q: T) -> &mut Self {
        self.rz(-FRAC_PI_4, q)
    }

    /// Applies a two-qubit XX rotation gate.
    ///
    /// By default, this is implemented in terms of Hadamard (`h`) and ZZ rotation (`rzz`) gates.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle in radians.
    /// - `q1`: The first qubit index.
    /// - `q2`: The second qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    #[inline]
    fn rxx(&mut self, theta: f64, q1: T, q2: T) -> &mut Self {
        self.h(q1).h(q2).rzz(theta, q1, q2).h(q1).h(q2)
    }

    /// Applies a two-qubit YY rotation gate.
    ///
    /// By default, this is implemented in terms of SX and ZZ rotation (`rzz`) gates.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle in radians.
    /// - `q1`: The first qubit index.
    /// - `q2`: The second qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    #[inline]
    fn ryy(&mut self, theta: f64, q1: T, q2: T) -> &mut Self {
        self.sx(q1).sx(q2).rzz(theta, q1, q2).sxdg(q1).sxdg(q2)
    }

    /// Applies a two-qubit ZZ rotation gate.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle in radians.
    /// - `q1`: The first qubit index.
    /// - `q2`: The second qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    fn rzz(&mut self, theta: f64, q1: T, q2: T) -> &mut Self;

    /// Applies a composite rotation gate using RXX, RYY, and RZZ gates.
    ///
    /// # Parameters
    /// - `theta`: The rotation angle for the RXX gate in radians.
    /// - `phi`: The rotation angle for the RYY gate in radians.
    /// - `lambda`: The rotation angle for the RZZ gate in radians.
    /// - `q1`: The first qubit index.
    /// - `q2`: The second qubit index.
    ///
    /// # Returns
    /// A mutable reference to `Self` for method chaining.
    ///
    /// # Note
    /// The current implementation might have a reversed order of operations.
    #[inline]
    fn rxxryyrzz(&mut self, theta: f64, phi: f64, lambda: f64, q1: T, q2: T) -> &mut Self {
        // TODO: This is likely backwards..
        self.rxx(theta, q1, q2).ryy(phi, q1, q2).rzz(lambda, q1, q2)
    }
}
