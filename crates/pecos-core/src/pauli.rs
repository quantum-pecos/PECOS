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

use crate::{Set, VecSet};
use core::ops::{BitAnd, BitOr, BitXor};
use std::fmt::Debug;

/// Represents the phase of a Pauli operator.
///
/// The phase can take on one of four values:
/// - `PLUS`: Represents +1 phase.
/// - `MINUS`: Represents -1 phase.
/// - `PLUS_I`: Represents +i phase.
/// - `MINUS_I`: Represents -i phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase(u8);

impl Default for Phase {
    fn default() -> Self {
        Self::PLUS
    }
}

impl Phase {
    pub const PLUS: Phase = Phase(0b00);
    pub const MINUS: Phase = Phase(0b01);
    pub const PLUS_I: Phase = Phase(0b10);
    pub const MINUS_I: Phase = Phase(0b11);

    /// Multiplies two `Phase` values and returns the result.
    ///
    /// This operation uses a precomputed lookup table for performance.
    ///
    /// # Parameters
    /// - `other`: The other `Phase` to multiply with.
    ///
    /// # Returns
    /// A new `Phase` that represents the product of the two phases.
    #[inline]
    #[must_use]
    pub fn multiply(&self, other: &Phase) -> Phase {
        // Fast lookup table for phase multiplication
        const MULT_TABLE: [[u8; 4]; 4] = [
            [0b00, 0b01, 0b10, 0b11],
            [0b01, 0b00, 0b11, 0b10],
            [0b10, 0b11, 0b00, 0b01],
            [0b11, 0b10, 0b01, 0b00],
        ];
        Phase(MULT_TABLE[self.0 as usize][other.0 as usize])
    }
}

/// A trait for general Pauli operators.
///
/// This trait defines the behavior and properties of Pauli operators, including their
/// ability to be multiplied, determining their weight, and checking commutation relations.
#[allow(clippy::module_name_repetitions)]
pub trait PauliOperator: Clone + Debug {
    /// The type of the individual elements representing positions in the operator.
    type Item: Copy;

    /// Multiplies two Pauli operators and returns the resulting operator.
    ///
    /// # Parameters
    /// - `other`: The other Pauli operator to multiply with.
    ///
    /// # Returns
    /// A new Pauli operator representing the product of the two.
    #[must_use]
    fn multiply(&self, other: &Self) -> Self;

    /// Calculates the weight of the Pauli operator.
    ///
    /// The weight is the number of positions where the operator acts non-trivially
    /// (i.e., acts as X, Y, or Z instead of the identity).
    ///
    /// # Returns
    /// The weight of the operator as a `usize`.
    fn weight(&self) -> usize;

    /// Determines whether this Pauli operator commutes with another.
    ///
    /// # Parameters
    /// - `other`: The other Pauli operator to check commutation with.
    ///
    /// # Returns
    /// `true` if the operators commute, `false` if they anti-commute.
    fn commutes_with(&self, other: &Self) -> bool;
}

#[allow(clippy::module_name_repetitions)]
pub type StdPauli = SetPauli<VecSet<usize>>;

// Represents a Pauli operator with positions for X and Z components.
///
/// The `SetPauli` struct uses generic sets (`x_positions` and `z_positions`) to track qubit
/// positions affected by the X and Z components of the operator.
///
/// - Positions in `x_positions` are affected by the X operator.
/// - Positions in `z_positions` are affected by the Z operator.
/// - Positions in both are affected by the Y operator.
#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SetPauli<T: for<'a> Set<'a>> {
    phase: Phase,
    x_positions: T,
    z_positions: T,
}

impl<E, T> Default for SetPauli<T>
where
    T: for<'a> Set<'a, Element = E> + Default,
{
    fn default() -> Self {
        Self {
            phase: Phase::PLUS,
            x_positions: T::default(),
            z_positions: T::default(),
        }
    }
}

