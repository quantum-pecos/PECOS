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

from pecos.slr import QReg, Vars


# TODO: see if something more elegant... then using Vars...
class Surface4444RotPatch(Vars):
    """Rotated Surface code rectangular patch on 4.4.4.4 lattice."""

    def __init__(
        self,
        distance: int | tuple[int, int],
        name: str | None = None,
    ) -> None:
        # TODO: figure out the minimum of what a surface code will need to track...

        super().__init__()
        self.name = str(type(self).__name__)
        if name is not None:
            self.name = f"{self.name}_{name}"

        if isinstance(distance, int):
            self.dx = distance
            self.dz = distance
        else:
            self.dx, self.dz = distance
        self.distance = min(self.dx, self.dz)

        self.default_orientation = (
            True  # True: logical X runs North-South; False: logical X runs West-East
        )

        n = self.dx * self.dz
        self.data_reg = QReg(f"{self.name}_{id(self)}", n)
        self.data = [self.data_reg[i] for i in range(n)]  # TODO: this might be janky...

        self.vars = [
            self.data_reg,
        ]

    @staticmethod
    def new(distance: int | tuple[int, int]) -> Surface4444RotPatch:
        """Creates a new instance of `Surface4444RotPatch`."""
        return Surface4444RotPatch(distance)

    @staticmethod
    def new_vec(
        distance: int | tuple[int, int],
        num: int,
    ) -> list[Surface4444RotPatch]:
        """Create a collection of surface code patches."""
        return [Surface4444RotPatch(distance) for _ in range(num)]
