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


def register_phir():
    """
    Registers a new generator for "phir" in the CodeGenRegistry.

    This function uses the CodeGenRegistry to associate the "phir" key with an instance of `PHIRGenerator`. The
    association enables  retrieval and usage of the registered code generator within the context of PECOS's code
    generation functionalities.
    """

    from pecos.slr.codegen import CodeGenRegistry

    CodeGenRegistry.register("phir", PHIRGenerator())
