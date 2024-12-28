from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from pecos.qeclib.surface.patches.patch_base import SurfacePatch


@dataclass
class VisData:
    """Container for visualization data"""

    nodes: list[tuple[int, int]]
    polygons: list[list[tuple[int, int]]]
    polygon_colors: dict[int, int]
    plot_cups: bool


class VisualizationStrategy(Protocol):
    """Protocol for different visualization approaches"""

    def get_visualization_data(self, patch: SurfacePatch) -> VisData: ...
    def supports_view(self, view_type: str) -> bool: ...


@dataclass
class BaseVisConfig:
    """Base configuration for visualization"""

    figsize: tuple[int, int] | None = None
    colors: list[str] | None = None
