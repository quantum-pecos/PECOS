//! # Angle: Fixed-Point Representation for Rotations
//!
//! The `Angle` struct provides a fixed-point fractional representation for angles in the range `[0, 2^n)`,
//! where `n` is the bit width of the underlying unsigned integer (`u32`, `u64`, or `u128`).
//! It is optimized for fast, modular arithmetic operations, suitable for applications like simulations,
//! graphics, and scientific computing.
//!
//! ## Key Features
//! - Fixed-point representation with efficient modular arithmetic.
//! - Common angle constants: `ZERO`, `QUARTER_TURN`, `HALF_TURN`, `THREE_QUARTERS_TURN`, `FULL_TURN`.
//! - Arithmetic operations: addition, subtraction, multiplication, division.
//! - Conversion to radians.
//!
//! ## Example Usage
//! ```rust
//! use pecos_core::Angle64;
//! let quarter_turn = Angle64::QUARTER_TURN;
//! let half_turn = quarter_turn + quarter_turn;
//! assert_eq!(half_turn.fraction(), Angle64::HALF_TURN.fraction());
//!
//! let radians = half_turn.to_radians();
//! assert!((radians - std::f64::consts::PI).abs() < 1e-6);
//! ```
mod parse;

use num_traits::{
    Bounded, FromPrimitive, ToPrimitive, Unsigned, WrappingAdd, WrappingMul, WrappingNeg,
    WrappingSub, Zero,
};
use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};

pub use parse::ParseAngleError;

/// Alias for `Angle` with a 32-bit unsigned integer.
#[allow(clippy::module_name_repetitions)]
pub type Angle32 = Angle<u32>;

/// Alias for `Angle` with a 64-bit unsigned integer.
#[allow(clippy::module_name_repetitions)]
pub type Angle64 = Angle<u64>;

/// Alias for `Angle` with a 128-bit unsigned integer.
#[allow(clippy::module_name_repetitions)]
pub type Angle128 = Angle<u128>;

/// A fixed-point representation for angles, stored as a fraction of a full turn (2π radians).
///
/// - The fractional range is `[0, 2^n)` for an `n`-bit unsigned integer.
/// - Modular arithmetic ensures that angles wrap around naturally at a full turn.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Angle<T: Unsigned + Copy> {
    fraction: T, // Fixed-point fractional representation in [0, 2^n) turns
}

impl<T> Angle<T>
where
    T: Unsigned
        + Copy
        + ToPrimitive
        + FromPrimitive
        + Zero
        + Bounded
        + WrappingAdd
        + WrappingSub
        + WrappingNeg
        + Rem<Output = T>,
{
    /// Creates a new angle from a fraction of a full turn.
    /// The fraction is interpreted as a fixed-point number where the full range
    /// of T represents one full turn.
    #[inline]
    pub const fn new(fraction: T) -> Self {
        Self { fraction }
    }

    /// Returns the bit (fixed-point) representation of the angle
    pub fn fraction(&self) -> T {
        self.fraction
    }

    /// Converts the angle to radians.
    ///
    /// # Panics
    /// This function will panic if the conversion of `fraction` or `max_value` to `f64` fails.
    pub fn to_radians(&self) -> f64 {
        let max_value = T::max_value()
            .to_f64()
            .expect("Failed to convert max_value to f64");
        self.fraction
            .to_f64()
            .expect("Failed to convert fraction to f64")
            / max_value
            * std::f64::consts::TAU
    }

    /// Creates an angle from a value in radians.
    ///
    /// # Panics
    /// This function will panic if the conversion from f64 to the target type fails.
    #[inline]
    #[must_use]
    pub fn from_radians(radians: f64) -> Self {
        // First normalize the input to [0, 2π)
        let normalized_radians = radians.rem_euclid(std::f64::consts::TAU);

        let fraction = (normalized_radians / std::f64::consts::TAU
            * T::max_value()
                .to_f64()
                .expect("Failed to convert max_value to f64"))
        .round();
        Self {
            fraction: T::from_f64(fraction).expect("Failed to convert fraction to target type"),
        }
    }

    /// Returns the sine of the angle.
    #[inline]
    pub fn sin(&self) -> f64 {
        self.to_radians().sin()
    }

    /// Returns the cosine of the angle.
    #[inline]
    pub fn cos(&self) -> f64 {
        self.to_radians().cos()
    }

    /// Returns the tangent of the angle.
    #[inline]
    pub fn tan(&self) -> f64 {
        self.to_radians().tan()
    }

    /// Returns true if this angle is exactly 0.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.fraction == T::zero()
    }

    /// Normalizes the angle to be within [0, 2π).
    /// This is a no-op for this implementation since the fixed-point representation is always normalized.
    #[inline]
    #[must_use]
    pub fn normalize(&self) -> Self {
        *self
    }
}

