from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pecos.qeclib.surface.patches.surface_4444_rot_patch import Surface4444RotPatch


class SurfaceTransversalGates:
    """A collection of transversal gate implementations for the `Surface4444RotPatch`."""

    @staticmethod
    def x(*patches: Surface4444RotPatch) -> Surface4444RotPatch:
        """Apply logical Pauli X on surface code patches."""
        # TODO: ...

    @staticmethod
    def y(*patches: Surface4444RotPatch):
        """Apply logical Pauli Y on surface code patches."""
        # TODO: ...

    @staticmethod
    def z(*patches: Surface4444RotPatch):
        """Apply logical Pauli Z on surface code patches."""
        # TODO: ...

    @staticmethod
    def h(*patches: Surface4444RotPatch, permute=True):
        """Apply transversal Hadamard on surface code patches followed by a Permutation/Relabeling.

        Examples:
            ```python
            from pecos.slr import Main
            from pecos.qeclib.surface import Surface4444RotPatch as SP
            s := [SP.new(3) for _ in range(4)],
            SP.h(s[0], s[3]),

        """
        # TODO: ...

    @staticmethod
    def cx(
        *patches: Surface4444RotPatch | tuple[Surface4444RotPatch, Surface4444RotPatch],
    ):
        """Apply transversal CX on surface code patches.

        Arguments:
             Can either be a tuples of two surface code patches or a sequence of surface code patches. If tuples, then
             the first is considered the control and the second the target surface code. If a sequence of surface code
             patches are supplies, it is assumed the sequence is of a control surface code followed by a target surface
             code patch sequence.

        Examples:
             ```python
             from pecos.slr import Main
             from pecos.qeclib.surface import Surface4444RotPatch as SP

             prog = Main(
                 s := [SP.new(3) for _ in range(4)],
                 SP.cx((s[0], s[2]), (s[1], s[3])),
                 SP.cx(s[0], s[2], s[1], s[3]),  # Equivalent to previous line
                 # Equivalent to the following two lines:
                 SP.cx(s[0], s[2]),
                 SP.cx(s[1], s[3]),
             )
             ```
        """
        # TODO: ...
