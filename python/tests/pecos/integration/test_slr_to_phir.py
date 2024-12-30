import pytest
from pecos.engines.hybrid_engine import HybridEngine
from pecos.qeclib import qubit as Q  # noqa: N812
from pecos.slr import CReg, If, Main, QReg
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

    # Validate PHIR structure
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

    # Validate PHIR structure
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

    # Validate PHIR structure
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

    # Validate PHIR structure
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

    # Validate PHIR structure
    model = PHIRModel.model_validate(phir)
    qvar_defs = [op for op in model.ops if getattr(op, "data", None) == "qvar_define"]
    cvar_defs = [op for op in model.ops if getattr(op, "data", None) == "cvar_define"]

    assert len(qvar_defs) == 1
    assert qvar_defs[0].size == 3
    assert len(cvar_defs) == 2
    assert any(op.size == 32 for op in cvar_defs)
    assert any(op.size == 1 for op in cvar_defs)


def test_error_cases():
    """Test error handling."""

    def test_error_cases():
        """Test error handling."""
        # Combine into single statement
        with pytest.raises(TypeError):
            Main(q := QReg("q", -1)).gen(PHIRGenerator())  # Invalid size

        # Combine into single statement
        with pytest.raises((ValueError, AttributeError)):  # Accept either error type
            Main(q := QReg("q", 3), Q.CCX(q[0], q[1], q[2])).gen(
                PHIRGenerator(),
            )  # 3-qubit gate not implemented
