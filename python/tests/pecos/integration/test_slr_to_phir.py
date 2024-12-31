import pytest
from pecos.engines.hybrid_engine import HybridEngine
from pecos.qeclib import qubit as Q  # noqa: N812
from pecos.slr import Barrier, Comment, CReg, If, Main, QReg
from pecos.slr.gen_codes.gen_phir import PHIRGenerator
from phir.model import PHIRModel


def test_basic_operations():
    """Test basic single-qubit operations and measurements."""
    prog = Main(
        q := QReg("q", 1),
        c := CReg("c", 1),
        Q.X(q[0]),
        Q.Measure(q[0]) > c[0],
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    assert len(model.ops) == 4  # 2 var defs + X + Measure

    # Validate simulation results
    eng = HybridEngine()
    result = eng.run(phir, shots=100)
    # After X gate, should always measure 1
    assert all(r == "1" for r in result["c"])


def test_two_qubit_operations():
    """Test two-qubit operations (Bell state preparation)."""
    prog = Main(
        q := QReg("q", 2),
        c := CReg("c", 2),
        Q.H(q[0]),
        Q.CX(q[0], q[1]),
        Q.Measure(q[0]) > c[0],
        Q.Measure(q[1]) > c[1],
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    assert len(model.ops) == 6  # 2 var defs + H + CX + 2 Measures

    # Validate simulation results
    eng = HybridEngine()
    results = eng.run(phir, shots=1000)
    counts = {"00": 0, "11": 0}
    for r in results["c"]:
        counts[r] += 1
    # Should see roughly equal 00 and 11 states
    assert 400 < counts["00"] < 600
    assert 400 < counts["11"] < 600


def test_classical_control():
    """Test classical register operations and control."""
    prog = Main(
        q := QReg("q", 2),
        c := CReg("c", 2),
        _d := CReg("d", 1),
        Q.H(q[0]),
        Q.Measure(q[0]) > c[0],
        If(c[0] == 1).Then(
            Q.X(q[1]),
        ),
        Q.Measure(q[1]) > c[1],
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    assert any(hasattr(op, "condition") for op in model.ops)  # Should have an if block

    # Validate simulation results
    eng = HybridEngine()
    results = eng.run(phir, shots=1000)
    # Should never see '10' since q[1] is only flipped when q[0] is 1
    assert all(r != "10" for r in results["c"])


def test_rotation_gates():
    """Test rotation gates with angles."""
    prog = Main(
        q := QReg("q", 1),
        c := CReg("c", 1),
        Q.RX[3.14159](q[0]),  # π rotation ≈ X gate
        Q.Measure(q[0]) > c[0],
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    rotation_ops = [op for op in model.ops if hasattr(op, "angles")]
    assert len(rotation_ops) == 1
    # Check angles format: [[values], unit]
    assert len(rotation_ops[0].angles) == 2
    assert len(rotation_ops[0].angles[0]) == 1  # One angle value
    assert rotation_ops[0].angles[1] in ("rad", "pi")  # Valid unit

    # Try state-vector first, fall back to other simulators
    try:
        from pecos.simulators.projectq.state import ProjectQSim

        if ProjectQSim is not None:
            eng = HybridEngine(qsim="state-vector")
        else:
            pytest.skip("ProjectQSim not available, skipping simulation part of test")
    except ImportError:
        # Try other simulators that support rotation gates
        simulators = ["Qulacs", "QuEST", "CuStateVec", "MPS"]
        for sim in simulators:
            try:
                from pecos.simulators import str_to_sim

                if str_to_sim[sim] is not None:
                    eng = HybridEngine(qsim=sim)
                    break
            except (ImportError, KeyError):
                continue
        else:
            pytest.skip("No suitable simulator available for rotation gates")

    results = eng.run(phir, shots=100)
    # π rotation should give mostly 1s with some numerical precision effects
    ones = sum(1 for r in results["c"] if r == "1")
    assert ones > 95


def test_variable_definitions():
    """Test various register definitions."""
    prog = Main(
        _q := QReg("q", 3),
        _c := CReg("c", 32),
        _d := CReg("d", 1),
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    qvar_defs = [op for op in model.ops if getattr(op, "data", None) == "qvar_define"]
    cvar_defs = [op for op in model.ops if getattr(op, "data", None) == "cvar_define"]

    assert len(qvar_defs) == 1
    assert qvar_defs[0].size == 3
    assert len(cvar_defs) == 2
    assert any(op.size == 32 for op in cvar_defs)
    assert any(op.size == 1 for op in cvar_defs)


def test_complex_classical_control():
    """Test more complex classical control structures."""
    prog = Main(
        q := QReg("q", 2),
        c := CReg("c", 2),
        r := CReg("r", 1),
        # Initial operations
        Q.H(q[0]),
        Q.Measure(q[0]) > c[0],
        # Nested control
        If(c[0] == 1).Then(
            Q.H(q[1]),
            Q.Measure(q[1]) > c[1],
            If(c[1] == 0).Then(
                Q.X(q[0]),
            ),
        ),
        # Classical arithmetic
        r[0].set(c[0] & c[1]),  # Classical AND
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    if_blocks = [op for op in model.ops if getattr(op, "block", None) == "if"]
    assert len(if_blocks) >= 1  # Should have at least one if block
    classical_ops = [op for op in model.ops if hasattr(op, "cop")]
    assert len(classical_ops) >= 1  # Should have classical operations


def test_barrier_operations():
    """Test barrier operations."""
    prog = Main(
        q := QReg("q", 2),
        Q.H(q[0]),
        Barrier(q[0], q[1]),  # Barrier between qubits
        Q.CX(q[0], q[1]),
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    barriers = [op for op in model.ops if getattr(op, "meta", None) == "barrier"]
    assert len(barriers) == 1
    assert len(barriers[0].args) == 2  # Should reference both qubits


def test_comments():
    """Test comment handling in PHIR output."""
    prog = Main(
        q := QReg("q", 1),
        c := CReg("c", 1),
        Comment("Initialize qubit"),
        Q.H(q[0]),
        Comment("Measurement phase"),
        Q.Measure(q[0]) > c[0],
    )
    phir = prog.gen(PHIRGenerator())

    raw_comments = [op for op in phir["ops"] if "//" in op]
    assert len(raw_comments) == 2

    model = PHIRModel.model_validate(phir)

    comments = [op for op in model.ops if type(op).__name__ == "Comment"]
    assert len(comments) == 2


def test_classical_arithmetic():
    """Test various classical arithmetic and logical operations."""
    prog = Main(
        _q := QReg("q", 1),
        a := CReg("a", 8),
        b := CReg("b", 8),
        r := CReg("r", 8),
        # Test various operations
        a[0].set(1),
        b[0].set(1),
        r[0].set(a[0] & b[0]),
        r[1].set(a[0] | b[0]),
        r[2].set(a[0] ^ b[0]),
        r[3].set(~a[0]),
        r[4].set(a[0] << 2),
        r[5].set(b[0] >> 1),
    )
    phir = prog.gen(PHIRGenerator())

    # Validate PHIR structure
    model = PHIRModel.model_validate(phir)
    classical_ops = [op for op in model.ops if hasattr(op, "cop")]

    # Should have 8 operations (6 arithmetic + 2 initial sets)
    assert len(classical_ops) == 8

    # Helper function to recursively gather operation types
    def gather_op_types(op):
        types = set()
        if isinstance(op, dict):
            if "cop" in op:
                types.add(op["cop"])
                for arg in op["args"]:
                    types.update(gather_op_types(arg))
        return types

    # Verify we have different types of operations
    op_types = set()
    for op in classical_ops:
        op_types.update(gather_op_types(op))


def test_barrier_patterns():
    """Test different barrier patterns and combinations."""
    prog = Main(
        q := QReg("q", 4),
        # Different barrier patterns
        Q.H(q[0]),
        Q.H(q[1]),
        Barrier(q[0], q[1]),  # Barrier between specific qubits
        Q.CX(q[0], q[1]),
        Q.H(q[2]),
        Q.H(q[3]),
        Barrier(q[0], q[1], q[2], q[3]),  # Explicit barrier with all qubits
        Q.CX(q[2], q[3]),
        Barrier(q[0], q[1], q[2], q[3]),  # Another full barrier
    )
    phir = prog.gen(PHIRGenerator())

    # Validate PHIR structure
    model = PHIRModel.model_validate(phir)
    barriers = [op for op in model.ops if getattr(op, "meta", None) == "barrier"]
    assert len(barriers) == 3
    # Check different barrier sizes
    barrier_sizes = [len(barrier.args) for barrier in barriers]
    assert 2 in barrier_sizes  # Two-qubit barrier
    assert 4 in barrier_sizes  # Four-qubit barrier


def test_comment_variations():
    """Test different types of comments and formats."""
    multiline_comment = """First line
    Second line
    Third line"""

    prog = Main(
        q := QReg("q", 1),
        c := CReg("c", 1),
        Comment("Simple comment"),
        Comment(multiline_comment),
        Comment("Special chars: !@#$%^&*()"),
        Q.H(q[0]),
        Comment("     Indented comment"),
        Q.Measure(q[0]) > c[0],
    )
    phir = prog.gen(PHIRGenerator())

    # Validate raw PHIR structure
    raw_comments = [op for op in phir["ops"] if "//" in op]
    assert len(raw_comments) == 4

    # Check comment content
    comment_texts = [op["//"] for op in raw_comments]
    assert any("\n" in text for text in comment_texts)  # Has multiline comment
    assert any(
        text.startswith("     ") for text in comment_texts
    )  # Has indented comment


def test_register_edge_cases():
    """Test edge cases with register sizes and indices."""
    prog = Main(
        q := QReg("q", 32),  # Large quantum register
        c := CReg("c", 64),  # Large classical register
        _m := CReg("m", 1),  # Minimum size register
        # Access various indices
        Q.H(q[0]),  # First index
        Q.H(q[31]),  # Last valid index
        Q.Measure(q[0]) > c[0],
        Q.Measure(q[31]) > c[63],
    )
    phir = prog.gen(PHIRGenerator())

    # Validate PHIR structure
    model = PHIRModel.model_validate(phir)

    # Check register definitions
    qvar_defs = [op for op in model.ops if getattr(op, "data", None) == "qvar_define"]
    cvar_defs = [op for op in model.ops if getattr(op, "data", None) == "cvar_define"]

    assert any(op.size == 32 for op in qvar_defs)
    assert any(op.size == 64 for op in cvar_defs)


def has_nested_ops(op):
    """Recursively check for nested operations or block structures."""
    print(f"Checking operation: {op}")  # Debug

    if hasattr(op, "cop"):  # Dynamically check for COp
        if hasattr(op, "args"):
            for arg in op.args:
                # Detect nested COp-like objects
                if hasattr(arg, "cop"):  # Dynamically check nested COp
                    return True
                # Detect dictionary-based operations
                if isinstance(arg, dict) and "cop" in arg:
                    return True
                # Traverse lists or tuples
                if isinstance(arg, (list, tuple)):
                    if any(has_nested_ops(sub_arg) for sub_arg in arg):
                        return True
                # Recursively check nested objects with args
                if hasattr(arg, "args"):
                    if has_nested_ops(arg):
                        return True

    elif hasattr(op, "block"):  # Dynamically check for block types
        print(f"This appears to be a Block: {op}")  # Debug
        if hasattr(op, "ops"):  # Blocks contain operations
            for inner_op in op.ops:
                if has_nested_ops(inner_op):
                    print(f"Nested operation found in block: {inner_op}")  # Debug
                    return True

    return False


def test_complex_classical_expressions():
    """Test more complex combinations of classical operations."""
    prog = Main(
        a := CReg("a", 8),
        b := CReg("b", 8),
        c := CReg("c", 8),
        r := CReg("r", 8),
        # Compound operations
        r[0].set((a[0] & b[0]) | c[0]),  # AND then OR
        r[1].set(~(a[0] & b[0])),  # NOT of an AND
        r[2].set(a[0] << (b[0] & 3)),  # Shift by result of AND
        # Multiple operations in sequence
        r[3].set(a[0] & b[0]),
        r[3].set(r[3] | c[0]),  # Using previous result
        # Nested operations
        r[4].set((a[0] & (b[0] | c[0])) ^ (a[1] & b[1])),
    )
    phir = prog.gen(PHIRGenerator())

    # Validate structure
    model = PHIRModel.model_validate(phir)
    classical_ops = [op for op in model.ops if hasattr(op, "cop")]

    # Detect nested ops
    nested_ops = [op for op in classical_ops if has_nested_ops(op)]
    print("\nDetected nested ops:", nested_ops)
    assert len(nested_ops) > 0  # Ensure at least one nested operation is detected


def test_quantum_classical_mixing():
    """Test mixing quantum and classical operations."""
    prog = Main(
        q := QReg("q", 2),
        c := CReg("c", 2),
        r := CReg("r", 8),
        Q.H(q[0]),
        Q.Measure(q[0]) > c[0],
        # Classical compute affecting quantum
        If(c[0] & 1 == 1).Then(  # Bitwise operation in condition
            Q.X(q[1]),
        ),
        # Quantum result affecting classical
        Q.Measure(q[1]) > c[1],
        r[0].set(c[0] & c[1]),  # Classical operation on measurement results
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    quantum_ops = [op for op in model.ops if hasattr(op, "qop")]
    classical_ops = [op for op in model.ops if hasattr(op, "cop")]
    assert len(quantum_ops) > 0
    assert len(classical_ops) > 0


def test_classical_edge_cases():
    """Test edge cases in classical operations."""
    prog = Main(
        a := CReg("a", 64),  # Full 64-bit register
        b := CReg("b", 64),
        r := CReg("r", 64),
        # Large shifts
        r[0].set(a[0] << 63),  # Maximum left shift
        r[1].set(b[0] >> 63),  # Maximum right shift
        # Operations on register boundaries
        r[63].set(a[63] & b[63]),  # Highest bit
        r[0].set(a[0] & b[0]),  # Lowest bit
        # Chained operations
        r[2].set(((a[0] << 2) & 0xFF) >> 1),  # Multiple shifts
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    # Verify operations on boundary bits
    classical_ops = [op for op in model.ops if hasattr(op, "cop")]
    assert len(classical_ops) >= 5  # Should have at least our test operations


def test_conditional_operations():
    """Test various forms of conditional operations."""
    prog = Main(
        _q := QReg("q", 2),
        c := CReg("c", 8),
        r := CReg("r", 8),
        # Set initial values
        c[0].set(1),
        c[1].set(2),
        # Nested conditions with classical ops
        If(c[0] == 1).Then(
            r[0].set(c[0] & c[1]),
            If(r[0] == 0).Then(
                c[2].set(~c[1]),
            ),
        ),
        # Condition using bitwise operation
        If((c[0] & c[1]) != 0).Then(
            r[1].set(1),
        ),
        # Multiple conditions
        If(c[0] == 1).Then(
            If(c[1] == 2).Then(
                r[2].set(3),
            ),
        ),
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    if_blocks = [op for op in model.ops if getattr(op, "block", None) == "if"]
    nested_if = any(
        any(getattr(inner_op, "block", None) == "if" for inner_op in block.true_branch)
        for block in if_blocks
    )
    assert nested_if


def test_deterministic_classical_operations():
    """
    Test deterministic outcomes of classical operations.

    This test evaluates bitwise AND, OR, XOR, and NOT operations.
    """
    prog = Main(
        a := CReg("a", 8),
        b := CReg("b", 8),
        r := CReg("r", 8),
        a[0].set(1),
        b[0].set(0),
        r[0].set(a[0] & b[0]),
        r[1].set(a[0] | b[0]),
        r[2].set(a[0] ^ b[0]),
        r[3].set(~a[0]),
    )
    phir = prog.gen(PHIRGenerator())

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    # Extract and print bit-by-bit results
    r_bits = [int(b) for b in results["r"][0][::-1]]
    print("\nExtracted bits for r:")
    for i, bit in enumerate(r_bits):
        print(f"r[{i}] = {bit}")

    assert r_bits[0] == 0, f"Expected AND to produce 0, got {r_bits[0]}"
    assert r_bits[1] == 1, f"Expected OR to produce 1, got {r_bits[1]}"
    assert r_bits[2] == 1, f"Expected XOR to produce 1, got {r_bits[2]}"
    assert r_bits[3] == 0, f"Expected NOT to produce 0, got {r_bits[3]}"


def test_deterministic_quantum_operations():
    """Test deterministic outcomes of quantum stabilizer operations."""
    prog = Main(
        q := QReg("q", 1),
        c := CReg("c", 1),
        Q.X(q[0]),
        Q.Measure(q[0]) > c[0],
    )
    phir = prog.gen(PHIRGenerator())

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)
    assert results["c"][0] == "1"


def test_deterministic_simulation():
    """
    Test deterministic results for quantum and classical operations.

    This test evaluates Bell state preparation and classical AND/OR operations.
    """
    prog = Main(
        q := QReg("q", 2),
        c := CReg("c", 2),
        r := CReg("r", 8),
        Q.H(q[0]),
        Q.CX(q[0], q[1]),
        Q.Measure(q[0]) > c[0],
        Q.Measure(q[1]) > c[1],
        r[0].set(c[0] & c[1]),
        r[1].set(c[0] | c[1]),
    )
    phir = prog.gen(PHIRGenerator())

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    # Extract measurement and result bits
    c_bits = [int(bit) for bit in results["c"][0][::-1]]
    r_bits = [int(b) for b in results["r"][0][::-1]]

    # Validate quantum results (Bell state)
    assert c_bits in (
        [0, 0],
        [1, 1],
    ), f"Expected Bell state [0,0] or [1,1], got {c_bits}"

    # Validate classical operations based on quantum results
    if c_bits == [0, 0]:
        assert r_bits[0] == 0, f"Expected AND to produce 0, got {r_bits[0]}"
        assert r_bits[1] == 0, f"Expected OR to produce 0, got {r_bits[1]}"
    elif c_bits == [1, 1]:
        assert r_bits[0] == 1, f"Expected AND to produce 1, got {r_bits[0]}"
        assert r_bits[1] == 1, f"Expected OR to produce 1, got {r_bits[1]}"


def test_nested_binary_operations():
    """Test nested binary operations like (a & b) & c."""
    prog = Main(
        a := CReg("a", 8),
        b := CReg("b", 8),
        c := CReg("c", 8),
        r := CReg("r", 8),
        # Set initial values
        a[0].set(1),
        b[0].set(1),
        c[0].set(1),
        # Nested AND operation
        r[0].set(a[0] & b[0] & c[0]),
        # Nested mixed operations
        r[1].set((a[0] & b[0]) | c[0]),  # Explicit nesting
        r[2].set(a[0] & (b[0] | c[0])),  # Different nesting order
    )
    phir = prog.gen(PHIRGenerator())

    # Find the nested operations
    nested_ops = [op for op in phir["ops"] if "cop" in op]

    # Verify tree structure of the operations
    r0_op = next(
        op for op in nested_ops if "returns" in op and op["returns"] == [["r", 0]]
    )
    assert r0_op["cop"] == "="
    assert r0_op["args"][0]["cop"] == "&"
    assert r0_op["args"][0]["args"][0]["cop"] == "&"
    assert r0_op["args"][0]["args"][0]["args"] == [["a", 0], ["b", 0]]
    assert r0_op["args"][0]["args"][1] == ["c", 0]

    # Verify explicit nesting preserved
    r1_op = next(
        op for op in nested_ops if "returns" in op and op["returns"] == [["r", 1]]
    )
    assert r1_op["args"][0]["cop"] == "|"
    assert r1_op["args"][0]["args"][0]["cop"] == "&"

    # Verify different nesting order preserved
    r2_op = next(
        op for op in nested_ops if "returns" in op and op["returns"] == [["r", 2]]
    )
    assert r2_op["args"][0]["cop"] == "&"
    assert r2_op["args"][0]["args"][1]["cop"] == "|"


def test_single_bit_operations():
    """Test that operations on single bits only affect that bit."""
    prog = Main(
        a := CReg("a", 8),
        b := CReg("b", 8),
        r := CReg("r", 8),
        a[0].set(1),
        b[0].set(0),
        r[0].set(a[0] & b[0]),
    )
    phir = prog.gen(PHIRGenerator())

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    # Should only have bit 0 affected
    assert int(results["a"][0], 2) == 1
    assert int(results["b"][0], 2) == 0
    assert int(results["r"][0], 2) == 0  # AND of bits should be 0


def test_simple_or_operation():
    """
    Validate the behavior of a minimal OR operation in PHIR.

    This test ensures that the logical OR operation between two single-bit registers
    is correctly executed and produces the expected result in the PHIR output.
    """
    prog = Main(
        a := CReg("a", 8),
        b := CReg("b", 8),
        r := CReg("r", 8),
        a[0].set(1),
        b[0].set(0),
        r[0].set(a[0] | b[0]),
    )
    phir = prog.gen(PHIRGenerator())

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    r_bits = [int(bit) for bit in results["r"][0][::-1]]
    assert r_bits[0] == 1, f"Expected OR to produce 1, got {r_bits[0]}"


def test_simple_and_operation():
    """Minimal test for AND operation in PHIR."""
    prog = Main(
        a := CReg("a", 8),
        b := CReg("b", 8),
        r := CReg("r", 8),
        a[0].set(1),
        b[0].set(1),
        r[0].set(a[0] & b[0]),
    )
    phir = prog.gen(PHIRGenerator())

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    r_bits = [int(bit) for bit in results["r"][0][::-1]]
    assert r_bits[0] == 1, f"Expected AND to produce 1, got {r_bits[0]}"


def test_empty_registers():
    """Test operations with empty registers."""
    with pytest.raises(TypeError, match="Register size must be positive"):
        Main(
            _a := CReg("a", 0),
            _q := QReg("q", 0),
        ).gen(PHIRGenerator())


def test_maximum_register_size():
    """Test operations with maximum register size within supported limits."""
    max_size = 64  # Adjust to framework's constraints
    prog = Main(
        a := CReg("a", max_size),
        b := CReg("b", max_size),
        r := CReg("r", max_size),
        a[0].set(1),
        b[max_size - 1].set(1),
        r[max_size - 1].set(a[0] | b[max_size - 1]),  # Edge of the register
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    assert len(model.ops) > 2  # Definitions + operations


def test_uninitialized_registers():
    """Ensure using uninitialized registers behaves as expected."""
    prog = Main(
        a := CReg("a", 8),
        r := CReg("r", 8),
        r[0].set(a[0]),  # Use uninitialized register
    )
    phir = prog.gen(PHIRGenerator())

    # Check default behavior (e.g., initialized to 0)
    eng = HybridEngine()
    results = eng.run(phir, shots=1)
    assert int(results["r"][0], 2) == 0  # Default value


def test_chained_operations():
    """Test chained operations with classical and quantum registers."""
    prog = Main(
        a := CReg("a", 8),
        r := CReg("r", 8),
        a[0].set(1),
        r[0].set((a[0] << 1) & 3),  # Chain of shifts and logical AND
        r[1].set(~r[0]),  # Inversion of the result
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    assert len(model.ops) > 3  # Definitions + chained operations


def test_superposition_edge_case():
    """Test operations that create and measure a superposition state."""
    prog = Main(
        q := QReg("q", 1),
        c := CReg("c", 1),
        Q.H(q[0]),
        Q.Measure(q[0]) > c[0],
    )
    phir = prog.gen(PHIRGenerator())

    eng = HybridEngine()
    results = eng.run(phir, shots=1000)
    counts = {"0": 0, "1": 0}
    for r in results["c"]:
        counts[r] += 1

    # Check if measurements are roughly 50-50
    assert 400 < counts["0"] < 600
    assert 400 < counts["1"] < 600


def test_invalid_gate():
    """Test behavior with an unsupported gate."""
    with pytest.raises(
        AttributeError,
        match="module 'pecos.qeclib.qubit' has no attribute",
    ):
        Main(
            q := QReg("q", 3),
            Q.CCX(
                q[0],
                q[1],
                q[2],
            ),
        ).gen(PHIRGenerator())


def test_barrier_edge_case():
    """Test barriers with an empty or one-qubit register."""
    c = CReg("c", 1)
    prog = Main(
        q := QReg("q", 1),
        Q.H(q[0]),
        Barrier(q[0]),
        Q.Measure(q[0]) > c[0],
    )
    phir = prog.gen(PHIRGenerator())

    model = PHIRModel.model_validate(phir)
    barriers = [op for op in model.ops if getattr(op, "meta", None) == "barrier"]
    assert len(barriers) == 1
