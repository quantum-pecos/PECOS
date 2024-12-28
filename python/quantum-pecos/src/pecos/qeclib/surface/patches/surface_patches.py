from __future__ import annotations

from typing import TYPE_CHECKING

from pecos.qeclib.surface.layouts.square_lattice import SquareRotatedLayout
from pecos.qeclib.surface.patches.base_patch import BaseSurfacePatch

if TYPE_CHECKING:
    from pecos.qeclib.surface.layouts.layout_base import Layout
    from pecos.qeclib.surface.protocols import SurfacePatchOrientation


class RotatedSurfacePatch(BaseSurfacePatch):
    """Rotated surface patch."""

    def __init__(
        self,
        dx: int,
        dz: int,
        orientation: SurfacePatchOrientation,
        layout: Layout | None = None,
        name: str | None = None,
    ):
        super().__init__(dx, dz, orientation, name)

        # TODO: Should each surface patch carry this or should it be stored somewhere for reuse...
        #       or cached somehow
        if layout is None:
            layout = SquareRotatedLayout()
        self.layout = layout
        self.stab_gens = self.layout.get_stabilizers_gens(self.dx, self.dz)

    def _calculate_qubit_count(self) -> int:
        return self.dx * self.dz


class NonRotatedSurfacePatch(BaseSurfacePatch):
    """Standard surface patch."""

    def _calculate_qubit_count(self) -> int:
        # TODO: fix for non-rotated surface code
        return self.dx * self.dz
