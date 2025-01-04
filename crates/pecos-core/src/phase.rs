use num_complex::Complex64;

#[allow(clippy::module_name_repetitions)]
pub mod quarter_phase;
pub mod sign;

pub trait Phase {
    #[must_use]
    fn phase(&self) -> &Self {
        self
    }
    fn to_complex(&self) -> Complex64;
    #[must_use]
    fn conjugate(&self) -> Self;

    #[must_use]
    fn multiply(&self, other: &Self) -> Self;
}