impl<E, T> SetPauli<T>
where
    T: for<'a> Set<'a, Element = E> + FromIterator<E> + Clone + Default,
    for<'a> &'a T: BitAnd<Output = T> + BitOr<Output = T> + BitXor<Output = T>,
    E: Ord + Copy,
{
    /// Initializes a new empty Pauli operator, which is equivalent to the identity.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `SetPauli` instance with the specified phase and qubit positions for X, Y, and Z operators.
    ///
    /// This method constructs a Pauli operator using the provided qubit positions:
    /// - `x`: Positions affected by the X operator.
    /// - `y`: Positions affected by both X and Z operators (added to both `x_positions` and `z_positions`).
    /// - `z`: Positions affected by the Z operator.
    ///
    /// The `phase` specifies the initial phase of the operator.
    ///
    /// # Parameters
    /// - `phase`: The phase of the Pauli operator (`+1`, `-1`, `+i`, `-i`).
    /// - `x`: A slice of positions affected by the X operator.
    /// - `y`: A slice of positions affected by both X and Z operators.
    /// - `z`: A slice of positions affected by the Z operator.
    ///
    /// # Returns
    /// A `Result` containing a new `SetPauli` instance if the input is valid,
    /// or an error message as a `String` if the input is invalid.
    ///
    /// # Errors
    /// This method returns an `Err` if:
    /// - Any qubit positions in `x` and `z` overlap. Such overlaps are not allowed
    ///   since a single qubit cannot simultaneously be affected by both X and Z components
    ///   in the same Pauli operator.
    ///
    /// # Examples
    /// ```
    /// use pecos_core::{SetPauli, Phase, VecSet};
    ///
    /// let phase = Phase::PLUS;
    /// let x = [1, 2];
    /// let y = [3];
    /// let z = [4];
    ///
    /// let pauli: SetPauli<VecSet<usize>> = SetPauli::with_operators(phase, &x, &y, &z).unwrap();
    /// ```
    ///
    /// # Panics
    /// This function does not panic under normal usage.
    pub fn with_operators(phase: Phase, x: &[E], y: &[E], z: &[E]) -> Result<Self, String> {
        let mut x_set: T = x.iter().copied().collect();
        let mut z_set: T = z.iter().copied().collect();

        if x_set.intersection(&z_set).next().is_some() {
            return Err("x and z share common elements".to_string());
        }

        for &elem in y {
            x_set = (&x_set | &T::from_iter([elem])).clone();
            z_set = (&z_set | &T::from_iter([elem])).clone();
        }

        Ok(Self {
            phase,
            x_positions: x_set,
            z_positions: z_set,
        })
    }
}

// TODO: Consider making a clear distinction between mutation in place and not

impl<E, T> PauliOperator for SetPauli<T>
where
    T: for<'a> Set<'a, Element = E> + FromIterator<E> + Clone + Default,
    for<'a> &'a T: BitAnd<Output = T> + BitOr<Output = T> + BitXor<Output = T>,
    E: Ord + Copy,
{
    type Item = E;

    /// Multiplies two `SetPauli` operators and returns the result.
    ///
    /// # Parameters
    /// - `other`: The other `SetPauli` operator to multiply with.
    ///
    /// # Returns
    /// A new `SetPauli` operator representing the product.
    #[inline]
    #[must_use]
    fn multiply(&self, other: &Self) -> Self {
        let mut phase = self.phase.multiply(&other.phase);

        let x_and_z = &self.x_positions & &other.z_positions;
        if !x_and_z.is_empty() {
            phase = phase.multiply(&Phase::MINUS);
        }

        Self {
            phase,
            x_positions: &self.x_positions ^ &other.x_positions,
            z_positions: &self.z_positions ^ &other.z_positions,
        }
    }

    /// Calculates the weight of the `SetPauli` operator.
    ///
    /// The weight is the total number of unique positions affected by the X and Z components.
    ///
    /// # Returns
    /// The weight as a `usize`.
    #[inline]
    fn weight(&self) -> usize {
        self.x_positions.len() + self.z_positions.len()
    }

    /// Checks if this `SetPauli` operator commutes with another.
    ///
    /// # Parameters
    /// - `other`: The other `SetPauli` operator to check commutation with.
    ///
    /// # Returns
    /// `true` if the operators commute, `false` if they anti-commute.
    #[inline]
    fn commutes_with(&self, other: &Self) -> bool {
        // Check if the anti-commutation count is even (commutes) or odd (anti-commutes)
        let x_and_z = &self.x_positions & &other.z_positions;
        let z_and_x = &self.z_positions & &other.x_positions;

        (x_and_z.len() + z_and_x.len()) % 2 == 0
    }
}

