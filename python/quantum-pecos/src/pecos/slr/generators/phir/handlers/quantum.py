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

from pecos.slr.generators.phir.handlers.base import BaseHandler, HandlerRegistry
from pecos.slr.vars import QReg


class SingleQubitGateHandler(BaseHandler):
    """
    Handles single-qubit gate operations and processes their data.

    This class is designed to handle single-qubit gate operations (`op`) within
    a specific context. It processes the operation's symbolic representation,
    parameters (if any), qubit arguments, and measurement returns (if applicable).
    The method `handle` returns a dictionary containing the processed gate data.
    """

    def handle(self, op, context):
        """
        Handles the operation and context to produce gate data in a specific format.

        This function processes an operation and its related context to extract relevant
        information, such as parameters, qubit arguments, and return values. The extracted
        data is structured into a dictionary format that can be interpreted by the calling
        system. If the operation includes measured qubits or custom parameters, those are
        appropriately handled and incorporated into the output.

        Args:
            op: The operation object to be processed. May include parameters, symbolic
                names, and qubits.
            context: The context object containing helper attributes such as
                id_formatter, which aids in converting elements like qubits and bits
                to unique identifiers.

        Returns:
            A dictionary containing the processed gate data, including attributes like
            operation name (symbolic), parameters, qubit arguments, and optionally, return
            values in case of measurement operations.
        """
        gate_data = {"qop": op.sym}
        if hasattr(op, "params") and op.params:
            gate_data["angles"] = [[float(p) for p in op.params], "rad"]
        gate_data["args"] = [
            context.id_formatter.qubit_to_id(q) for q in op.qargs if hasattr(q, "reg")
        ]
        if op.sym == "Measure" and hasattr(op, "cout"):
            gate_data["returns"] = [context.id_formatter.bit_to_id(c) for c in op.cout]
        return gate_data


class TwoQubitGateHandler(BaseHandler):
    """
    Handles two-qubit gate operations and processes their corresponding data.

    This class is designed to facilitate the handling of operations involving
    two-qubit gates. It extracts and formats the gate parameters, qubit arguments,
    and other related information to a structure suitable for further processing
    or serialization in the given context.

    Methods
    -------
    handle(op: Operation, context: Context) -> dict
        Processes a two-qubit gate operation and its associated data.
    """

    def handle(self, op, context):
        """
        Handles the processing of a quantum operation object by creating a data
        structure containing the operation's symbolic representation, parameters
        (if any), and arguments based on the qubit identifiers in the specified
        context.

        Parameters
        ----------
        op : Operation
            An operation object that contains symbolic representation,
            parameters, and qubit arguments.
        context : Context
            Formatting context providing utility functions such as
            conversion of qubits to unique identifiers.

        Returns
        -------
        dict
            A dictionary containing the following keys:
            - "qop": Symbolic representation of the operation (str).
            - "angles": If operation has parameters, a list containing their
              float values and the unit "rad" (list).
            - "args": A nested list representing the qubits involved in the
              operation as their formatted identifiers.

        Raises
        ------
        TypeError
            Raised if the 'qargs' attribute of the input operation does not
            contain tuples of two qubits when tuples are expected.
        """
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
                gate_data["args"].append(
                    [
                        context.id_formatter.qubit_to_id(q1),
                        context.id_formatter.qubit_to_id(q2),
                    ],
                )
            else:
                msg = f"Expected two-qubit tuple arguments, got: {op.qargs}"
                raise TypeError(msg)
        return gate_data


class BarrierHandler(BaseHandler):
    """
    Handles operations of type "Barrier".

    The BarrierHandler processes "barrier" operations on quantum registers (qregs)
    and converts their corresponding qubit identifiers into a format suitable
    for the given context.

    Attributes
    ----------
    inheritance hierarchy: BaseHandler
        The class inherits from BaseHandler and is utilized in the operation
        handling system of the framework.
    """

    def handle(self, op, context):
        qubit_ids = []
        for q in op.qregs:
            if isinstance(q, QReg):
                qubit_ids.extend(
                    context.id_formatter.qubit_to_id(q[i]) for i in range(q.size)
                )
            else:
                qubit_ids.append(context.id_formatter.qubit_to_id(q))
        return {"meta": "barrier", "args": qubit_ids}


# Register handlers
HandlerRegistry.register("SingleQubitGate", SingleQubitGateHandler())
HandlerRegistry.register("TwoQubitGate", TwoQubitGateHandler())
HandlerRegistry.register("Barrier", BarrierHandler())
