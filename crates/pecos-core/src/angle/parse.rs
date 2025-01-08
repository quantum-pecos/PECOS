//! String parsing implementation for angles.
//!
//! This module provides functionality to parse angles from string representations,
//! with support for both radian-based and π-based expressions. The parser provides:
//!
//! - Exact ratio arithmetic for π-based expressions
//! - Support for decimal values in radians
//! - Automatic normalization to [0, 2π) range
//! - Clear error reporting for various failure modes
//!
//! The implementation prioritizes exact representation where possible, using ratio
//! arithmetic for π-based expressions to avoid floating point imprecision.

use super::Angle;
use num_traits::{
    Bounded, FromPrimitive, PrimInt, ToPrimitive, Unsigned, WrappingAdd, WrappingMul, WrappingNeg,
    WrappingSub, Zero,
};
use std::fmt::Debug;
use std::ops::Rem;
use std::str::FromStr;

/// Errors that can occur when parsing angle strings.
///
/// This error type distinguishes between different failure modes:
/// - Invalid format (e.g., malformed expressions)
/// - Numeric parsing failures (invalid numerators/denominators)
/// - Mathematical errors (division by zero)
/// - Range errors (values too large to represent)
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Eq)]
pub enum ParseAngleError {
    /// The input string format was invalid
    InvalidFormat,
    /// The numerator portion could not be parsed as a number
    InvalidNumerator,
    /// The denominator portion could not be parsed as a number
    InvalidDenominator,
    /// The denominator was zero, which would result in division by zero
    DivisionByZero,
    /// The resulting angle value would be too large to represent in the target type
    Overflow,
}

impl std::fmt::Display for ParseAngleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid angle format"),
            Self::InvalidNumerator => write!(f, "invalid numerator"),
            Self::InvalidDenominator => write!(f, "invalid denominator"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::Overflow => write!(f, "angle too large to represent"),
        }
    }
}

impl std::error::Error for ParseAngleError {}

