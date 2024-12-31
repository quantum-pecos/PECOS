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
This module provides utility classes and context management for generating PHIR (PECOS High-level Intermediate
Representation).

Classes:
- **IdFormatter**: Formats quantum and classical register elements into PHIR-compatible identifiers.
- **GeneratorContext**: Maintains state and configuration for PHIR generation.

Examples:
-----------
# Example usage of IdFormatter
qubit = Qubit(QReg("q", 2), 0)  # Assume Qubit and QReg are defined elsewhere
formatter = IdFormatter()
id = formatter.qubit_to_id(qubit)
print(id)  # Output: ["q", 0]

# Example usage of GeneratorContext
context = GeneratorContext(add_versions=True)
print(context.version)  # Output: "0.1.0"

"""

from dataclasses import dataclass, field


@dataclass
class IdFormatter:
    """Formats quantum and classical register elements into PHIR-compatible identifiers."""

    @staticmethod
    def qubit_to_id(qubit) -> list[str]:
        """Convert a qubit to a PHIR-compatible identifier.

        Args:
            qubit: The qubit object to format.

        Returns:
            A list representing the qubit identifier (e.g., ["q", 0]).
        """
        return [qubit.reg.sym, qubit.index] if hasattr(qubit, "reg") else [qubit.sym, 0]

    @staticmethod
    def bit_to_id(bit) -> list[str]:
        """Convert a bit to a PHIR-compatible identifier.

        Args:
            bit: The bit object to format.

        Returns:
            A list representing the bit identifier (e.g., ["c", 0]).
        """
        return [bit.reg.sym, bit.index] if hasattr(bit, "reg") else [bit.sym, 0]


@dataclass
class GeneratorContext:
    """
    Represents the context for a code generator, including configuration options
    and utilities.

    This class provides settings and utilities required for generator
    operations, e.g., formatting and version handling. It facilitates
    adjustable configurations for generator behavior through its attributes.

    Attributes:
        add_versions: bool
            Determines whether versions should be added to the generated
            output.
        id_formatter: IdFormatter
            An instance of IdFormatter responsible for formatting identifiers.

    Properties:
        version: str
            Provides the current version of PHIR, represented as a string.
    """

    add_versions: bool = True
    id_formatter: IdFormatter = field(default_factory=IdFormatter)

    @property
    def version(self) -> str:
        """Return the current PHIR version."""
        return "0.1.0"
