from pecos.qeclib.surface.macrolibs.preps.project_pauli import PrepProjectZ
from pecos.qeclib.surface.patches.surface_4444_rot_patch import Surface4444RotPatch


class SurfaceMeasPrepGates:

    @staticmethod
    def pz(*patches: Surface4444RotPatch) -> list[PrepProjectZ]:
        return [PrepProjectZ(p.data) for p in patches]

    @staticmethod
    def mz(patches, outputs):
        """Destructively measure in the Z basis."""
        # TODO: ...
