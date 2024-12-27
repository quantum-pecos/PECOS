from pecos.qeclib.surface.patches.surface_4444_rot_patch import Surface4444RotPatch


class SurfaceBareSynGates:

    @staticmethod
    def syn_extr(*patches: Surface4444RotPatch, rounds=1):
        """Measure `rounds` number of syndrome extraction of X and Z checks using bare ancillas."""
        # TODO: ...

    @staticmethod
    def qec(*patches: Surface4444RotPatch):
        """Run distance number of rounds of syndrome extraction."""
        return [SurfaceBareSynGates.syn_extr(rounds=p.distance) for p in patches]
