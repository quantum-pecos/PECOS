from __future__ import annotations

from typing import Protocol

from pecos.qeclib.surface.patches.surface_4444_rot_patch import Surface4444RotPatch


class SurfacePatch(Protocol):

    @staticmethod
    def new(distance: int | tuple[int]) -> SurfacePatch:
        """Create a new surface patch."""
        return Surface4444RotPatch(distance)
