use pecos_core::{Pauli, PauliBitmap, PauliOperator, PauliSparse, QuarterPhase, VecSet};

/// Helper function to test multiplication of Pauli operators.
fn test_multiply<O: PauliOperator + PartialEq + std::fmt::Debug>(
    op1: &O,
    op2: &O,
    expected: &O,
    expected_phase: QuarterPhase,
) {
    let result = op1.multiply(op2);

    // Compare operator components (excluding phase)
    assert_eq!(
        result.x_positions(),
        expected.x_positions(),
        "X positions mismatch for {op1:?} * {op2:?}. Expected X positions: {:?}, got: {:?}",
        expected.x_positions(),
        result.x_positions()
    );
    assert_eq!(
        result.z_positions(),
        expected.z_positions(),
        "Z positions mismatch for {op1:?} * {op2:?}. Expected Z positions: {:?}, got: {:?}",
        expected.z_positions(),
        result.z_positions()
    );

    // Compare phases
    assert_eq!(
        result.phase(),
        expected_phase,
        "Phase mismatch for {op1:?} * {op2:?}. Expected phase: {expected_phase:?}, got: {:?}",
        result.phase()
    );
}

/// Helper function to test the weight of a Pauli operator.
fn test_weight<O: PauliOperator>(op: &O, expected_weight: usize) {
    let weight = op.weight();
    assert_eq!(weight, expected_weight, "Failed weight test with {op:?}");
}

/// Helper function to test commutation relations of Pauli operators.
fn test_commutes_with<O: PauliOperator>(op1: &O, op2: &O, expected_commutes: bool) {
    let commutes = op1.commutes_with(op2);
    assert_eq!(
        commutes, expected_commutes,
        "Failed commutes_with test for {op1:?} and {op2:?}"
    );
}

/// Run the tests for a specific implementation of `PauliOperator`.
fn run_pauli_operator_tests<O: PauliOperator + PartialEq + std::fmt::Debug>(
    make_identity: fn() -> O,
    make_x: fn() -> O,
    make_z: fn() -> O,
    make_y: fn() -> O,
) {
    let id = make_identity();
    let x = make_x();
    let z = make_z();
    let y = make_y();

    // Test multiplication
    test_multiply(&id, &id, &id, QuarterPhase::PlusOne); // I * I = I
    test_multiply(&x, &x, &id, QuarterPhase::PlusOne); // X * X = I
    test_multiply(&z, &z, &id, QuarterPhase::PlusOne); // Z * Z = I
    test_multiply(&y, &y, &id, QuarterPhase::PlusOne); // Y * Y = I
    test_multiply(&x, &z, &y, QuarterPhase::MinusI); // X * Z = -i Y
    test_multiply(&z, &x, &y, QuarterPhase::PlusI); // Z * X = +i Y

    // Test weight
    test_weight(&id.clone(), 0); // I has weight 0
    test_weight(&x.clone(), 1); // X has weight 1
    test_weight(&z.clone(), 1); // Z has weight 1
    test_weight(&y.clone(), 1); // Y has weight 1

    // Test commutation
    test_commutes_with(&id.clone(), &x.clone(), true); // I commutes with X
    test_commutes_with(&x.clone(), &z.clone(), false); // X anti-commutes with Z
    test_commutes_with(&z, &x, false); // Z anti-commutes with X
    test_commutes_with(&y, &y, true); // Y commutes with itself
}

type BasicSparse = PauliSparse<VecSet<usize>>;

#[test]
fn test_pauli_sparse() {
    run_pauli_operator_tests(
        BasicSparse::new,                         // Identity operator
        || BasicSparse::from_single(0, Pauli::X), // X operator
        || BasicSparse::from_single(0, Pauli::Z), // Z operator
        || BasicSparse::from_single(0, Pauli::Y), // Y operator
    );
}

#[test]
fn test_pauli_bitmap() {
    run_pauli_operator_tests(
        PauliBitmap::new,                         // Identity operator
        || PauliBitmap::from_single(0, Pauli::X), // X operator
        || PauliBitmap::from_single(0, Pauli::Z), // Z operator
        || PauliBitmap::from_single(0, Pauli::Y), // Y operator
    );
}
