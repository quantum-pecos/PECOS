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

from pecos.qeclib import qubit
from pecos.slr import BitSlice, Block, Comment, QubitSlice


class NoFlagMeasureX(Block):
    def __init__(self, d: QubitSlice[7], a: QubitSlice[1], out: BitSlice[1]):
        super().__init__()

        self.extend(
            Comment("Measure logical X with no flagging"),
            qubit.Prep(a[0]),
            qubit.H(a[0]),
            qubit.CX(
                (d[0], a[0]),
                (d[1], a[0]),
                (d[2], a[0]),
            ),
            qubit.H(a[0]),
            qubit.Measure(a[0]) > out[0],
        )
