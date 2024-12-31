# Copyright 2024 The PECOS Developers
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
# the License.You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
# specific language governing permissions and limitations under the License.

"""
This module provides functionality for converting SLR (Simple Logical Representation), a front-end DSL, into PHIR
(PECOS High-level Intermediate Representation). PHIR is designed for realistic quantum-classical noisy simulations,
encapsulating hybrid program constructs in a lightweight JSON format.

Classes and Key Functions:
- **PHIRGenerator**: Core class for generating PHIR representations from SLR constructs.
- **HandlerRegistry**: Registry for mapping operation types to their corresponding handlers.
- **BaseHandler**: Abstract base class for all handlers.
- **ClassicalExprHandler**: Handles the conversion of classical expressions to PHIR.
- **SingleQubitGateHandler** and **TwoQubitGateHandler**: Manage quantum operations for single and two-qubit gates.
- **BarrierHandler**: Handles barrier operations in PHIR.

General Workflow:
1. The `PHIRGenerator` is instantiated, optionally including version metadata.
2. SLR constructs (e.g., `Block`, `If`, `Main`) are passed to the generator.
3. Handlers registered in the `HandlerRegistry` convert these constructs into PHIR-compatible JSON dictionaries.
4. The final output is a complete PHIR program, ready for simulation.

Key Design Considerations:
- **Extensibility**: New operations can be added by implementing a handler and registering it in the `HandlerRegistry`.
- **Modularity**: Transformation logic is encapsulated in handlers, keeping the generator lightweight and focused.
- **Error Handling**: The module raises clear exceptions for unsupported features or incorrect usage.

Examples:
-----------
# Basic Example
from pecos.slr.generators.phir import PHIRGenerator
from pecos.slr import Main, QReg, CReg, If
from pecos.qeclib import qubit as Q

def example_program():
    prog = Main(
        q := QReg("q", 2),
        c := CReg("c", 2),
        If(c[0] == 1).Then(
            Q.X(q[1]),
        )
    )
    phir_gen = PHIRGenerator()
    phir_output = prog.gen(phir_gen)
    print(phir_output)

example_program()

For further information, see the PHIR specification or individual class docstrings.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from pecos import __version__
from pecos.slr.generators.phir.context import GeneratorContext
from pecos.slr.generators.phir.handlers.base import HandlerRegistry


@dataclass
class PHIRGenerator:
    """
    PHIRGenerator is responsible for converting programmatic constructs and operations into
    the PHIR (PECOS High-level Intermediate Representation) format. This format aims
    to standardize and serialize the representations of various programming constructs.

    The generator supports handling of blocks, variables (e.g., quantum or classical registers),
    and operations into a JSON-compliant dictionary structure. It incorporates mechanisms to
    process nested blocks and operations, and its functionalities are extendable via handlers
    from a registry. This class offers tools for systematically transforming an input program
    representation into PHIR and is designed for use in program compilation or transformation
    pipelines.

    Attributes:
        add_versions: A boolean indicating whether to include metadata such as generator version.
        current_scope: The block or context currently being processed (None if not set).
        output: A dictionary representing the accumulated PHIR output, including format
            specifications, metadata, and generated operations.
    """

    add_versions: bool = True
    current_scope: Any | None = None
    output: dict[str, Any] = field(
        default_factory=lambda: {
            "format": "PHIR/JSON",
            "version": "0.1.0",
            "ops": [],
        },
    )

    def __post_init__(self):
        """Initialize the generator and its context."""
        self.context = GeneratorContext(add_versions=self.add_versions)
        self._init_handlers()
        if self.add_versions:
            self.output["metadata"] = {"generated_by": f"PECOS version {__version__}"}

    def _init_handlers(self):
        """Initialize all handlers from the registry."""
        self.handlers = HandlerRegistry.get_handlers()

    @staticmethod
    def process_var_def(var) -> dict[str, Any]:
        """Process variable definitions into PHIR format.

        Args:
            var: The variable to process (e.g., QReg, CReg).

        Returns:
            A dictionary representing the variable in PHIR format.

        Raises:
            TypeError: If the variable type is unsupported or has invalid size.
        """
        if var.size <= 0:
            msg = f"Register size must be positive, got {var.size}"
            raise TypeError(msg)

        var_type = type(var).__name__
        register_types = {
            "QReg": {"data": "qvar_define", "data_type": "qubits"},
            "CReg": {"data": "cvar_define", "data_type": "i64"},
        }

        if var_type not in register_types:
            msg = f"Unsupported variable type: {var_type}"
            raise TypeError(msg)

        definition = register_types[var_type].copy()
        definition["variable"] = var.sym
        definition["size"] = var.size
        return definition

    @staticmethod
    def process_block_ops(block, context) -> list[dict[str, Any]]:
        """Process operations within a block into PHIR format.

        Args:
            block: The block containing operations.
            context: The generator context.

        Returns:
            A list of dictionaries representing operations in PHIR format.

        Raises:
            TypeError: For unsupported operation types.
        """
        ops = []  # List to store processed operations
        phir = PHIRGenerator(
            add_versions=False,
        )  # Temporary generator for nested processing

        handlers = HandlerRegistry.get_handlers()
        for op in block.ops:
            # Check if operation has nested blocks
            if hasattr(op, "ops"):
                if type(op).__name__ == "If":
                    ops.append(handlers["Block"].handle(op, context))
                else:
                    # Recursively process nested blocks
                    phir.generate_block(op)
                    if phir.output["ops"]:
                        ops.append({"block": "sequence", "ops": phir.output["ops"]})
            else:
                op_name = type(op).__name__
                # Handle specific operations like Comment or Barrier
                if op_name in ["Comment", "Barrier"]:
                    ops.append(handlers[op_name].handle(op, context))
                else:
                    # Process remaining operations
                    phir_op = phir.handle_op(op)
                    if phir_op:
                        ops.append(phir_op)
        return ops

    def handle_op(self, op) -> dict[str, Any] | None:
        """Handle an individual operation using registered handlers.

        Args:
            op: The operation to process.

        Returns:
            A dictionary representing the operation in PHIR format, or None if unsupported.
        """
        op_name = type(op).__name__

        # Route based on operation type
        if hasattr(op, "is_qgate") and op.is_qgate:
            handler_key = "SingleQubitGate" if op.qsize == 1 else "TwoQubitGate"
            return self.handlers[handler_key].handle(op, self.context)
        elif op_name in [
            "SET",
            "EQUIV",
            "NEQUIV",
            "LT",
            "GT",
            "LE",
            "GE",
            "MUL",
            "DIV",
            "XOR",
            "AND",
            "OR",
            "PLUS",
            "MINUS",
            "RSHIFT",
            "LSHIFT",
            "NEG",
            "NOT",
        ]:
            return self.handlers["ClassicalExpr"].handle(op, self.context)
        elif op_name in self.handlers:
            return self.handlers[op_name].handle(op, self.context)
        return None

    def enter_block(self, block) -> Any:
        """Enter a new block scope and process its variables.

        Args:
            block: The block to enter.

        Returns:
            The previous scope before entering the new block.
        """
        previous_scope = self.current_scope
        self.current_scope = block

        if type(block).__name__ == "Main":
            for var in block.vars:
                self.output["ops"].append(self.process_var_def(var))
            for op in block.ops:
                if type(op).__name__ == "Vars":
                    for var in op.vars:
                        self.output["ops"].append(self.process_var_def(var))

        return previous_scope

    def generate_block(self, block):
        """Generate PHIR for a given block.

        Args:
            block: The block to process.
        """
        previous_scope = self.enter_block(block)
        block_name = type(block).__name__

        if block_name == "If":
            self.output["ops"].append(
                self.handlers["Block"].handle(block, self.context),
            )
        elif block_name == "Repeat":
            for _ in range(block.cond):
                self.output["ops"].extend(self.process_block_ops(block, self.context))
        else:
            self.output["ops"].extend(self.process_block_ops(block, self.context))

        self.current_scope = previous_scope

    def get_output(self) -> dict[str, Any]:
        """Retrieve the final PHIR representation.

        Returns:
            The generated PHIR as a dictionary.
        """
        return self.output

    def generate(self, block) -> dict[str, Any]:
        """Main entry point for PHIR generation.

        Args:
            block: The block to process.

        Returns:
            The generated PHIR as a dictionary.
        """
        self.generate_block(block)
        return self.get_output()
