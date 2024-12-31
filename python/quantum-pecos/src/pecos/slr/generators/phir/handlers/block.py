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

from pecos.slr.generators.phir.generator import PHIRGenerator
from pecos.slr.generators.phir.handlers.base import BaseHandler, HandlerRegistry
from pecos.slr.generators.phir.handlers.classical import ClassicalExprHandler


class BlockHandler(BaseHandler):
    """
    Handles the processing of specific block types in a given context.

    This class is used to handle the transformation or processing of various
    block constructs in a structured format. It enables the extraction and
    conversion of conditional statements (if blocks) and their respective
    components such as conditions and branches. The processed data is
    structured into a dictionary format to facilitate further usage.

    Attributes:
        Inherits attributes from BaseHandler class.

    Methods:
        handle(block, context):
            Processes a block based on its type and generates a structured
            representation for specific block types. Returns None if the
            block type is not handled.
    """

    def handle(self, block, context):
        block_name = type(block).__name__
        if block_name == "If":
            return {
                "block": "if",
                "condition": ClassicalExprHandler().handle(block.cond, context),
                "true_branch": PHIRGenerator.process_block_ops(block, context),
            }
        return None


HandlerRegistry.register("Block", BlockHandler())
