from pecos.qeclib.surface import (
    Lattice2DView,
    LatticeType,
    NonRotatedSurfacePatch,
    RotatedSurfacePatch,
    SurfacePatchBuilder,
    SurfacePatchOrientation,
)
from pecos.slr import Main


def test_default_rot_surface_patch():

    prog = Main(
        s := RotatedSurfacePatch.default(3),
    )
    assert isinstance(s, RotatedSurfacePatch)
    prog.qasm()


def test_default_rot_surface_patch_name():

    prog = Main(
        s := RotatedSurfacePatch.default(3, "s"),
    )
    assert isinstance(s, RotatedSurfacePatch)
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


def test_build_rot_surface_patch():
    prog = Main(
        s := (
            SurfacePatchBuilder()
            .set_name("s")
            .with_distances(3, 5)
            .with_lattice(LatticeType.SQUARE)
            .with_orientation(SurfacePatchOrientation.Z_TOP_BOTTOM)
            .build()
        ),
    )
    assert isinstance(s, RotatedSurfacePatch)
    prog.qasm()


def test_surface_patch_builder_render():
    s = SurfacePatchBuilder().with_distances(3, 3).build()
    Lattice2DView.render(s).show()


def test_rot_surface_patch_render():
    s = RotatedSurfacePatch.default(3)
    Lattice2DView.render(s).show()
