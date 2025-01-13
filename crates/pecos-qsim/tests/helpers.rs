use num_complex::Complex64;

/// Compare two quantum states up to global phase.
///
/// This function ensures that two state vectors represent the same quantum state,
/// accounting for potential differences in global phase.
///
/// # Arguments
/// - `state1`: A reference to the first state vector.
/// - `state2`: A reference to the second state vector.
///
/// # Panics
/// The function will panic if the states differ in norm or relative phase beyond a small numerical tolerance.
#[allow(dead_code)]
pub fn assert_states_equal(state1: &[Complex64], state2: &[Complex64]) {
    const TOLERANCE: f64 = 1e-10;

    if state1[0].norm() < TOLERANCE && state2[0].norm() < TOLERANCE {
        // Both first components near zero, compare other components directly
        for (index, (a, b)) in state1.iter().zip(state2.iter()).enumerate() {
            assert!(
                (a.norm() - b.norm()).abs() < TOLERANCE,
                "States differ in magnitude at index {index}: {a} vs {b}"
            );
        }
    } else {
        // Get phase from the first pair of non-zero components
        let ratio = match state1
            .iter()
            .zip(state2.iter())
            .find(|(a, b)| a.norm() > TOLERANCE && b.norm() > TOLERANCE)
        {
            Some((a, b)) => b / a,
            None => panic!("States have no corresponding non-zero components"),
        };
        println!("Phase ratio between states: {ratio:?}");

        for (index, (a, b)) in state1.iter().zip(state2.iter()).enumerate() {
            assert!(
                (a * ratio - b).norm() < TOLERANCE,
                "States differ at index {index}: {a} vs {b} (adjusted with ratio {ratio:?}), diff = {}",
                (a * ratio - b).norm()
            );
        }
    }
}
