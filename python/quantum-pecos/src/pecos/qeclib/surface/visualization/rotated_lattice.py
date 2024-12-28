from typing import TYPE_CHECKING

from pecos.qeclib.surface.visualization.visualization_base import VisData

if TYPE_CHECKING:
    from pecos.qeclib.surface.protocols import SurfacePatch


class RotatedLatticeVisualization:
    @staticmethod
    def get_visualization_data(patch: "SurfacePatch") -> VisData:
        return patch.layout.get_visualization_elements(
            patch.dx,
            patch.dz,
            patch.stab_gens,
        )

    @staticmethod
    def supports_view(view_type: str) -> bool:
        return view_type == "lattice"
