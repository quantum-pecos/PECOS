from dataclasses import dataclass
from typing import Protocol


@dataclass
class VisData:
    """Container for visualization data"""


class Visualizable(Protocol):
    """Basic trait for anything visualizable."""

    def get_visualizable(self) -> VisData: ...
    def support_view(self, view_type: str) -> bool: ...


class Lattice2DView:
    @staticmethod
    def render(patch):
        """Render a figure of a 2D layout of data qubits and an abstracted notion of the lattice it belongs to."""
