from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from pecos.qeclib.surface.enums import SurfacePatchOrientation
    from pecos.qeclib.surface.layouts.layout_base import Layout
    from pecos.qeclib.surface.visualization.visualization_base import VisData
    from pecos.slr import Qubit

# TODO: Create a vector or an array of objects...
# TODO: Set check scheduling for syn_extract
# TODO: deal with rot surface code having 4 orientations (X up, Z up + mirror... left, right)


class SurfacePatch(Protocol):
    """A general surface patch.

    The patch has two code distances: dx and dz, corresponding to the X and Z logical
    operators respectively. The overall code distance (the minimum weight of any logical
    operator) is the minimum of dx and dz.
    """

    name: str
    dx: int  # Distance of the X logical operator
    dz: int  # Distance of the Z logical operator
    orientation: SurfacePatchOrientation
    data: list[Qubit]
    stab_gens: list[tuple[str, tuple[int, ...]]]
    layout: Layout

    @property
    def distance(self) -> int:
        """The code distance (minimum weight of any logical operator)."""
        return min(self.dx, self.dz)

    def validate(self) -> None:
        """Raises an exception if invalid configuration"""

    def get_visualization_data(self) -> VisData: ...

    @classmethod
    def default(cls) -> SurfacePatch:
        """Constructor for common settings."""
