use crate::{Pauli, PauliString, QubitId, Sign};

#[allow(dead_code)]
pub struct PauliStabilizerString {
    sign: Sign,
    paulis: Vec<(Pauli, QubitId)>,
}

// TODO: Consider having underlying Pauli vec for both PauliString and PauliStabilizerString

#[allow(dead_code)]
impl PauliStabilizerString {
    pub fn new() -> Self {
        Self {
            sign: Sign::PlusOne,
            paulis: Vec::new(),
        }
    }

    pub fn try_from_pauli_string(pauli_string: &PauliString) -> Result<Self, String> {
        let sign = Sign::try_from(pauli_string.get_phase())
            .map_err(|_| "Invalid phase for PauliStabilizerString")?;

        Ok(Self {
            sign,
            paulis: (*pauli_string.get_paulis().clone()).to_owned(),
        })
    }

    /// Multiply two `PauliStabilizerString`s.
    pub fn multiply(&self, other: &Self) -> Self {
        let new_sign = self.sign.multiply(other.sign);
        let mut new_paulis = self.paulis.clone();

        for &(pauli, qubit) in &other.paulis {
            // Simplify this as per your Pauli multiplication logic
            new_paulis.push((pauli, qubit));
        }

        // Simplify the new_paulis as necessary
        Self {
            sign: new_sign,
            paulis: new_paulis,
        }
    }

    /// Check if the `PauliStabilizerString` commutes with another.
    pub fn commutes_with(&self, _other: &Self) -> bool {
        // Implement commutation logic based on pauli positions
        todo!()
    }
}

impl Default for PauliStabilizerString {
    fn default() -> Self {
        Self::new()
    }
}
