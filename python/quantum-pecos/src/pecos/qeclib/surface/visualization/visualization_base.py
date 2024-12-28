from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from pecos.qeclib.surface.protocols import SurfacePatch


@dataclass
class VisData:
    """Container for visualization data"""

    nodes: list[tuple[int, int]]
    polygons: list[list[tuple[int, int]]]
    polygon_colors: dict[int, int]


class VisualizationStrategy(Protocol):
    """Protocol for different visualization approaches"""

    def get_visualization_data(self, patch: "SurfacePatch") -> VisData: ...
    def supports_view(self, view_type: str) -> bool: ...


class Visualizable(Protocol):
    """Basic trait for anything visualizable."""

    def get_visualization_data(self) -> VisData: ...
    def support_view(self, view_type: str) -> bool: ...