impl<T> Angle<T>
where
    T: TryFrom<u128> + Default + Unsigned + Copy,
{
    /// Creates an `Angle` from a rational fraction of a turn.
    ///
    /// This method calculates the angle as `numerator / denominator` of a turn,
    /// where a full turn corresponds to the maximum fixed-point value.
    ///
    /// # Panics
    ///
    /// - If `denominator == 0`.
    /// - If the calculation overflows.
    pub fn from_fraction(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            panic!("Denominator cannot be zero");
        }
        if numerator.abs() > i64::MAX / denominator.abs() {
            panic!("Numerator multiplication overflow");
        }
        let fraction = ((numerator as i128) << 63) / (denominator as i128);
        Self {
            fraction: T::try_from(fraction as u128).unwrap_or_else(|_| {
                panic!("Failed to convert fraction to target type")
            }),
        }
    }
}

// convert from Angle<u32> to Angle<u64>
// This requires up-scaling to the larger range
// [0, 2^32) to [0, 2^64)
// This should be lossless and preserve the same notion of angles
impl From<Angle<u32>> for Angle<u64> {
    fn from(angle: Angle<u32>) -> Self {
        let scaled = u64::from(angle.fraction) << 32;
        Self { fraction: scaled }
    }
}

// convert from Angle<u64> to Angle<u32>
// This requires down-scaling to the smaller range
// [0, 2^64) to [0, 2^32)
// This is lossy, but maintains the same notion of angles.
impl From<Angle<u64>> for Angle<u32> {
    fn from(angle: Angle<u64>) -> Self {
        let scaled = (angle.fraction >> 32) as u32;
        Self { fraction: scaled }
    }
}

impl From<Angle<u64>> for Angle<u128> {
    fn from(angle: Angle<u64>) -> Self {
        let scaled = u128::from(angle.fraction) << 64;
        Self { fraction: scaled }
    }
}

impl From<Angle<u128>> for Angle<u64> {
    fn from(angle: Angle<u128>) -> Self {
        let scaled = (angle.fraction >> 64) as u64;
        Self { fraction: scaled }
    }
}

macro_rules! impl_angle_constants {
    ($t:ty) => {
        impl Angle<$t> {
            pub const ZERO: Self = Self { fraction: 0 };
            pub const QUARTER_TURN: Self = Self {
                fraction: 1 << (<$t>::BITS - 2),
            };
            pub const HALF_TURN: Self = Self {
                fraction: 1 << (<$t>::BITS - 1),
            };
            pub const THREE_QUARTERS_TURN: Self = Self {
                fraction: 3 << (<$t>::BITS - 2),
            };
            pub const FULL_TURN: Self = Self { fraction: 0 }; // Wraps to 0
        }
    };
}

impl_angle_constants!(u8);
impl_angle_constants!(u16);
impl_angle_constants!(u32);
impl_angle_constants!(u64);
impl_angle_constants!(u128);

/// Implements addition for angles, with modular wrapping.
impl<T> Add for Angle<T>
where
    T: Unsigned + WrappingAdd + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let sum = self.fraction.wrapping_add(&rhs.fraction);
        Self { fraction: sum }
    }
}

/// Implements subtraction for angles, with modular wrapping.
impl<T: Unsigned + WrappingAdd + WrappingSub + Copy> Sub for Angle<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            fraction: self.fraction.wrapping_sub(&other.fraction),
        }
    }
}

