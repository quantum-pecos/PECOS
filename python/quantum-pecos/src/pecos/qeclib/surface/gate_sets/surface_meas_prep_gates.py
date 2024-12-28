from pecos.qeclib.surface.macrolibs.preps.project_pauli import PrepProjectZ
from pecos.qeclib.surface.patches.patch_base import SurfacePatch


class SurfaceMeasPrepGates:

    @staticmethod
    def pz(*patches: SurfacePatch) -> list[PrepProjectZ]:
        return [PrepProjectZ(p.data) for p in patches]

    @staticmethod
    def mz(patches, outputs):
        """Destructively measure in the Z basis."""
        # TODO: ...