/// Represents a compact Pauli operator using bitmaps for up to 64 qubits.
///
/// The `BitSetPauli` struct uses `x_bits` and `z_bits` to track which qubits are affected
/// by X and Z components of the operator. Each bit corresponds to a qubit, where a set bit
/// indicates the qubit is affected by the respective component.
///
/// - `x_bits`: A 64-bit bitmap indicating qubits affected by X.
/// - `z_bits`: A 64-bit bitmap indicating qubits affected by Z.
/// - `phase`: Represents the overall phase of the operator (`+1`, `-1`, `+i`, or `-i`).
///
/// # Performance
/// This representation is optimized for fixed-size systems (up to 64 qubits), allowing
/// fast bitwise operations to compute multiplication, weight, and commutation properties.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct BitSetPauli {
    phase: Phase,
    x_bits: u64,
    z_bits: u64,
}

impl BitSetPauli {
    /// Initializes a new empty Pauli operator, which is equivalent to the identity.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `BitSetPauli` instance with the specified phase and qubit positions for X, Y, and Z operators.
    ///
    /// This method constructs a Pauli operator using the provided qubit positions:
    /// - `x`: Positions affected by the X operator.
    /// - `y`: Positions affected by both X and Z operators (added to both `x_positions` and `z_positions`).
    /// - `z`: Positions affected by the Z operator.
    ///
    /// The `phase` specifies the initial phase of the operator.
    ///
    /// # Parameters
    /// - `phase`: The phase of the Pauli operator (`+1`, `-1`, `+i`, `-i`).
    /// - `x`: A slice of positions affected by the X operator.
    /// - `y`: A slice of positions affected by both X and Z operators.
    /// - `z`: A slice of positions affected by the Z operator.
    ///
    /// # Returns
    /// A `Result` containing a new `SetPauli` instance if the input is valid,
    /// or an error message as a `String` if the input is invalid.
    ///
    /// # Errors
    /// This method returns an `Err` if:
    /// - Any qubit positions in `x`, `y`, or `z` overlap. Such overlaps are not allowed
    ///   since it is assumed the user is inputting unique Pauli operators.
    ///
    /// # Examples
    /// ```
    /// use pecos_core::{BitSetPauli, Phase};
    ///
    /// let phase = Phase::PLUS;
    /// let x = [1, 2];
    /// let y = [3];
    /// let z = [4];
    ///
    /// let pauli = BitSetPauli::with_operators(phase, &x, &y, &z).unwrap();
    /// ```
    ///
    /// # Panics
    /// This function does not panic under normal usage.
    pub fn with_operators(phase: Phase, x: &[u8], y: &[u8], z: &[u8]) -> Result<Self, String> {
        for &pos in x.iter().chain(y).chain(z) {
            if pos >= 64 {
                return Err("position exceeds 64 qubits".to_string());
            }
        }

        let mut x_bits = x.iter().fold(0, |bits, &pos| bits | (1 << pos));
        let mut z_bits = z.iter().fold(0, |bits, &pos| bits | (1 << pos));

        if x_bits & z_bits != 0 {
            return Err("x and z share common elements".to_string());
        }

        let y_bits = y.iter().fold(0, |bits, &pos| bits | (1 << pos));
        x_bits |= y_bits;
        z_bits |= y_bits;

        Ok(Self {
            phase,
            x_bits,
            z_bits,
        })
    }
}