/// Implements scalar multiplication for angles.
impl<T: Unsigned + Copy + WrappingMul> Mul<T> for Angle<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            fraction: self.fraction.wrapping_mul(&scalar),
        }
    }
}

/// Implements scalar division for angles.
impl<T: Unsigned + Copy + FromPrimitive> Div<T> for Angle<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self {
        Self {
            fraction: self.fraction / scalar,
        }
    }
}

/// Implements `Display` for angles in terms of turns.
impl<T: Unsigned + ToPrimitive + Bounded + Copy> fmt::Display for Angle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fraction = self
            .fraction
            .to_f64()
            .expect("Failed to convert fraction to f64")
            / T::max_value()
                .to_f64()
                .expect("Failed to convert max_value to f64");
        write!(f, "{fraction:.6} turns")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};
    use rand::Rng;

    // Basic Construction and Properties
    #[test]
    fn test_constructors() {
        let angle = Angle64::new(0);
        assert_eq!(angle, Angle64::ZERO);

        let normalized_angle = Angle64::from_radians(7.0 * PI).normalize();
        assert!((normalized_angle.to_radians() - PI).abs() < 1e-10);
    }

    #[test]
    fn test_zero_angle() {
        let zero = Angle64::ZERO;
        assert!(zero.is_zero());
        assert!((zero.to_radians()).abs() < 1e-10);
        assert!((zero.sin()).abs() < 1e-10);
        assert!((zero.cos() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_constant_relationships() {
        // Test relationships between constants
        assert_eq!(
            Angle64::HALF_TURN.fraction,
            Angle64::QUARTER_TURN.fraction * 2
        );
        assert_eq!(
            Angle64::THREE_QUARTERS_TURN.fraction,
            Angle64::QUARTER_TURN.fraction * 3
        );
        assert_eq!(Angle64::FULL_TURN.fraction, 0);

        // Test that constants maintain expected relationships in radians
        assert!((Angle64::QUARTER_TURN.to_radians() - FRAC_PI_2).abs() < 1e-10);
        assert!((Angle64::THREE_QUARTERS_TURN.to_radians() - (3.0 * FRAC_PI_2)).abs() < 1e-10);
    }

    // Basic Arithmetic Operations
    #[test]
    fn test_addition() {
        let a = Angle64::QUARTER_TURN;
        let b = Angle64::QUARTER_TURN;
        let result = a + b;
        assert_eq!(result.fraction, Angle64::HALF_TURN.fraction);
    }

    #[test]
    fn test_angle_arithmetic() {
        let quarter = Angle64::QUARTER_TURN;
        let half = Angle64::HALF_TURN;
        let three_quarters = Angle64::THREE_QUARTERS_TURN;
        let full = Angle64::FULL_TURN;

        assert_eq!((quarter + quarter).fraction, half.fraction);
        assert_eq!((quarter + half).fraction, three_quarters.fraction);
        assert_eq!((quarter + three_quarters).fraction, full.fraction);
    }

    #[test]
    fn test_u128_arithmetic() {
        let quarter = Angle128::QUARTER_TURN;
        let half = Angle128::HALF_TURN;

        // Test addition
        assert_eq!((quarter + quarter).fraction, half.fraction);

        // Test multiplication
        let doubled = quarter * 2u128;
        assert_eq!(doubled.fraction, half.fraction);

        // Test division
        let halved = half / 2u128;
        assert_eq!(halved.fraction, quarter.fraction);
    }

    #[test]
    fn test_division_edge_cases() {
        let angle = Angle64::HALF_TURN;

        // Test division by smallest value
        let divided = angle / 1u64;
        assert_eq!(divided, angle);

        // Test division by power of 2 (should be exact)
        let divided = angle / 4u64;
        assert_eq!(divided, Angle64::QUARTER_TURN / 2u64);

        // Test division of zero angle
        let zero = Angle64::ZERO / 1000u64;
        assert!(zero.is_zero());
    }

    #[test]
    fn test_scalar_operations() {
        let quarter = Angle64::QUARTER_TURN;
        let half = Angle64::HALF_TURN;

        // Test multiplication
        let doubled = quarter * 2u64;
        assert_eq!(doubled.fraction, half.fraction);

        // Test division
        let halved = half / 2u64;
        assert_eq!(halved.fraction, quarter.fraction);
    }

    #[test]
    fn test_large_scalar_division() {
        let angle = Angle64::QUARTER_TURN;

        // Test that division by larger and larger numbers produces smaller and smaller angles
        let small = angle / 1000u64;
        let tiny = angle / (u64::MAX / 2);
        assert!(small.to_radians() > tiny.to_radians());

        // Division by sufficiently large numbers will eventually produce zero
        // due to integer division behavior
        let microscopic = angle / u64::MAX;
        assert!(microscopic.to_radians() >= 0.0); // Should either be 0 or very small positive
    }

    #[test]
    fn test_scalar_multiplication_overflow() {
        let angle = Angle64::QUARTER_TURN;

        // This should wrap around due to overflow
        let result = angle * u64::MAX;
        assert_ne!(result.fraction, angle.fraction);

        // Multiplying by 4 should give us a full turn (0)
        let result = Angle64::QUARTER_TURN * 4u64;
        assert!(result.is_zero());
    }

    #[test]
    fn test_addition_subtraction_reversibility() {
        let quarter = Angle64::QUARTER_TURN;
        let half = Angle64::HALF_TURN;

        let test_angle = quarter + half;
        assert_eq!((test_angle - half).fraction, quarter.fraction);
    }

    #[test]
    fn test_accumulation() {
        let quarter = Angle64::QUARTER_TURN;

        // Test accumulation of quarter turns
        let mut angle = Angle64::ZERO;
        for _ in 0..4 {
            angle = angle + quarter;
        }
        assert_eq!(angle.fraction, Angle64::ZERO.fraction);

        // Test fine-grained accumulation
        let step = Angle64::QUARTER_TURN / 16u64;
        let accumulated = (0..16).fold(Angle64::ZERO, |acc, _| acc + step);
        assert_eq!(accumulated.fraction, Angle64::QUARTER_TURN.fraction);
    }

    #[test]
    fn test_precision_conversion() {
        // Test potential precision loss from higher to lower bit width
        let small_angle = Angle128::new(1);
        let converted: Angle64 = small_angle.into();
        assert!(converted.is_zero()); // Should lose precision

        // Test preservation of significant bits
        let significant_angle = Angle128::QUARTER_TURN;
        let converted: Angle64 = significant_angle.into();
        let back: Angle128 = converted.into();
        assert!((back.to_radians() - significant_angle.to_radians()).abs() < 1e-10);
    }

    #[test]
    fn test_very_small_angles() {
        // Test smallest possible non-zero angle for each bit width
        let small32 = Angle32::new(1);
        let small64 = Angle64::new(1);
        let small128 = Angle128::new(1);

        assert!(!small32.is_zero());
        assert!(!small64.is_zero());
        assert!(!small128.is_zero());

        // These should all be very close to zero but not exactly zero
        assert!(small32.to_radians() > 0.0);
        assert!(small64.to_radians() > 0.0);
        assert!(small128.to_radians() > 0.0);
    }

    #[test]
    fn test_near_full_turn() {
        // Test angles very close to a full turn
        let almost_full32 = Angle32::new(u32::MAX);
        let almost_full64 = Angle64::new(u64::MAX);
        let almost_full128 = Angle128::new(u128::MAX);

        // Should all be very close to TAU but not exactly TAU
        assert!((almost_full32.to_radians() - TAU).abs() < 1e-6);
        assert!((almost_full64.to_radians() - TAU).abs() < 1e-10);
        assert!((almost_full128.to_radians() - TAU).abs() < 1e-10);

        // Adding 1 to these should wrap to 0
        assert!((almost_full64 + Angle64::new(1)).is_zero());
    }

    // Conversion and Representation
    #[test]
    fn test_to_radians() {
        let angle = Angle32 {
            fraction: u32::MAX / 2,
        };
        assert!((angle.to_radians() - PI).abs() < 1e-6);
    }

    #[test]
    fn test_from_radians_boundary() {
        use std::f64::consts::TAU;

        // Test exact TAU (should wrap to 0)
        let angle = Angle64::from_radians(TAU);
        assert!(angle.is_zero());

        // Test negative angles
        let angle = Angle64::from_radians(-PI);
        assert_eq!(angle.fraction, Angle64::HALF_TURN.fraction);

        // Test slightly negative angles
        let angle = Angle64::from_radians(-0.1);
        assert!((angle.to_radians() - (TAU - 0.1)).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "Failed to convert")]
    fn test_from_radians_overflow() {
        let _ = Angle32::from_radians(f64::INFINITY);
    }

    #[test]
    fn test_display() {
        let angle = Angle64 {
            fraction: u64::MAX / 4,
        };
        assert_eq!(format!("{angle}"), "0.250000 turns");
    }

    #[test]
    fn test_bit_width_conversions() {
        let angle32 = Angle32::QUARTER_TURN;
        let angle64: Angle64 = angle32.into();
        let back32: Angle32 = angle64.into();
        assert_eq!(angle32, back32);

        let angle128: Angle128 = angle64.into();
        let back64: Angle64 = angle128.into();
        assert_eq!(angle64, back64);
    }

    // Trigonometric Functions
    #[test]
    fn test_trig_functions() {
        let quarter = Angle64::QUARTER_TURN;
        assert!((quarter.sin() - 1.0).abs() < 1e-10);
        assert!(quarter.cos().abs() < 1e-10);

        let eighth = quarter / 2u64;
        assert!((eighth.sin() - FRAC_PI_4.sin()).abs() < 1e-10);
        assert!((eighth.cos() - FRAC_PI_4.cos()).abs() < 1e-10);

        let angle = Angle64::from_radians(FRAC_PI_4);
        assert!((angle.tan() - 1.0).abs() < 1e-10);
    }

    // Special Properties
    #[test]
    fn test_wrapping_behavior() {
        let quarter = Angle64::QUARTER_TURN;
        let full = Angle64::FULL_TURN;

        // Test wrapping around full circle
        let wrapped = full + quarter;
        assert_eq!(wrapped.fraction, quarter.fraction);

        // Test multiple wraps
        let mut angle = Angle64::ZERO;
        for _ in 0..8 {
            // Two full rotations
            angle = angle + quarter;
        }
        assert_eq!(angle.fraction, Angle64::ZERO.fraction);
    }

    #[test]
    fn test_ordering() {
        let zero = Angle64::ZERO;
        let quarter = Angle64::QUARTER_TURN;
        let half = Angle64::HALF_TURN;

        assert!(zero < quarter);
        assert!(quarter < half);
        assert!(zero < half);

        let angles = vec![half, zero, quarter];
        let mut sorted = angles.clone();
        sorted.sort();
        assert_eq!(sorted, vec![zero, quarter, half]);
    }

    #[test]
    fn test_fraction_ordering() {
        let zero = Angle64::ZERO;
        let quarter = Angle64::QUARTER_TURN;
        let half = Angle64::HALF_TURN;
        let three_quarters = Angle64::THREE_QUARTERS_TURN;
        let full = Angle64::FULL_TURN;

        // Test fraction ordering
        assert!(quarter.fraction < half.fraction);
        assert!(half.fraction < three_quarters.fraction);
        assert!(three_quarters.fraction > quarter.fraction);

        // Test extremes
        assert_eq!(zero.fraction, 0);
        assert_eq!(full.fraction, 0); // Full turn wraps to 0

        // Test that fractions increase monotonically
        assert!(zero.fraction < quarter.fraction);
        assert!(quarter.fraction < half.fraction);
        assert!(half.fraction < three_quarters.fraction);
    }

    // Implementation Details/Edge Cases
    #[test]
    fn test_effective_modulus_u8() {
        let max_value = u8::MAX; // 255
        let effective_modulus = max_value.wrapping_add(1);
        assert_eq!(
            effective_modulus, 0,
            "Effective modulus should wrap to 0 for u8"
        );
    }

    #[test]
    fn test_effective_modulus_u64() {
        let max_value = u64::MAX;
        let effective_modulus = max_value.wrapping_add(1);
        assert_eq!(
            effective_modulus, 0,
            "Effective modulus should wrap to 0 for u64"
        );
    }

    #[test]
    fn test_angle_arithmetic_with_u16() {
        let a = Angle { fraction: 100_u16 };
        let b = Angle { fraction: 200_u16 };
        let result = a + b;
        assert!(result.fraction < u16::MAX, "Result must be within bounds");
    }

    #[test]
    fn test_angle_u32_to_u64_lossless() {
        let angle_u32 = Angle { fraction: 1_u32 };
        let angle_u64: Angle<u64> = angle_u32.into();
        assert_eq!(angle_u64.fraction, 1_u64 << 32);

        let angle_u32 = Angle { fraction: u32::MAX };
        let angle_u64: Angle<u64> = angle_u32.into();
        assert_eq!(angle_u64.fraction, u64::from(u32::MAX) << 32);
    }

    #[test]
    fn test_angle_u64_to_u32_lossy() {
        let angle_u64 = Angle { fraction: 1_u64 << 32 };
        let angle_u32: Angle<u32> = angle_u64.into();
        assert_eq!(angle_u32.fraction, 1_u32);

        let angle_u64 = Angle { fraction: u64::MAX };
        let angle_u32: Angle<u32> = angle_u64.into();
        assert_eq!(angle_u32.fraction, u32::MAX);
    }

    #[test]
    fn test_angle_constants_conversion() {
        let zero_u32 = Angle { fraction: 0_u32 };
        let zero_u64: Angle<u64> = zero_u32.into();
        assert_eq!(zero_u64.fraction, 0_u64);
        assert_eq!(Angle::<u32>::from(zero_u64).fraction, 0_u32);

        let quarter_u32 = Angle { fraction: 1_u32 << 30 }; // 2^30
        let quarter_u64: Angle<u64> = quarter_u32.into();
        assert_eq!(quarter_u64.fraction, 1_u64 << 62);
        assert_eq!(Angle::<u32>::from(quarter_u64).fraction, 1_u32 << 30);

        let half_u32 = Angle { fraction: 1_u32 << 31 }; // 2^31
        let half_u64: Angle<u64> = half_u32.into();
        assert_eq!(half_u64.fraction, 1_u64 << 63);
        assert_eq!(Angle::<u32>::from(half_u64).fraction, 1_u32 << 31);

        let full_u32 = Angle { fraction: 0_u32 }; // Wraps to 0
        let full_u64: Angle<u64> = full_u32.into();
        assert_eq!(full_u64.fraction, 0_u64);
        assert_eq!(Angle::<u32>::from(full_u64).fraction, 0_u32);
    }

    #[test]
    fn test_round_trip_conversion_u32_u64() {
        // Test that converting from Angle<u32> to Angle<u64> and back is lossless
        let angle_u32 = Angle::<u32>::new(123456);
        let converted: Angle<u64> = angle_u32.into();
        let back: Angle<u32> = converted.into();
        assert_eq!(angle_u32, back);
    }

    #[test]
    fn test_round_trip_conversion_u64_u32() {
        // Test that converting from Angle<u64> to Angle<u32> and back is approximately correct
        let angle_u64 = Angle::<u64>::new(1 << 40);
        let converted: Angle<u32> = angle_u64.into();
        let back: Angle<u64> = converted.into();
        // Check for approximate equality due to lossy conversion
        assert_eq!(back.fraction >> 32, angle_u64.fraction >> 32);
    }

    #[test]
    fn test_randomized_values_u32_to_u64() {
        // Test with randomized values to ensure correctness for u32 -> u64 -> u32 conversions
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let random_u32: u32 = rng.gen();
            let angle_u32 = Angle::<u32>::new(random_u32);
            let converted: Angle<u64> = angle_u32.into();
            let back: Angle<u32> = converted.into();
            assert_eq!(angle_u32, back);
        }
    }

    #[test]
    fn test_randomized_values_u64_to_u32() {
        // Test with randomized values to ensure correctness for u64 -> u32 -> u64 conversions
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let random_u64: u64 = rng.gen();
            let angle_u64 = Angle::<u64>::new(random_u64);
            let converted: Angle<u32> = angle_u64.into();
            let back: Angle<u64> = converted.into();
            // Lossy conversion, check for approximate equality
            assert_eq!(back.fraction >> 32, angle_u64.fraction >> 32);
        }
    }

    #[test]
    fn test_subdivision_values() {
        // Test boundary cases for small subdivision values during conversions
        let angle_u32 = Angle::<u32>::new(1);
        let converted: Angle<u64> = angle_u32.into();
        assert_eq!(converted.fraction, 1_u64 << 32);

        let angle_u64 = Angle::<u64>::new(1);
        let converted: Angle<u32> = angle_u64.into();
        assert_eq!(converted.fraction, 0); // Too small to be represented in u32
    }

    #[test]
    fn test_near_boundary_values() {
        // Test conversion near boundary values to ensure correctness
        let angle_u32 = Angle::<u32>::new(u32::MAX - 1);
        let converted: Angle<u64> = angle_u32.into();
        assert_eq!(converted.fraction, (u64::from(u32::MAX - 1) << 32));

        let angle_u64 = Angle::<u64>::new((1 << 32) - 1);
        let converted: Angle<u32> = angle_u64.into();
        assert_eq!(converted.fraction, 0); // Fractional part lost
    }

    #[test]
    fn test_overflow_and_underflow() {
        // Test cases for overflow and underflow during conversions
        let angle_u32 = Angle::<u32>::new(u32::MAX);
        let converted: Angle<u64> = angle_u32.into();
        assert_eq!(converted.fraction, (u64::from(u32::MAX) << 32));

        let angle_u64 = Angle::<u64>::new(u64::MAX);
        let converted: Angle<u32> = angle_u64.into();
        assert_eq!(converted.fraction, u32::MAX);
    }

    #[test]
    fn test_non_uniform_scaling() {
        // Test non-uniform scaling conversions
        let angle_u32 = Angle::<u32>::new(u32::MAX / 3);
        let converted: Angle<u64> = angle_u32.into();
        assert_eq!(converted.fraction, (u64::from(u32::MAX / 3) << 32));

        let angle_u64 = Angle::<u64>::new(u64::MAX / 3);
        let converted: Angle<u32> = angle_u64.into();
        assert_eq!(converted.fraction, (u32::MAX / 3));
    }

    #[test]
    fn test_constants_conversion() {
        // Test that predefined constants are correctly converted between types
        assert_eq!(Angle::<u32>::from(Angle::<u64>::ZERO), Angle::<u32>::ZERO);
        assert_eq!(Angle::<u64>::from(Angle::<u32>::ZERO), Angle::<u64>::ZERO);

        assert_eq!(Angle::<u32>::from(Angle::<u64>::HALF_TURN), Angle::<u32>::HALF_TURN);
        assert_eq!(Angle::<u64>::from(Angle::<u32>::HALF_TURN), Angle::<u64>::HALF_TURN);

        assert_eq!(Angle::<u32>::from(Angle::<u64>::QUARTER_TURN), Angle::<u32>::QUARTER_TURN);
        assert_eq!(Angle::<u64>::from(Angle::<u32>::QUARTER_TURN), Angle::<u64>::QUARTER_TURN);

        assert_eq!(Angle::<u32>::from(Angle::<u64>::FULL_TURN), Angle::<u32>::FULL_TURN);
        assert_eq!(Angle::<u64>::from(Angle::<u32>::FULL_TURN), Angle::<u64>::FULL_TURN);
    }

    #[test]
    #[should_panic(expected = "Denominator cannot be zero")]
    fn test_from_fraction_panic_on_zero_denominator() {
        Angle64::from_fraction(1, 0);
    }

    #[test]
    #[should_panic(expected = "Numerator multiplication overflow")]
    fn test_from_fraction_panic_on_numerator_overflow() {
        Angle64::from_fraction(i64::MAX, 1);
    }

    #[test]
    fn test_from_fraction_valid_cases() {
        assert_eq!(
            Angle64::from_fraction(1, 2).fraction,
            Angle64::HALF_TURN.fraction
        );
        assert_eq!(
            Angle64::from_fraction(1, 4).fraction,
            Angle64::HALF_TURN.fraction / 2
        );
        assert_eq!(
            Angle64::from_fraction(3, 4).fraction,
            3 * (Angle64::HALF_TURN.fraction / 2)
        );
    }


}
