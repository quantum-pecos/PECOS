use crate::Phase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)] // Ensures same binary representation as Phase
pub enum Sign {
    PlusOne = 0b00,
    MinusOne = 0b01,
}

impl Sign {
    /// Multiplies two `Sign` values using XOR (superfast).
    #[must_use]
    pub fn multiply(self, other: Self) -> Self {
        unsafe { std::mem::transmute((self as u8) ^ (other as u8)) }
    }
}

impl TryFrom<Phase> for Sign {
    type Error = &'static str;

    fn try_from(phase: Phase) -> Result<Self, Self::Error> {
        match phase {
            Phase::PlusOne => Ok(Sign::PlusOne),
            Phase::MinusOne => Ok(Sign::MinusOne),
            _ => Err("Invalid phase: Sign can only be PlusOne or MinusOne"),
        }
    }
}

impl From<Sign> for Phase {
    fn from(sign: Sign) -> Phase {
        // Safe because `Sign` variants map directly to `Phase` variants
        unsafe { std::mem::transmute(sign as u8) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_multiply() {
        let sign = Sign::PlusOne;

        match sign {
            Sign::PlusOne => assert_eq!(sign.multiply(Sign::PlusOne), Sign::PlusOne),
            Sign::MinusOne => assert_eq!(sign.multiply(Sign::MinusOne), Sign::PlusOne),
        }
    }
}
