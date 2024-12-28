from enum import Enum


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
