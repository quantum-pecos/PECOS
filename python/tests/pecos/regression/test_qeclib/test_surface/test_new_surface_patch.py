from pecos.qeclib.surface import SurfacePatch
from pecos.qeclib.surface.surface_patch import (
    LatticeType,
    NonRotatedSurfacePatch,
    RotatedSurfacePatch,
    SurfacePatchOrientation,
)
from pecos.slr import Main


def test_new_surface_patch():

    prog = Main(
        s := SurfacePatch.new(3),
    )
    assert isinstance(s, RotatedSurfacePatch)
    prog.qasm()


def test_new_surface_patch_name():

    prog = Main(
        s := SurfacePatch.new(3, "s"),
    )
    assert isinstance(s, RotatedSurfacePatch)
    prog.qasm()


def test_build_surface_patch():
    prog = Main(
        s := (
            SurfacePatch.builder()
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
