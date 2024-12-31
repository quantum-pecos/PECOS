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


class ClassicalExprHandler(BaseHandler):
    """Handle the conversion of classical operations to PHIR format.

    This class is responsible for transforming various types of classical
    operations into a PHIR-compatible format. It uses internal logic to process
    and convert a variety of input expressions, such as integers, strings, and
    other custom objects with specific attributes.

    Designed to be utilized in systems where classical expressions or operations
    need to be parsed and transformed into a standardized intermediate
    representation.
    """

    def handle(self, op, context):
        """
        Handles the given operation by processing the expression using the internal method.

        Parameters:
        op
            The operation to be handled.
        context
            The context in which the operation is being processed.

        Returns:
            The result of the `_process_expr` method.
        """
        return self._process_expr(op)

    def _process_expr(self, expr):
        """
        Processes an expression into a normalized form suitable for internal use.

        This method processes various types of input expressions, such as integers, strings,
        and objects with specific attributes, into structured representations. The function
        supports different expression types, including objects with symbolic attributes,
        registers, or index properties. If the input expression does not conform to one of
        the recognized forms, a TypeError is raised with a descriptive message.

        Args:
            expr (int | str | object): The input expression to be processed. It can be
                an integer, string, or an object with specific properties such as `reg`,
                `index`, `sym`, `symbol`, etc.

        Returns:
            Union[int, str, list, dict]: The normalized representation of the input
                expression, which varies depending on the input type and attributes.
                It may return the original input for integers and strings, a list for
                objects with register properties, or a detailed dictionary for symbolic
                expressions.

        Raises:
            TypeError: If the input expression type is unsupported or does not have the
                required attributes to be processed.
        """
        if isinstance(expr, (int, str)):
            return expr
        elif hasattr(expr, "reg") and hasattr(expr, "index"):
            return [expr.reg.sym, expr.index]
        elif hasattr(expr, "sym"):
            return expr.sym
        elif hasattr(expr, "symbol"):
            if type(expr).__name__ == "SET":
                return {
                    "cop": "=",
                    "args": [self._process_expr(expr.right)],
                    "returns": [self._process_expr(expr.left)],
                }
            elif hasattr(expr, "value"):
                return {
                    "cop": expr.symbol,
                    "args": [self._process_expr(expr.value)],
                }
            else:
                return {
                    "cop": expr.symbol,
                    "args": [
                        self._process_expr(expr.left),
                        self._process_expr(expr.right),
                    ],
                }
        msg = f"Unsupported classical expression: {type(expr)}"
        raise TypeError(msg)


HandlerRegistry.register("ClassicalExpr", ClassicalExprHandler())
