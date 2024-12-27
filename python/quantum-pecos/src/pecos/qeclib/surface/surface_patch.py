from __future__ import annotations

from enum import Enum
from typing import TYPE_CHECKING, Protocol, TypeVar

from pecos.qeclib.surface.visualization import Visualizable
from pecos.slr import QReg, Vars

if TYPE_CHECKING:
    from pecos.slr import Qubit

Self = TypeVar("Self")

# TODO: Create a vector or an array of objects...
# TODO: Consider if we should use the Result pattern or not.
# TODO: Set check scheduling


class LatticeType(Enum):
    """Lattices that the patches of the surface code can be constructed from.

    References:
        1. Jonas Anderson, "Fault-tolerance in two-dimensional topological systems" by
           <https://digitalrepository.unm.edu/phyc_etds/4/>
    """

    SQUARE = (4, 4, 4, 4)
    RHOMBITRIHEXAGONAL = (3, 4, 6, 4)
    TRIHEXAGONAL = (3, 6, 3, 6)


class SurfacePatchOrientation(Enum):
    X_TOP_BOTTOM = 0
    Z_TOP_BOTTOM = 1


class SurfacePatch(Protocol):
    """A general surface patch.

    The patch has two code distances: dx and dz, corresponding to the X and Z logical
    operators respectively. The overall code distance (the minimum weight of any logical
    operator) is the minimum of dx and dz.
    """

    name: str
    dx: int  # Distance of the X logical operator
    dz: int  # Distance of the Z logical operator
    orientation: SurfacePatchOrientation
    data: list[Qubit]

    @property
    def distance(self) -> int:
        """The code distance (minimum weight of any logical operator)."""
        return min(self.dx, self.dz)

    def validate(self) -> None:
        """Raises an exception if invalid configuration"""

    @staticmethod
    def new(distance: int, name: str | None = None) -> SurfacePatch:
        """Simple constructor for a square rotated surface-patch on a 4.4.4.4 lattice"""
        if distance < 1:
            msg = "Distance must be at least 1."
            raise TypeError(msg)

        return RotatedSurfacePatch(
            dx=distance,
            dz=distance,
            orientation=SurfacePatchOrientation.X_TOP_BOTTOM,
            name=name,
        )

    @staticmethod
    def builder() -> SurfacePatchBuilder:
        return SurfacePatchBuilder()


class BaseSurfacePatch(SurfacePatch, Visualizable, Vars):
    """Base implementation with shared code"""

    def __init__(
        self,
        dx: int,
        dz: int,
        orientation: SurfacePatchOrientation,
        name: str | None = None,
    ):
        super().__init__()
        self.dx = dx
        self.dz = dz
        self.orientation = orientation

        self.name = f"{type(self).__name__}_{id(self)}"
        if name is not None:
            self.name = f"{self.name}_{name}"

        # Validate before creating resources
        self.validate()

    def validate(self) -> None:
        """Shared validation logic"""
        if self.dx < 1:
            msg = "X distance must be at least 1"
            raise TypeError(msg)
        if self.dz < 1:
            msg = "Z distance must be at least 1"
            raise TypeError(msg)
        if not isinstance(self.orientation, SurfacePatchOrientation):
            msg = "Invalid orientation type"
            raise TypeError(msg)

    @staticmethod
    def supports_view(view_type) -> bool:
        """Determine whether this surface patch supports viewing."""
        match view_type:
            case "lattice":
                return True
            case _:
                return False


class RotatedSurfacePatch(BaseSurfacePatch):
    """Rotated surface patch."""

    def __init__(
        self,
        dx: int,
        dz: int,
        orientation: SurfacePatchOrientation,
        name: str | None = None,
    ):
        super().__init__(dx, dz, orientation, name)

        n = self.dx * self.dz
        self.data_reg = QReg(f"{self.name}_data", n)
        self.data = [self.data_reg[i] for i in range(n)]  # TODO: this might be janky...

        self.vars = [
            self.data_reg,
        ]

        self.x_checks: list[tuple[int, ...]] = []
        self.z_checks: list[tuple[int, ...]] = []

    def get_check_lists(self):
        """Get a list of checks represented as tuples of data qubit ids (`list[tuple[int, ...]]`)."""
        # TODO: do...


class NonRotatedSurfacePatch(BaseSurfacePatch):
    """Standard surface patch."""

    def __init__(
        self,
        dx: int,
        dz: int,
        orientation: SurfacePatchOrientation,
        name: str | None = None,
    ):
        super().__init__(dx, dz, orientation, name)

        n = self.dx * self.dz  # TODO: fix for non-rotated surface code
        self.data_reg = QReg(f"{self.name}_data", n)
        self.data = [self.data_reg[i] for i in range(n)]  # TODO: this might be janky...

        self.vars = [
            self.data_reg,
        ]


class SurfacePatchBuilder:
    """Build for complex patch configurations."""

    def __init__(self) -> None:
        self.name: str | None = None
        self.dx: int | None = None
        self.dz: int | None = None
        self.is_rotated: bool = True
        self.lattice_type = LatticeType.SQUARE
        self.orientation = SurfacePatchOrientation.X_TOP_BOTTOM

    def set_name(self, name: str) -> Self:
        self.name = name
        return self

    def with_distances(self, dx: int, dz: int) -> Self:
        """Set the X and Z code distances (where the overall distance of the code is the minimum of the two)."""
        self.dx = dx
        self.dz = dz
        return self

    def not_rotated(self) -> Self:
        self.is_rotated = False
        return self

    def with_orientation(
        self,
        orientation: SurfacePatchOrientation,
    ) -> SurfacePatchBuilder:
        self.orientation = orientation
        return self

    def with_lattice(self, lattice: LatticeType) -> Self:
        self.lattice_type = lattice
        return self

    def build(self) -> SurfacePatch:
        # Validate configuration
        if self.dx is None or self.dz is None:
            msg = "Must specify distance(s)"
            raise TypeError(msg)

        if self.lattice_type != LatticeType.SQUARE:
            msg = "Currently only Lattice type SQUARE is supported"
            raise NotImplementedError(msg)

        if self.dx < 1 or self.dz < 1:
            msg = "The x and z distances must be at least 1."
            raise TypeError(msg)

        if self.is_rotated:
            return RotatedSurfacePatch(
                name=self.name,
                dx=self.dx,
                dz=self.dz,
                orientation=self.orientation,
            )
        else:
            return NonRotatedSurfacePatch(
                name=self.name,
                dx=self.dx,
                dz=self.dz,
                orientation=self.orientation,
            )
