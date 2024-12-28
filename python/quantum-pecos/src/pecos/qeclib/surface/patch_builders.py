from typing import TYPE_CHECKING, TypeVar

from pecos.qeclib.surface.layouts.layout_base import LatticeType
from pecos.qeclib.surface.patches.patch_base import SurfacePatchOrientation
from pecos.qeclib.surface.patches.surface_patches import (
    NonRotatedSurfacePatch,
    RotatedSurfacePatch,
)

if TYPE_CHECKING:
    from pecos.qeclib.surface.patches.patch_base import SurfacePatch

Self = TypeVar("Self")


class SurfacePatchBuilder:
    """Build for complex patch configurations."""

    def __init__(self) -> None:
        self.name: str | None = None
        self.dx: int | None = None
        self.dz: int | None = None
        self.is_rotated: bool = True
        self.lattice_type = LatticeType.SQUARE
        self.orientation = SurfacePatchOrientation.X_TOP_BOTTOM

    def set_name(self, name: str) -> Self:
        self.name = name
        return self

    def with_distances(self, dx: int, dz: int) -> Self:
        """Set the X and Z code distances (where the overall distance of the code is the minimum of the two)."""
        self.dx = dx
        self.dz = dz
        return self

    def not_rotated(self) -> Self:
        self.is_rotated = False
        return self

    def with_orientation(
        self,
        orientation: SurfacePatchOrientation,
    ) -> Self:
        self.orientation = orientation
        return self

    def with_lattice(self, lattice: LatticeType) -> Self:
        self.lattice_type = lattice
        return self

    def build(self) -> "SurfacePatch":
        # Validate configuration
        if self.dx is None or self.dz is None:
            msg = "Must specify distance(s)"
            raise TypeError(msg)

        if self.lattice_type != LatticeType.SQUARE:
            msg = "Currently only Lattice type SQUARE is supported"
            raise NotImplementedError(msg)

        if self.dx < 1 or self.dz < 1:
            msg = "The x and z distances must be at least 1."
            raise TypeError(msg)

        if self.is_rotated:
            return RotatedSurfacePatch(
                name=self.name,
                dx=self.dx,
                dz=self.dz,
                orientation=self.orientation,
            )
        else:
            return NonRotatedSurfacePatch(
                name=self.name,
                dx=self.dx,
                dz=self.dz,
                orientation=self.orientation,
            )
