from typing import Protocol

from pecos.qeclib.surface.visualization.visualization_base import VisData


class Layout(Protocol):
    """Protocol for different layout strategies"""

    def get_stabilizers_gens(
        self,
        dx: int,
        dz: int,
    ) -> list[tuple[str, tuple[int, ...]]]: ...
    def get_data_positions(self, dx: int, dz: int) -> list[tuple[int, int]]: ...
    def validate_dimensions(self, dx: int, dz: int) -> None: ...
    def get_visualization_elements(
        self,
        dx: int,
        dz: int,
        stab_gens: list[tuple[str, tuple[int, ...]]],
    ) -> VisData: ...
