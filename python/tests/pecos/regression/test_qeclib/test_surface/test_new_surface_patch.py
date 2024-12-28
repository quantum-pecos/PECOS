from pecos.qeclib.surface import NonRotatedSurfacePatch, SurfacePatchBuilder
from pecos.qeclib.surface.enums import (
    LatticeType,
    SurfacePatchOrientation,
)
from pecos.slr import Main


def test_new_surface_patch():

    prog = Main(
        s := NonRotatedSurfacePatch.default(3),
    )
    assert isinstance(s, NonRotatedSurfacePatch)
    prog.qasm()


def test_new_surface_patch_name():

    prog = Main(
        s := NonRotatedSurfacePatch.default(3, "s"),
    )
    assert isinstance(s, NonRotatedSurfacePatch)
    prog.qasm()


def test_build_surface_patch():
    prog = Main(
        s := (
            SurfacePatchBuilder()
            .set_name("s")
            .with_distances(3, 5)
            .with_lattice(LatticeType.SQUARE)
            .with_orientation(SurfacePatchOrientation.Z_TOP_BOTTOM)
            .not_rotated()
            .build()
        ),
    )
    assert isinstance(s, NonRotatedSurfacePatch)
    prog.qasm()
