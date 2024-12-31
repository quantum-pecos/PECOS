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

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, ClassVar


@dataclass
class CodeGenCapabilities:
    """
    Represents the code generation capabilities for a quantum language backend.

    This class defines the constraints and abilities of a quantum language backend related to code generation, such as
    the maximum number of qubits, maximum classical registers, conditional execution support, and a set of supported
    gates. It provides a structured way to describe the backend's properties for compatibility checking or feature
    utilization.
    """

    max_qubits: int | None = None
    max_cregs: int | None = None
    supports_conditionals: bool = True
    supported_gates: set[str] = None


class CodeGenerator(ABC):
    """
    Abstract base class for code generation.

    The class provides a blueprint for implementing code generators that translate input programs into a specific target
    format. It also includes a property for retrieving the capabilities of the generator. This is an abstract class and
    must be subclassed with implementations for the abstract methods and properties.

    Methods:
        generate: Abstract method to generate code in a target format from the provided input program.
        capabilities: Abstract property to expose the specific abilities or constraints of the code generator.
    """

    @abstractmethod
    def generate(self, program: Any, **kwargs) -> str:
        """Generate code in target format."""

    @property
    @abstractmethod
    def capabilities(self) -> CodeGenCapabilities:
        """Get generator capabilities."""


class CodeGenRegistry:
    """
    Manages the registration and retrieval of code generator classes.

    This class acts as a registry for code generator classes, allowing them to be registered with a unique name and
    retrieved later by their name. It is useful to maintain a central repository for code generator implementations and
    provides a way to dynamically handle various generator types during runtime.
    """

    _generators: ClassVar[dict[str, type[CodeGenerator]]] = {}

    @classmethod
    def register(cls, name: str, generator: CodeGenerator):
        cls._generators[name] = generator

    @classmethod
    def get(cls, name: str) -> CodeGenerator | None:
        return cls._generators.get(name)


def register_generator(format_name: str):
    """
    Function to register a generator class with the code generation registry using a specific format name. This function
    returns a decorator, which can be applied to a class to register it as the generator for the provided format.

    Args:
        format_name (str): The name of the generator format to be registered.

    Returns:
        callable: A decorator function that registers the class with the specified format name.
    """

    def decorator(cls):
        CodeGenRegistry.register(format_name, cls)
        return cls

    return decorator