impl Default for BitSetPauli {
    fn default() -> Self {
        Self {
            phase: Phase::PLUS,
            x_bits: 0,
            z_bits: 0,
        }
    }
}

impl PauliOperator for BitSetPauli {
    type Item = u8;

    #[must_use]
    #[inline]
    fn multiply(&self, other: &Self) -> Self {
        let mut phase = self.phase.multiply(&other.phase);
        // Check anti-commutation from both X-Z and Z-X overlaps at single positions
        let commute_bits = (self.x_bits & other.z_bits) ^ (self.z_bits & other.x_bits);
        if commute_bits.count_ones() % 2 == 1 {
            phase = phase.multiply(&Phase::MINUS);
        }

        Self {
            phase,
            x_bits: self.x_bits ^ other.x_bits,
            z_bits: self.z_bits ^ other.z_bits,
        }
    }

    #[inline]
    fn weight(&self) -> usize {
        (self.x_bits | self.z_bits).count_ones() as usize
    }

    #[inline]
    fn commutes_with(&self, other: &Self) -> bool {
        let overlap_count =
            ((self.x_bits & other.z_bits) ^ (self.z_bits & other.x_bits)).count_ones();
        overlap_count % 2 == 0
    }
}

#[allow(dead_code)]
#[allow(clippy::module_name_repetitions)]
pub struct PauliCollection<T: PauliOperator> {
    paulis: Vec<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VecSet;

    fn assert_sets_equal<E: Ord + Debug + Clone, T: for<'a> Set<'a, Element = E>>(
        left: &T,
        right: &T,
    ) {
        let mut left_elements: Vec<E> = left.iter().cloned().collect();
        let mut right_elements: Vec<E> = right.iter().cloned().collect();
        left_elements.sort();
        right_elements.sort();
        assert_eq!(left_elements, right_elements);
    }

    #[test]
    fn test_valid_pauli_creation() {
        let pauli =
            SetPauli::with_operators(Phase::PLUS, &[1usize, 2], &[3usize], &[4usize]).unwrap();

        assert_eq!(pauli.phase, Phase::PLUS);
        assert_sets_equal(&pauli.x_positions, &VecSet::from_iter([1usize, 2, 3]));
        assert_sets_equal(&pauli.z_positions, &VecSet::from_iter([3usize, 4]));
    }

    #[test]
    fn test_overlap_in_x_and_z() {
        // Simply use Vec to avoid array size issues
        let result = StdPauli::with_operators(
            Phase::MINUS,
            &[1usize, 2],
            &[3usize],
            &[2usize, 4], // Overlaps with x
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "x and z share common elements");
    }

    #[test]
    fn test_y_addition_to_x_and_z() {
        let pauli = SetPauli::with_operators(Phase::PLUS, &[1usize], &[2usize], &[3usize]).unwrap();
        assert_sets_equal(&pauli.x_positions, &VecSet::from_iter([1usize, 2]));
        assert_sets_equal(&pauli.z_positions, &VecSet::from_iter([2usize, 3]));
    }

    #[test]
    fn test_empty_inputs() {
        // Test default/empty constructor
        let pauli = StdPauli::new();
        assert_eq!(pauli.phase, Phase::PLUS);
        assert!(pauli.x_positions.is_empty());
        assert!(pauli.z_positions.is_empty());
    }

    #[test]
    fn test_partial_inputs() {
        let pauli = StdPauli::with_operators(Phase::PLUS_I, &[1usize, 2], &[], &[]).unwrap();
        assert_eq!(pauli.phase, Phase::PLUS_I);
        assert_eq!(pauli.x_positions, VecSet::from_iter([1usize, 2]));
        assert!(pauli.z_positions.is_empty());
    }

    // BitSetPauli tests
    #[test]
    fn test_valid_bitset_pauli_creation() {
        let pauli = BitSetPauli::with_operators(Phase::PLUS_I, &[1u8, 2], &[3u8], &[4u8]).unwrap();
        assert_eq!(pauli.phase, Phase::PLUS_I);
        assert_eq!(pauli.x_bits, 0b1110); // Bits 1,2,3 set
        assert_eq!(pauli.z_bits, 0b11000); // Bits 3,4 set
    }

