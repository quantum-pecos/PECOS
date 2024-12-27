from pecos.qeclib.surface.surface_patch import SurfacePatch


class SurfaceBareSynGates:

    @staticmethod
    def syn_extr(*patches: SurfacePatch, rounds=1):
        """Measure `rounds` number of syndrome extraction of X and Z checks using bare ancillas."""
        # TODO: ...

    @staticmethod
    def qec(*patches: SurfacePatch):
        """Run distance number of rounds of syndrome extraction."""
        return [SurfaceBareSynGates.syn_extr(rounds=p.distance) for p in patches]
