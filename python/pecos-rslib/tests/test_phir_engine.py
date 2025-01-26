# PECOS/python/pecos-rslib/tests/test_phir_engine.py
import json
import pytest
from pecos_rslib._pecos_rslib import PHIREngine

def test_phir_minimal():
    """Test with a minimal PHIR program to verify basic functionality."""
    phir_json = json.dumps({
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "metadata": {"generated_by": "PECOS version 0.6.0.dev8"},
        "ops": [
            {
                "data": "qvar_define",
                "data_type": "qubits",
                "variable": "q",
                "size": 2
            },
            {
                "data": "cvar_define",
                "data_type": "i64",
                "variable": "m",
                "size": 2
            },
            {
                "qop": "Measure",
                "args": [["q", 0]],
                "returns": [["m", 0]]
            }
        ]
    })

    # Create engine
    engine = PHIREngine(phir_json)

    # Get commands
    commands = engine.process_program()
    assert len(commands) == 1, "Expected 1 quantum command"

    # Verify it's a measure gate
    cmd = commands[0]
    assert cmd["gate_type"] == "Measure", "Expected Measure gate"
    assert cmd["params"]["result_id"] == 0, "Expected result_id to be 0"
    assert cmd["qubits"][0] == 0, "Expected measurement on qubit 0"

    # Handle measurement
    engine.handle_measurement(1)  # Send a measurement result of 1

    # Get results
    results = engine.get_results()

    # Extract the measurement key
    assert len(results) > 0, "Expected at least one measurement result"
    measurement_key = next(iter(results.keys()))

    # Verify the result
    assert results[measurement_key] == 0, f"Expected {measurement_key} to have value 0"

def test_phir_invalid_json():
    invalid_json = '{"format": "PHIR/JSON", "invalid": }'
    with pytest.raises(Exception):
        PHIREngine(invalid_json)


def test_phir_empty_program():
    phir = json.dumps({
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "metadata": {"generated_by": "Test"},
        "ops": []
    })

    engine = PHIREngine(phir)
    commands = engine.process_program()
    assert len(commands) == 0, "Expected empty command list"


def test_phir_full_circuit():
    phir = json.dumps({
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "metadata": {"generated_by": "PECOS version 0.6.0.dev8"},
        "ops": [
            {"data": "qvar_define", "data_type": "qubits", "variable": "q", "size": 2},
            {"data": "cvar_define", "data_type": "i64", "variable": "c", "size": 2},
            {"qop": "Measure", "args": [["q", 0]], "returns": [["c", 0]]},
            {"qop": "Measure", "args": [["q", 1]], "returns": [["c", 1]]},
        ]
    })

    # Create engine
    engine = PHIREngine(phir)

    # Process the program and get commands
    commands = engine.process_program()
    print(f"Got {len(commands)} commands")

    # Handle example measurements
    engine.handle_measurement(1)

    # Get final results
    results = engine.get_results()
    print(f"Got results: {results}")

    assert len(results) > 0, "Expected measurement results"


def test_phir_full():
    """Test with a full PHIR program."""
    phir = {
        "format": "PHIR/JSON",
        "version": "0.1.0",
        "metadata": {"generated_by": "PECOS version 0.6.0.dev8"},
        "ops": [
            {
                "data": "qvar_define",
                "data_type": "qubits",
                "variable": "q",
                "size": 2
            },
            {
                "data": "cvar_define",
                "data_type": "i64",
                "variable": "m",
                "size": 2
            },
            {
                "qop": "Measure",
                "args": [["q", 0]],
                "returns": [["m", 0]]
            }
        ]
    }

    phir_json = json.dumps(phir)
    engine = PHIREngine(phir_json)
    results = engine.results_dict
    assert isinstance(results, dict)