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
This module provides the base abstractions for operation handlers in PHIR generation.

Classes:
- **BaseHandler**: Abstract base class for all operation handlers.
- **HandlerRegistry**: A registry mapping operation types to their corresponding handlers.

Examples:
-----------
# Example usage of HandlerRegistry
class ExampleHandler(BaseHandler):
    def handle(self, op, context):
        return {"example": op}

HandlerRegistry.register("ExampleOp", ExampleHandler())
handler = HandlerRegistry.get_handlers()["ExampleOp"]
output = handler.handle("sample_op", None)
print(output)  # Output: {"example": "sample_op"}

"""

from abc import ABC, abstractmethod
from typing import Any, ClassVar

from pecos.slr.generators.phir.context import GeneratorContext


class BaseHandler(ABC):
    """Abstract base class for all operation handlers."""

    @abstractmethod
    def handle(self, op: Any, context: GeneratorContext) -> dict[str, Any]:
        """Process an operation and return a PHIR-compatible dictionary.

        Args:
            op: The operation to process.
            context: The current generator context.

        Returns:
            A dictionary representing the PHIR-compatible output.
        """
        msg = "Handlers must implement the 'handle' method."
        raise NotImplementedError(msg)


class HandlerRegistry:
    """Registry to map operation types to handlers."""

    _handlers: ClassVar[dict[str, BaseHandler]] = {}

    @classmethod
    def register(cls, op_type: str, handler: BaseHandler):
        """Register a handler for a specific operation type.

        Args:
            op_type: The operation type as a string.
            handler: The handler instance.
        """
        cls._handlers[op_type] = handler

    @classmethod
    def get_handlers(cls):
        """Retrieve all registered handlers.

        Returns:
            A dictionary mapping operation types to handler instances.
        """
        if not cls._handlers:
            from . import quantum, classical, block, misc  # Lazy import # noqa

            cls._setup_default_handlers()
        return cls._handlers.copy()

    @classmethod
    def _setup_default_handlers(cls):
        """Initialize default handlers. This method is a placeholder."""
