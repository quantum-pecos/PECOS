from pecos.qeclib.surface.gate_sets.surface_bare_syn_gates import (
    SurfaceBareSynGates,
)
from pecos.qeclib.surface.gate_sets.surface_meas_prep_gates import (
    SurfaceMeasPrepGates,
)
from pecos.qeclib.surface.gate_sets.surface_transversal_gates import (
    SurfaceTransversalGates,
)


class SurfaceStdGates(
    SurfaceMeasPrepGates,
    SurfaceTransversalGates,
    SurfaceBareSynGates,
):
    """Collects a standard set of gates to use with `Surface4444RotPatch"""
