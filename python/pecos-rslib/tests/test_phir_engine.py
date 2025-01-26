# pecos-rslib/tests/test_phir_engine.py
import json
import pytest
from pecos_rslib._pecos_rslib import PHIREngine

# This is a minimal PHIR program that should work
MINIMAL_PHIR = {
    "format": "PHIR/JSON",
    "version": "0.1.0",
    "metadata": {"generated_by": "Test"},
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

def test_phir_minimal():
    """Test with a minimal PHIR program to verify basic functionality."""
    phir_json = json.dumps(MINIMAL_PHIR)
    try:
        import pecos.classical_interpreters
        print("\nDebug: Found PECOS module")
        # Version info removed as it's not exposed
    except ImportError as e:
        print(f"\nDebug: Failed to import PECOS: {e}")

    try:
        engine = PHIREngine(phir_json)
        print("\nDebug: Successfully created PHIREngine")
    except Exception as e:
        print(f"\nDebug: Failed to create PHIREngine: {e}")
        raise

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

def test_phir_invalid_json():
    invalid_json = '{"format": "PHIR/JSON", "invalid": }'
    with pytest.raises(Exception):
        PHIREngine(invalid_json)