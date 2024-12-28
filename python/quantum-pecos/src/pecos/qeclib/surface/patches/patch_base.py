from __future__ import annotations

from enum import Enum
from typing import TYPE_CHECKING, Protocol

from pecos.qeclib.surface.visualization.rotated_lattice import (
    RotatedLatticeVisualization,
)
from pecos.slr import QReg, Vars

if TYPE_CHECKING:
    from pecos.qeclib.surface.layouts.layout_base import Layout
    from pecos.qeclib.surface.visualization.visualization_base import (
        VisData,
        VisualizationStrategy,
    )
    from pecos.slr import Qubit

# TODO: Create a vector or an array of objects...
# TODO: Set check scheduling for syn_extract
# TODO: deal with rot surface code having 4 orientations (X up, Z up + mirror... left, right)


class SurfacePatchOrientation(Enum):
    X_TOP_BOTTOM = 0
    Z_TOP_BOTTOM = 1


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
    def default(cls, distance: int, name: str | None = None) -> SurfacePatch:
        """Constructor for common settings."""


class BaseSurfacePatch(SurfacePatch, Vars):
    """Base implementation with shared code"""

    def __init__(
        self,
        dx: int,
        dz: int,
        orientation: SurfacePatchOrientation,
        name: str | None = None,
        visualizer: VisualizationStrategy | None = None,
    ):
        super().__init__()
        self.dx = dx
        self.dz = dz
        self.orientation = orientation

        self.name = f"{type(self).__name__}_{id(self)}"
        if name is not None:
            self.name = f"{self.name}_{name}"

        self.stab_gens: list[tuple[str, tuple[int, ...]]] = []

        # Validate before creating resources
        self.validate()

        self._initialize_data()

        self.vars = [
            self.data_reg,
        ]

        self.visualizer = visualizer or RotatedLatticeVisualization()

    @classmethod
    def default(cls, distance: int, name: str | None = None) -> SurfacePatch:
        """Create a surface patch with common settings"""
        return cls(
            dx=distance,
            dz=distance,
            orientation=SurfacePatchOrientation.X_TOP_BOTTOM,
            name=name,
        )

    def validate(self) -> None:
        """Shared validation logic"""
        if self.dx < 1:
            msg = "X distance must be at least 1"
            raise TypeError(msg)
        if self.dz < 1:
            msg = "Z distance must be at least 1"
            raise TypeError(msg)
        if not isinstance(self.orientation, SurfacePatchOrientation):
            msg = "Invalid orientation type"
            raise TypeError(msg)

    def _initialize_data(self):
        n = self._calculate_qubit_count()
        self.data_reg = QReg(f"{self.name}_data", n)
        self.data = [self.data_reg[i] for i in range(n)]

    def _calculate_qubit_count(self) -> int:
        """Hook for implementations to define qubit count"""
        raise NotImplementedError

    def get_visualization_data(self) -> VisData:
        return self.visualizer.get_visualization_data(self)

    def supports_view(self, view_type: str) -> bool:
        return self.visualizer.supports_view(view_type)
