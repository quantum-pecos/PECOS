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

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Protocol

from pecos import __version__
from pecos.slr.vars import QReg

if TYPE_CHECKING:
    from pecos.slr.vars import Qubit, Var


class OperationHandler(Protocol):
    """Protocol for operation handlers."""

    def __call__(self, op: Any) -> dict[str, Any]: ...


@dataclass
class PHIRGenerator:
    """Generates PHIR (PECOS High-level Intermediate Representation) from SLR."""

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
        """Initialize version metadata if needed."""
        if self.add_versions:
            self.output["metadata"] = {"generated_by": f"PECOS version {__version__}"}

    # Register Definition Methods
    @staticmethod
    def process_var_def(var: Var) -> dict[str, Any]:
        """Process variable definitions with validation."""
        if var.size <= 0:
            msg = f"Register size must be positive, got {var.size}"
            raise TypeError(msg)

        register_types = {
            "QReg": lambda v: {
                "data": "qvar_define",
                "data_type": "qubits",
                "variable": v.sym,
                "size": v.size,
            },
            "CReg": lambda v: {
                "data": "cvar_define",
                "data_type": "i64",
                "variable": v.sym,
                "size": v.size,
            },
        }

        var_type = type(var).__name__
        if var_type not in register_types:
            msg = f"Unsupported variable type: {var_type}"
            raise TypeError(msg)

        return register_types[var_type](var)

    # Block Processing Methods
    def _process_block_ops(self, block, is_true_branch=False) -> list[dict[str, Any]]:
        """Process operations within a block."""
        ops = []
        for op in block.ops:
            if hasattr(op, "ops"):
                if type(op).__name__ == "If":
                    ops.append(self._process_if_block(op))
                else:
                    inner_phir = PHIRGenerator(add_versions=False)
                    inner_phir.generate_block(op)
                    if inner_phir.output["ops"]:
                        ops.append(
                            {"block": "sequence", "ops": inner_phir.output["ops"]},
                        )
            else:
                phir_op = self.generate_op(op)
                if phir_op:
                    ops.append(phir_op)
        return ops

    def _process_if_block(self, block) -> dict[str, Any]:
        """Process if blocks with conditions."""
        return {
            "block": "if",
            "condition": self._process_classical_expr(block.cond),
            "true_branch": self._process_block_ops(block, is_true_branch=True),
        }

    # Operation Processing Methods
    def generate_op(self, op) -> dict[str, Any] | None:
        """Generate PHIR for different operation types."""
        handlers: dict[str, OperationHandler] = {
            "Barrier": self._generate_barrier,
            "Comment": lambda x: {"//": x.txt},
            "Permute": lambda x: {"//": f"Permutation: {x}"},
            "If": self._process_if_block,
        }

        op_name = type(op).__name__
        if op_name in handlers:
            return handlers[op_name](op)
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
            return self._process_classical_expr(op)
        elif hasattr(op, "is_qgate") and op.is_qgate:
            return self._process_qgate(op)

        return None

    # Classical Expression Processing
    def _process_classical_expr(self, expr) -> int | str | list[str] | dict[str, Any]:
        """Process classical expressions into PHIR tree structure."""
        if isinstance(expr, (int, str)):
            return expr
        elif hasattr(expr, "reg") and hasattr(expr, "index"):
            return [expr.reg.sym, expr.index]
        elif hasattr(expr, "sym"):
            return expr.sym
        elif hasattr(expr, "symbol"):
            return self._process_operation_expr(expr)

        msg = f"Unsupported classical expression type: {type(expr)}"
        raise TypeError(msg)

    def _process_operation_expr(self, expr) -> dict[str, Any]:
        """Process operation expressions (SET, unary, binary)."""
        if type(expr).__name__ == "SET":
            return {
                "cop": "=",
                "args": [self._process_classical_expr(expr.right)],
                "returns": [self._process_classical_expr(expr.left)],
            }
        elif hasattr(expr, "value"):  # Unary operation
            return {
                "cop": expr.symbol,
                "args": [self._process_classical_expr(expr.value)],
            }
        else:  # Binary operation
            return {
                "cop": expr.symbol,
                "args": [
                    self._process_classical_expr(expr.left),
                    self._process_classical_expr(expr.right),
                ],
            }

    # ID Conversion Methods
    @staticmethod
    def _qubit_to_id(qubit: Qubit | QReg) -> list[str]:
        """Convert qubit reference to PHIR ID format."""
        return [qubit.reg.sym, qubit.index] if hasattr(qubit, "reg") else [qubit.sym, 0]

    @staticmethod
    def _bit_to_id(bit) -> list[str]:
        """Convert classical bit reference to PHIR ID format."""
        return [bit.reg.sym, bit.index] if hasattr(bit, "reg") else [bit.sym, 0]

    # Quantum Gate Processing Methods
    def _process_qgate(self, op) -> dict[str, Any]:
        """Process quantum gates based on size."""
        if op.qsize > 2:
            msg = f"Gates with more than 2 qubits not supported. Got gate with {op.qsize} qubits"
            raise ValueError(
                msg,
            )

        return self._process_tq_gate(op) if op.qsize == 2 else self._process_sq_gate(op)

    def _process_sq_gate(self, op) -> dict[str, Any]:
        """Process single qubit gates."""
        gate_data = {"qop": op.sym}

        if hasattr(op, "params") and op.params:
            gate_data["angles"] = [[float(p) for p in op.params], "rad"]

        gate_data["args"] = [
            self._qubit_to_id(q) for q in op.qargs if hasattr(q, "reg")
        ]

        if op.sym == "Measure" and hasattr(op, "cout"):
            gate_data["returns"] = [self._bit_to_id(c) for c in op.cout]

        return gate_data

    def _process_tq_gate(self, op) -> dict[str, Any]:
        """Process two qubit gates."""
        gate_data = {"qop": op.sym}

        if op.params:
            gate_data["angles"] = [[float(p) for p in op.params], "rad"]

        qargs = (
            [(op.qargs[0], op.qargs[1])]
            if not isinstance(op.qargs[0], tuple)
            else op.qargs
        )

        gate_data["args"] = []
        for q in qargs:
            if isinstance(q, tuple):
                q1, q2 = q
                gate_data["args"].append([self._qubit_to_id(q1), self._qubit_to_id(q2)])
            else:
                msg = f"For two-qubit gate, expected args to be a collection of size two tuples! Got: {op.qargs}"
                raise TypeError(
                    msg,
                )

        return gate_data

    def _generate_barrier(self, op) -> dict[str, Any]:
        """Generate PHIR for barrier operations."""
        qubit_ids = []
        for q in op.qregs:
            if isinstance(q, QReg):
                qubit_ids.extend(self._qubit_to_id(q[i]) for i in range(q.size))
            else:
                qubit_ids.append(self._qubit_to_id(q))
        return {
            "meta": "barrier",
            "args": qubit_ids,
        }

    # Main Block Methods
    def enter_block(self, block) -> Any:
        """Enter a new block scope and process variables."""
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
        """Generate PHIR for a block."""
        previous_scope = self.enter_block(block)
        block_name = type(block).__name__

        if block_name == "If":
            self.output["ops"].append(self._process_if_block(block))
        elif block_name == "Repeat":
            for _ in range(block.cond):
                self.output["ops"].extend(self._process_block_ops(block))
        else:
            self.output["ops"].extend(self._process_block_ops(block))

        self.current_scope = previous_scope

    def get_output(self) -> dict[str, Any]:
        """Get the complete PHIR output."""
        return self.output
