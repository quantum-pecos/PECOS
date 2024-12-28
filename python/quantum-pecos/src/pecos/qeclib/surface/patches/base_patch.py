from __future__ import annotations

from typing import TYPE_CHECKING

from pecos.qeclib.surface.enums import SurfacePatchOrientation
from pecos.qeclib.surface.visualization.rotated_lattice import (
    RotatedLatticeVisualization,
)
from pecos.slr import QReg, Vars

if TYPE_CHECKING:
    from pecos.qeclib.surface.visualization.visualization_base import (
        VisData,
        VisualizationStrategy,
    )
from pecos.qeclib.surface.protocols import SurfacePatch


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