    #[test]
    fn test_bitset_commuting() {
        let p1 = BitSetPauli::with_operators(Phase::PLUS, &[0u8, 1], &[], &[2]).unwrap();
        let p2 = BitSetPauli::with_operators(Phase::PLUS, &[1u8], &[], &[3]).unwrap();
        assert!(p1.commutes_with(&p2));
    }

    #[test]
    fn test_bitset_anticommuting() {
        let p1 = BitSetPauli::with_operators(Phase::PLUS, &[0u8, 1], &[], &[2]).unwrap();
        let p2 = BitSetPauli::with_operators(Phase::PLUS, &[1u8], &[], &[0]).unwrap();
        assert!(!p1.commutes_with(&p2));
    }

    #[test]
    fn test_bitset_overlap_detection() {
        let result = BitSetPauli::with_operators(Phase::PLUS, &[1u8, 2], &[], &[2u8, 4]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "x and z share common elements");
    }

    #[test]
    fn test_bitset_range_check() {
        let result = BitSetPauli::with_operators(
            Phase::PLUS,
            &[65u8], // Exceeds 64 qubits
            &[],
            &[2u8, 4],
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "position exceeds 64 qubits");
    }

    #[test]
    fn test_bitset_multiplication() {
        let p1 = BitSetPauli::with_operators(Phase::PLUS, &[0u8, 1], &[], &[2]).unwrap();
        let p2 = BitSetPauli::with_operators(Phase::PLUS, &[1u8], &[], &[0]).unwrap();
        let result = p1.multiply(&p2);
        assert_eq!(result.phase, Phase::MINUS);
        assert_eq!(result.x_bits, 0b1);
        assert_eq!(result.z_bits, 0b101); // Both bits 0 and 2
    }

    #[test]
    fn test_bitset_weight() {
        let pauli = BitSetPauli::with_operators(Phase::PLUS, &[1u8, 2], &[3], &[4]).unwrap();
        assert_eq!(pauli.weight(), 4); // Positions 1,2,3,4 (3 appears in both but counted once)
    }

    #[test]
    fn test_empty_bitset_pauli() {
        let pauli = BitSetPauli::new();
        assert_eq!(pauli.phase, Phase::PLUS);
        assert_eq!(pauli.x_bits, 0);
        assert_eq!(pauli.z_bits, 0);
        assert_eq!(pauli.weight(), 0);
    }

    #[test]
    fn test_setpauli_commutes() {
        let p1 = BitSetPauli::with_operators(Phase::PLUS, &[0u8, 1], &[], &[2]).unwrap();
        let p2 = BitSetPauli::with_operators(Phase::PLUS, &[1u8], &[], &[3]).unwrap();
        assert!(p1.commutes_with(&p2));
    }

    #[test]
    fn test_setpauli_anticommutes() {
        let p1 = StdPauli::with_operators(Phase::PLUS, &[0, 1], &[], &[2]).unwrap();
        let p2 = StdPauli::with_operators(Phase::PLUS, &[1], &[], &[0]).unwrap();
        assert!(!p1.commutes_with(&p2));
    }

    #[test]
    fn test_bitset_commutes() {
        let p1 = BitSetPauli::with_operators(Phase::PLUS, &[0, 1], &[], &[2]).unwrap();
        let p2 = BitSetPauli::with_operators(Phase::PLUS, &[1], &[], &[3]).unwrap();
        assert!(p1.commutes_with(&p2));
    }

    #[test]
    fn test_bitset_anticommutes() {
        let p1 = BitSetPauli::with_operators(Phase::PLUS, &[0, 1], &[], &[2]).unwrap();
        let p2 = BitSetPauli::with_operators(Phase::PLUS, &[1], &[], &[0]).unwrap();
        assert!(!p1.commutes_with(&p2));
    }
}