impl<T> FromStr for Angle<T>
where
    T: Unsigned
        + Copy
        + ToPrimitive
        + FromPrimitive
        + Zero
        + Bounded
        + WrappingAdd
        + WrappingSub
        + WrappingMul
        + WrappingNeg
        + Rem<Output = T>
        + PrimInt
        + Default
        + Debug
        + TryFrom<u128>,
{
    type Err = ParseAngleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radians(s)
    }
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
        + WrappingMul
        + WrappingNeg
        + Rem<Output = T>
        + PrimInt
        + Default
        + Debug
        + TryFrom<u128>,
{
    /// Creates an angle from a string representation of radians.
    ///
    /// # Format
    /// Supports several formats for angle specification:
    /// - "π" or "pi" or "PI": Represents π radians (half turn)
    /// - "π/2" or "pi/2": Fractions of π
    /// - "3π" or "3pi": Multiples of π
    /// - "3π/2" or "3pi/2": Complex fractions of π
    /// - "3 π / 2" or "3*π/2": Spaces and * are allowed and ignored
    /// - "1.5" or "-1.5": Raw radian values
    ///
    /// # Normalization
    /// All angles are normalized to the range [0, 2π):
    /// - Negative angles wrap around (e.g., "-π/2" becomes "3π/2")
    /// - Values greater than 2π wrap around modulo 2π
    ///
    /// # Precision
    /// - For pi-based fractions (like "π/2"), uses exact ratio arithmetic
    /// - For decimal values, uses floating point conversion
    ///
    /// # Examples
    /// ```
    /// use pecos_core::Angle64;
    /// let three_halves_pi = Angle64::from_str_radians("3π/2").unwrap();
    /// assert_eq!(three_halves_pi, Angle64::THREE_QUARTERS_TURN);
    ///
    /// // Negative angles wrap around
    /// let neg_half_pi = Angle64::from_str_radians("-π/2").unwrap();
    /// assert_eq!(neg_half_pi, Angle64::THREE_QUARTERS_TURN);
    /// ```
    ///
    /// # Errors
    /// Returns `ParseAngleError` if:
    /// - The string format is invalid (e.g., malformed expressions)
    /// - The numerator or denominator can't be parsed as numbers
    /// - The denominator is zero
    /// - The resulting angle would overflow the target type
    #[allow(clippy::cast_precision_loss)]
    pub fn from_str_radians(s: &str) -> Result<Self, ParseAngleError> {
        println!("Parsing string: {s}");
        let s = s.trim().replace([' ', '*'], "").to_lowercase();
        println!("After trim and lowercase: {s}");

        // First check if it's just "pi" or "π" or "-pi" or "-π"
        if s == "pi" || s == "π" {
            return Ok(Self::from_radians(std::f64::consts::PI));
        } else if s == "-pi" || s == "-π" {
            return Ok(Self::from_radians(-std::f64::consts::PI));
        }

        // If we have a decimal point, parse as floating point radians
        if s.contains('.') {
            let value = s
                .parse::<f64>()
                .map_err(|_| ParseAngleError::InvalidNumerator)?;
            if !value.is_finite() {
                return Err(ParseAngleError::Overflow);
            }
            return Ok(Self::from_radians(value));
        }

        // Split into numerator and denominator parts
        let (num_part, den_part) = if let Some((n, d)) = s.split_once('/') {
            println!("Split into num: {n}, den: {d}");
            (n, Some(d))
        } else {
            println!("No split, full value: {s}");
            (s.as_str(), None)
        };

        // Parse numerator, handling pi/π multiplier
        let (num_val, has_pi) = if num_part.contains("pi") || num_part.contains('π') {
            let n = num_part.replace("pi", "").replace('π', "");
            println!("Pi case, n after replacement: '{n}'");
            let num = if n.is_empty() {
                println!("Empty n, using 1");
                1
            } else if n == "-" {
                println!("Just minus sign, using -1");
                -1
            } else {
                // Try parsing - if it fails, determine if it's invalid format or overflow
                match n.parse::<i64>() {
                    Ok(val) => {
                        println!("Parsed i64: {val}");
                        val
                    }
                    Err(e) => {
                        println!("Failed to parse i64: {e}");
                        // Check if it's a valid number format that's just too big
                        let is_valid = n.starts_with('-')
                            && n[1..].chars().all(|c| c.is_ascii_digit())
                            || n.chars().all(|c| c.is_ascii_digit());
                        if is_valid {
                            return Err(ParseAngleError::Overflow);
                        }
                        return Err(ParseAngleError::InvalidNumerator);
                    }
                }
            };
            (num, true)
        } else if let Ok(val) = num_part.parse::<i64>() {
            (val, false)
        } else {
            let is_valid = num_part.starts_with('-')
                && num_part[1..].chars().all(|c| c.is_ascii_digit())
                || num_part.chars().all(|c| c.is_ascii_digit());
            if is_valid {
                return Err(ParseAngleError::Overflow);
            }
            return Err(ParseAngleError::InvalidNumerator);
        };
        println!("Parsed num_val: {num_val}, has_pi: {has_pi}");

        // Parse denominator
        let den_val = if let Some(d) = den_part {
            match d.parse::<i64>() {
                Ok(den) => {
                    if den == 0 {
                        return Err(ParseAngleError::DivisionByZero);
                    }
                    den
                }
                Err(_) => return Err(ParseAngleError::InvalidDenominator),
            }
        } else {
            1
        };
        println!("Parsed den_val: {den_val}");

        // Check for potential overflow in multiplication when has_pi is true
        if has_pi && (num_val.checked_mul(2).is_none() || den_val.checked_mul(2).is_none()) {
            return Err(ParseAngleError::Overflow);
        }

        // Convert to angle using appropriate method
        let result = if has_pi {
            Ok(Self::from_turn_ratio(num_val, den_val * 2))
        } else {
            let radians = num_val as f64 / den_val as f64;
            println!("Converting non-pi case to radians: {radians}");
            Ok(Self::from_radians(radians))
        };

        println!("Final result: {result:?}");
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Angle64;

    #[test]
    fn test_parse_basic_pi() {
        assert_eq!(Angle64::from_str_radians("pi").unwrap(), Angle64::HALF_TURN);
        assert_eq!(Angle64::from_str_radians("π").unwrap(), Angle64::HALF_TURN);
        assert_eq!(Angle64::from_str_radians("PI").unwrap(), Angle64::HALF_TURN);
    }

    #[test]
    fn test_parse_fractions() {
        assert_eq!(
            Angle64::from_str_radians("pi/2").unwrap(),
            Angle64::QUARTER_TURN
        );
        assert_eq!(
            Angle64::from_str_radians("π/2").unwrap(),
            Angle64::QUARTER_TURN
        );
        assert_eq!(
            Angle64::from_str_radians("PI/2").unwrap(),
            Angle64::QUARTER_TURN
        );
    }

    #[test]
    fn test_parse_multiples() {
        assert_eq!(
            Angle64::from_str_radians("2pi").unwrap(),
            Angle64::FULL_TURN
        );
        assert_eq!(Angle64::from_str_radians("2π").unwrap(), Angle64::FULL_TURN);
        assert_eq!(
            Angle64::from_str_radians("2PI").unwrap(),
            Angle64::FULL_TURN
        );
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(
            Angle64::from_str_radians("3pi/2").unwrap(),
            Angle64::THREE_QUARTERS_TURN
        );
        assert_eq!(
            Angle64::from_str_radians("3π/2").unwrap(),
            Angle64::THREE_QUARTERS_TURN
        );
        assert_eq!(
            Angle64::from_str_radians("3*pi/2").unwrap(),
            Angle64::THREE_QUARTERS_TURN
        );
    }

    #[test]
    fn test_parse_with_spaces() {
        assert_eq!(
            Angle64::from_str_radians("3 pi / 2").unwrap(),
            Angle64::THREE_QUARTERS_TURN
        );
        assert_eq!(
            Angle64::from_str_radians("3 * pi / 2").unwrap(),
            Angle64::THREE_QUARTERS_TURN
        );
        assert_eq!(
            Angle64::from_str_radians("  pi  ").unwrap(),
            Angle64::HALF_TURN
        );
    }

    #[test]
    fn test_parse_non_pi() {
        let angle = Angle64::from_str_radians("1.5").unwrap();
        assert!((angle.to_radians() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_parse_errors() {
        assert_eq!(
            Angle64::from_str_radians("invalid"),
            Err(ParseAngleError::InvalidNumerator)
        );
        assert_eq!(
            Angle64::from_str_radians("pi/0"),
            Err(ParseAngleError::DivisionByZero)
        );
        assert_eq!(
            Angle64::from_str_radians("pi/invalid"),
            Err(ParseAngleError::InvalidDenominator)
        );
    }

    #[test]
    fn test_parse_fromstr_trait() {
        let angle: Angle64 = "pi/2".parse().unwrap();
        assert_eq!(angle, Angle64::QUARTER_TURN);
    }

    #[test]
    fn test_parse_edge_cases() {
        // Very large numbers
        assert!(
            Angle64::from_str_radians(&format!("{}pi", i64::MAX)).is_err(),
            "failed to deal with large numbers"
        );

        // Negative numbers - Note: -pi/2 gets normalized to 3pi/2
        let neg_quarter = Angle64::from_str_radians("-pi/2").unwrap();
        assert_eq!(
            neg_quarter,
            Angle64::THREE_QUARTERS_TURN,
            "failed to handle negative numbers"
        );

        // Zero
        assert!(
            Angle64::from_str_radians("0").unwrap().is_zero(),
            "failed to handle zero (0)"
        );
        assert!(
            Angle64::from_str_radians("0pi").unwrap().is_zero(),
            "failed to handle zero (0pi)"
        );

        // Just negative sign
        assert_eq!(
            Angle64::from_str_radians("-pi").unwrap(),
            Angle64::HALF_TURN,
            "failed to handle -pi"
        );
    }
}
