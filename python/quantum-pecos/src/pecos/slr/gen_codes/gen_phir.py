from __future__ import annotations

from typing import TYPE_CHECKING, Any

from pecos import __version__
from pecos.slr.vars import CReg, QReg

if TYPE_CHECKING:
    from pecos.slr.vars import Var


class PHIRGenerator:
    """Generator for converting SLR to PHIR format."""

    def __init__(self, add_versions=True):
        self.output: dict[str, Any] = {
            "format": "PHIR/JSON",
            "version": "0.1.0",
            "ops": [],
        }
        self.current_scope = None
        self.add_versions = add_versions
        if self.add_versions:
            self.output["metadata"] = {
                "generated_by": f"PECOS version {__version__}",
            }

    def generate_block(self, block):
        """Generate PHIR for a block."""
        previous_scope = self.enter_block(block)

        block_name = type(block).__name__

        if block_name == "If":
            if_block = {
                "block": "if",
                "condition": self.generate_op(block.cond),
                "true_branch": [],
            }

            # Generate operations for the true branch
            for op in block.ops:
                if hasattr(op, "ops"):
                    self.generate_block(op)
                else:
                    phir_op = self.generate_op(op)
                    if phir_op:
                        if_block["true_branch"].append(phir_op)

            self.output["ops"].append(if_block)

        elif block_name == "Repeat":
            # Handle repeat blocks by unrolling them
            for _ in range(block.cond):
                for op in block.ops:
                    if hasattr(op, "ops"):
                        self.generate_block(op)
                    else:
                        phir_op = self.generate_op(op)
                        if phir_op:
                            self.output["ops"].append(phir_op)
        else:
            # Handle regular blocks
            for op in block.ops:
                if hasattr(op, "ops"):
                    self.generate_block(op)
                else:
                    phir_op = self.generate_op(op)
                    if phir_op:
                        self.output["ops"].append(phir_op)

        self.exit_block(block)
        self.current_scope = previous_scope

    def enter_block(self, block) -> Any | None:
        """Enter a new block scope."""
        previous_scope = self.current_scope
        self.current_scope = block

        block_name = type(block).__name__

        if block_name == "Main":
            # Handle variable definitions first
            for var in block.vars:
                var_def = self.process_var_def(var)
                self.output["ops"].append(var_def)

            for op in block.ops:
                op_name = type(op).__name__
                if op_name == "Vars":
                    for var in op.vars:
                        var_def = self.process_var_def(var)
                        self.output["ops"].append(var_def)

        return previous_scope

    def exit_block(self, block):
        """Exit the current block scope."""

    def process_var_def(self, var: Var) -> dict[str, Any]:
        """Process variable definitions."""
        var_type = type(var).__name__
        if var_type == "QReg":
            return {
                "data": "qvar_define",
                "data_type": "qubits",
                "variable": var.sym,
                "size": var.size,
            }
        elif var_type == "CReg":
            return {
                "data": "cvar_define",
                "data_type": "i64",
                "variable": var.sym,
                "size": var.size,
            }
        else:
            msg = f"Unsupported variable type: {var_type}"
            raise TypeError(msg)

    def generate_op(self, op) -> dict[str, Any] | None:
        """Generate PHIR for an operation."""
        op_name = type(op).__name__

        if op_name == "Barrier":
            return {
                "meta": "barrier",
                "args": [
                    str(q) if isinstance(q, (QReg, CReg)) else self._qubit_to_id(q)
                    for q in op.qregs
                ],
            }

        elif op_name == "Comment":
            return {"//": op.txt}

        elif op_name == "Permute":
            # Handle permutations as comments for now
            return {"//": f"Permutation: {op}"}

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
        ]:
            return self._process_classical_op(op)

        elif op_name in ["NEG", "NOT"]:
            return self._process_unary_op(op)

        elif hasattr(op, "is_qgate") and op.is_qgate:
            return self._process_quantum_gate(op)

        return None

    def _process_classical_op(self, op) -> dict[str, Any]:
        """Process classical operations."""
        op_name = type(op).__name__
        if op_name == "SET":
            return {
                "cop": "=",
                "args": [self._process_classical_expr(op.right)],
                "returns": [self._process_classical_expr(op.left)],
            }
        else:
            return {
                "cop": op.symbol,
                "args": [
                    self._process_classical_expr(op.left),
                    self._process_classical_expr(op.right),
                ],
            }

    def _process_unary_op(self, op) -> dict[str, Any]:
        """Process unary operations."""
        return {
            "cop": op.symbol,
            "args": [self._process_classical_expr(op.value)],
        }

    def _process_quantum_gate(self, op) -> dict[str, Any]:
        """Process quantum gates."""
        gate_data = {
            "qop": op.sym,
        }

        # Handle gate parameters if present
        if op.params:
            gate_data["angles"] = [[float(p) for p in op.params], "rad"]

        # Process qubit arguments
        if op.qsize == 1:
            gate_data["args"] = [self._qubit_to_id(q) for q in op.qargs]
        else:
            gate_data["args"] = [
                [self._qubit_to_id(q1), self._qubit_to_id(q2)] for q1, q2 in op.qargs
            ]

        # Handle measurement returns
        if op.sym == "Measure" and hasattr(op, "cout"):
            gate_data["returns"] = [self._bit_to_id(c) for c in op.cout]

        return gate_data

    def _qubit_to_id(self, qubit) -> list[str]:
        """Convert a qubit reference to PHIR qubit ID format."""
        if hasattr(qubit, "reg"):
            return [qubit.reg.sym, qubit.index]
        return [qubit.sym, 0]  # For single qubit registers

    def _bit_to_id(self, bit) -> list[str]:
        """Convert a classical bit reference to PHIR bit ID format."""
        if hasattr(bit, "reg"):
            return [bit.reg.sym, bit.index]
        return [bit.sym, 0]  # For single bit registers

    def _process_classical_expr(
        self, expr,
    ) -> int | str | list[str] | dict[str, Any]:
        """Process classical expressions."""
        if isinstance(expr, (int, str)):
            return expr
        elif hasattr(expr, "reg"):
            return [expr.reg.sym, expr.index]
        elif hasattr(expr, "sym"):
            return expr.sym
        elif hasattr(expr, "symbol"):
            return {
                "cop": expr.symbol,
                "args": [
                    self._process_classical_expr(expr.left),
                    self._process_classical_expr(expr.right),
                ],
            }
        msg = f"Unsupported classical expression type: {type(expr)}"
        raise TypeError(msg)

    def get_output(self) -> dict[str, Any]:
        """Get the complete PHIR output."""
        return self.output
