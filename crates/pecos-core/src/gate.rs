pub enum Gate {
    // Paulis
    I,
    X,
    Y,
    Z,

    // Sqrt of Paulis
    SX,
    SXdg,
    SY,
    SYdg,
    SZ,
    SZdg,

    // Hadamards
    H,
    H2,
    H3,
    H4,
    H5,
    H6,

    // Face rotations
    F,
    Fdg,
    F2,
    F2dg,
    F3,
    F3dg,
    F4,
    F4dg,

    // Controlled-Paulis
    CX,
    CY,
    CZ,

    // Sqrt of two Paulis
    SXX,
    SXXdg,
    SYY,
    SYYdg,
    SZZ,
    SZZdg,

    // Other TQ gates
    SWAP,
    G,

    // Measurements
    MX,
    MnX,
    MY,
    MnY,
    MZ,
    MnZ,

    // Preps
    PX,
    PnX,
    PY,
    PnY,
    PZ,
    PnZ,

    // Measure + Prep
    MPX,
    MPnX,
    MPY,
    MPnY,
    MPZ,
    MPnZ,

    // # Non-Cliffords
    // SQ rotations
    RX(f64),
    RY(f64),
    RZ(f64),

    U(f64, f64, f64),
    R1XY(f64, f64),

    // T gates
    T,
    Tdg,

    // TQ rotations
    RXX(f64),
    RYY(f64),
    RZZ(f64),

    // Composite TQ rotation
    RXXRYYRZZ(f64, f64, f64),

    // Custom user gate
    CustomGate,
}
