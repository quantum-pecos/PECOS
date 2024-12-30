from __future__ import annotations

from typing import TYPE_CHECKING, NamedTuple, Protocol

if TYPE_CHECKING:
    from pecos.qeclib.surface.patches.patch_base import SurfacePatch


class VisualizationData(NamedTuple):
    """Container for visualization data"""

    nodes: list[tuple[int, int]]
    polygons: list[list[tuple[int, int]]]
    polygon_colors: dict[int, int]


class VisualizationStrategy(Protocol):
    """Strategy for visualizing different types of patches."""

    def get_visualization_data(self, patch: SurfacePatch) -> VisualizationData: ...
    def supports_view(self, view_type: str) -> bool: ...
