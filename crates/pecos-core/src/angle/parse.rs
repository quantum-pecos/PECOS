//! String parsing implementation for angles
//!
//! Provides functionality to parse angles from string representations of radians,
//! including pi-based expressions.

use super::Angle;
use num_traits::{
    Bounded, FromPrimitive, ToPrimitive, Unsigned, WrappingAdd, WrappingMul, WrappingNeg,
    WrappingSub, Zero,
};
use std::ops::Rem;
use std::str::FromStr;

/// Error types for angle parsing
#[derive(Debug, PartialEq, Eq)]
pub enum ParseAngleError {
    /// Input string format was invalid
    InvalidFormat,
    /// Failed to parse numerator
    InvalidNumerator,
    /// Failed to parse denominator
    InvalidDenominator,
    /// Denominator was zero
    DivisionByZero,
    /// The resulting angle would be too large to represent
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
    + Rem<Output = T>,
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
    + Rem<Output = T>,
{
    /// Creates an angle from a string representation of radians.
    ///
    /// Supports formats like:
    /// - "π" or "pi" or "PI"
    /// - "π/2" or "pi/2"
    /// - "3π" or "3pi"
    /// - "3π/2" or "3pi/2"
    /// - "3 π / 2" or "3*π/2"
    ///
    /// # Examples
    /// ```
    /// use pecos_core::Angle64;
    /// let three_halves_pi = Angle64::from_str_radians("3π/2").unwrap();
    /// assert_eq!(three_halves_pi, Angle64::THREE_QUARTERS_TURN);
    /// ```
    ///
    /// # Errors
    /// Returns `ParseAngleError` if:
    /// - The string format is invalid
    /// - The numerator or denominator can't be parsed
    /// - The denominator is zero
    /// - The resulting angle would overflow
    pub fn from_str_radians(s: &str) -> Result<Self, ParseAngleError> {
        let s = s.trim().replace(' ', "").replace('*', "").to_lowercase();

        if s == "pi" || s == "π" {
            return Ok(Self::new(T::max_value() / T::from_u8(2).unwrap()));
        }

        let (num_part, den_part) = if let Some((n, d)) = s.split_once('/') {
            (n, Some(d))
        } else {
            (s.as_str(), None)
        };

        let (num, has_pi) = if num_part.contains("pi") || num_part.contains('π') {
            let n = num_part.replace("pi", "").replace('π', "");
            let num = if n.is_empty() || n == "-" {
                if n == "-" { -1.0 } else { 1.0 }
            } else {
                n.parse::<f64>().map_err(|_| ParseAngleError::InvalidNumerator)?
            };
            (num, true)
        } else {
            (num_part.parse::<f64>().map_err(|_| ParseAngleError::InvalidNumerator)?, false)
        };

        let den = if let Some(d) = den_part {
            let den = d.parse::<f64>().map_err(|_| ParseAngleError::InvalidDenominator)?;
            if den == 0.0 {
                return Err(ParseAngleError::DivisionByZero);
            }
            den
        } else {
            1.0
        };

        let radians = if has_pi {
            num * std::f64::consts::PI / den
        } else {
            num / den
        };

        let max_value = T::max_value()
            .to_f64()
            .ok_or(ParseAngleError::Overflow)?;

        let normalized = radians.rem_euclid(std::f64::consts::TAU);
        let fraction = (normalized / std::f64::consts::TAU * max_value).round();

        // Ensure the fraction aligns perfectly with constants
        let fraction = if fraction > max_value {
            max_value
        } else if fraction < 0.0 {
            0.0
        } else {
            fraction
        };

        T::from_f64(fraction)
            .map(Self::new)
            .ok_or(ParseAngleError::Overflow)
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Angle64;
    use std::f64::consts::{FRAC_PI_2, PI};

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
        assert!(Angle64::from_str_radians(&format!("{}pi", i64::MAX)).is_err());

        // Negative numbers
        let neg_quarter = Angle64::from_str_radians("-pi/2").unwrap();
        assert!((neg_quarter.to_radians() + FRAC_PI_2).abs() < 1e-10);

        // Zero
        assert!(Angle64::from_str_radians("0").unwrap().is_zero());
        assert!(Angle64::from_str_radians("0pi").unwrap().is_zero());

        // Just negative sign
        assert_eq!(
            Angle64::from_str_radians("-pi").unwrap().to_radians(),
            -PI
        );
    }
}