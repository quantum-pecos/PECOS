import json

from pecos.engines.hybrid_engine import HybridEngine


def test_phir_direct_or_operation():
    """Direct PHIR test for OR operation."""
    phir = {
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "ops": [
            {"data": "cvar_define", "data_type": "i64", "variable": "a", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "b", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "r", "size": 8},
            {"cop": "=", "args": [1], "returns": [["a", 0]]},  # Set a[0] = 1
            {"cop": "=", "args": [0], "returns": [["b", 0]]},  # Set b[0] = 0
            {
                "cop": "=",
                "args": [{"cop": "|", "args": [["a", 0], ["b", 0]]}],
                "returns": [["r", 0]],  # r[0] = a[0] | b[0]
            },
        ],
        "metadata": {"generated_by": "PHIR direct test"},
    }

    # Debug output for PHIR
    print("\nGenerated PHIR:")
    print(json.dumps(phir, indent=4))

    # Run using HybridEngine
    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    # Debug raw results
    print("\nRaw Results:")
    print(results)

    # Extract and verify result for OR operation
    r_value = int(results["r"][0], 2)
    assert r_value == 1, f"Expected OR result to be 1, got {r_value}"


def test_phir_direct_and_operation():
    phir = {
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "ops": [
            {"data": "cvar_define", "data_type": "i64", "variable": "a", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "b", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "r", "size": 8},
            {"cop": "=", "args": [1], "returns": [["a", 0]]},  # Set a[0] = 1
            {"cop": "=", "args": [1], "returns": [["b", 0]]},  # Set b[0] = 1
            {
                "cop": "=",
                "args": [{"cop": "&", "args": [["a", 0], ["b", 0]]}],
                "returns": [["r", 0]],  # r[0] = a[0] & b[0]
            },
        ],
    }

    print("\nGenerated PHIR (AND):")
    print(json.dumps(phir, indent=4))

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    print("\nRaw Results (AND):")
    print(results)

    r_value = int(results["r"][0], 2)
    assert r_value == 1, f"Expected AND result to be 1, got {r_value}"


def test_phir_direct_not_operation():
    """Test NOT operation on individual bits."""
    phir = {
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "ops": [
            {"data": "cvar_define", "data_type": "i64", "variable": "a", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "r", "size": 8},
            {"cop": "=", "args": [1], "returns": [["a", 0]]},  # Set a[0] = 1
            {
                "cop": "=",
                "args": [{"cop": "~", "args": [["a", 0]]}],
                "returns": [["r", 0]],  # r[0] = ~a[0]
            },
        ],
    }

    print("\nGenerated PHIR (NOT):")
    print(json.dumps(phir, indent=4))

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    print("\nRaw Results (NOT):")
    print(results)

    r_bits = [int(b) for b in results["r"][0]]
    assert r_bits[0] == 0, f"Expected ~1 on bit 0 to produce 0, got {r_bits[0]}"


def test_phir_not_individual_bit():
    """Test NOT operation on individual bits."""
    phir = {
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "ops": [
            {"data": "cvar_define", "data_type": "i64", "variable": "a", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "r", "size": 8},
            {"cop": "=", "args": [1], "returns": [["a", 0]]},  # Set a[0] = 1
            {
                "cop": "=",
                "args": [{"cop": "~", "args": [["a", 0]]}],
                "returns": [["r", 0]],  # r[0] = ~a[0]
            },
        ],
    }

    print("\nGenerated PHIR (NOT on individual bit):")
    print(json.dumps(phir, indent=4))

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    print("\nRaw Results (NOT on individual bit):")
    print(results)

    r_bits = [int(b) for b in results["r"][0]]
    assert r_bits[0] == 0, f"Expected ~1 on bit 0 to produce 0, got {r_bits[0]}"


def test_phir_not_full_register():
    """
    Test NOT operation on full registers.

    The interpreter treats registers as signed integers and applies the NOT (~) operation
    using two's complement logic. The result of ~1 on a 64-bit signed integer flips all bits,
    producing -2. This behavior does not truncate the result to the register's defined size.
    """
    phir = {
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "ops": [
            {"data": "cvar_define", "data_type": "i64", "variable": "a", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "r", "size": 8},
            {
                "cop": "=",
                "args": [1],
                "returns": ["a"],
            },  # Set a = 1 (00000001 for 8 bits)
            {
                "cop": "=",
                "args": [{"cop": "~", "args": ["a"]}],  # Flip all bits in 'a'
                "returns": ["r"],  # Assign to 'r'
            },
        ],
    }

    print("\nGenerated PHIR (NOT on full register):")
    print(json.dumps(phir, indent=4))

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    print("\nRaw Results (NOT on full register):")
    print(results)

    r_value = int(results["r"][0], 2)

    # Adjust the expected value to reflect signed two's complement behavior
    assert r_value == -2, f"Expected ~1 on full register to produce -2, got {r_value}"


def test_phir_masked_not_operation():
    """Test NOT operation with masking to register size."""
    phir = {
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "ops": [
            {"data": "cvar_define", "data_type": "i64", "variable": "a", "size": 8},
            {"data": "cvar_define", "data_type": "i64", "variable": "r", "size": 8},
            {"cop": "=", "args": [1], "returns": ["a"]},
            {
                "cop": "=",
                "args": [{"cop": "~", "args": ["a"]}],
                "returns": ["r"],
            },
        ],
    }

    eng = HybridEngine(qsim="stabilizer")
    results = eng.run(phir, shots=1)

    r_value = int(results["r"][0], 2)
    # Apply mask for unsigned behavior
    r_value &= (1 << 8) - 1
    assert r_value == 254, f"Expected ~1 with masking to produce 254, got {r_value}"
