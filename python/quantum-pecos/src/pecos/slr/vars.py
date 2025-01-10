# Copyright 2023 The PECOS Developers
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
# the License.You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
# specific language governing permissions and limitations under the License.

from __future__ import annotations

from dataclasses import dataclass
from inspect import isclass
from typing import TYPE_CHECKING, Any, Generic, TypeVar

from typeguard import TypeCheckError, checker_lookup_functions

from pecos.slr.cops import SET, PyCOp
from pecos.slr.misc import Reorder

if TYPE_CHECKING:
    from collections.abc import Sequence

    from typeguard import TypeCheckerCallable, TypeCheckMemo

N = TypeVar("N", bound=int)

# TODO: Make it a VarDef


class Vars:
    """A collection of variables."""

    def __init__(self, *args):
        self.vars = []

    def extend(self, vars_obj: Vars):
        if isinstance(vars_obj, Vars):
            self.vars.extend(vars_obj.vars)
        else:
            msg = f"Was expecting a Vars object. Instead got type: {type(vars_obj)}"
            raise TypeError(msg)

    def append(self, op):
        if isinstance(op, Var):
            self.vars.append(op)
            return True
        else:
            return False

    def extend_vars(self, vargs):
        for v in vargs:
            if not self.append(v):
                msg = f"Unrecognized variable type: {type(v)}"
                raise TypeError(msg)

    def get(self, sym: str):
        for v in self.vars:
            if v.sym == sym:
                return v

    def __iter__(self):
        return iter(self.vars)


class Var: ...


class Reg(Var):
    """Base register class."""

    def __init__(self, sym: str, size: int, elem_type: type[Elem]) -> None:
        self.sym = sym
        self.size = size
        self.elems = []
        self.elem_type = elem_type

        for i in range(size):
            self.elems.append(self.new_elem(i))

    def new_elem(self, item):
        return self.elem_type(self, item)

    def set(self, other):
        return SET(self, other)

    def __getitem__(self, item):
        if isinstance(item, int):
            return self.elems[item]
        elif isinstance(item, (slice, list, tuple)):
            # Determine the appropriate slice type
            slice_type = QubitSlice if isinstance(self, QubitArray) else BitSlice

            # Validate slice creation for runtime type safety
            if slice_type is QubitSlice and not isinstance(self, QubitArray):
                msg = f"Cannot create QubitSlice from {type(self).__name__}"
                raise TypeError(msg)
            if slice_type is BitSlice and not isinstance(self, BitArray):
                msg = f"Cannot create BitSlice from {type(self).__name__}"
                raise TypeError(msg)

            # Return the validated slice
            return slice_type(self, item)
        else:
            msg = f"Invalid index type: {type(item)}"
            raise TypeError(msg)

    def __repr__(self):
        repr_str = self.__class__.__name__
        if self.sym is not None:
            repr_str = f"{repr_str}:{self.sym}"
        return f"<{repr_str} at {hex(id(self))}>"

    def __str__(self):
        return self.sym


class Elem(Var):
    def __init__(self, reg: Reg, idx: int):
        super().__init__()

        self.reg = reg
        self.index = idx

    def set(self, other):
        return SET(self, other)

    def __getitem__(self, item: int):
        msg = f"'{self.__class__.__name__}' object is not subscriptable"
        raise TypeError(msg)

    def __repr__(self):
        return f"<{self.__class__.__name__} {self.index} of {self.reg.sym}>"

    def __str__(self):
        return f"{self.reg.sym}[{self.index}]"


class QubitArray(Reg):
    """An array of qubits."""

    def __init__(self, sym: str, size: int) -> None:
        super().__init__(sym, size, elem_type=Qubit)

    def __len__(self):
        return self.size


class BitArray(Reg, PyCOp):
    """An array of bits."""

    def __init__(self, sym: str, size: int) -> None:
        super().__init__(sym, size, elem_type=Bit)

    def __len__(self):
        return self.size


class QReg(QubitArray):
    def __init__(self, *args, **kwargs):
        # warnings.warn(
        #     "QReg is deprecated, use QubitArray instead",
        #     DeprecationWarning,
        #     stacklevel=2
        # )
        super().__init__(*args, **kwargs)


class CReg(BitArray):
    def __init__(self, *args, **kwargs):
        # warnings.warn(
        #     "CReg is deprecated, use BitArray instead",
        #     DeprecationWarning,
        #     stacklevel=2
        # )
        super().__init__(*args, **kwargs)


class Qubit(Elem):
    """Quantum bit."""

    def __init__(self, reg: QubitArray, idx: int) -> None:
        super().__init__(reg, idx)


class Bit(Elem, PyCOp):
    """Classical bit."""

    def __init__(self, reg: BitArray, idx: int) -> None:
        super().__init__(reg, idx)


@dataclass
class RegSlice(Generic[N]):
    """Base class for register slices with optional size checking."""

    reg: Reg
    indices: slice | Sequence[int]
    size: N = None
    expected_size: int | None = None

    def __init__(self, reg: Reg, indices: slice | Sequence[int]):
        # Validate register type
        self._validate_register_type(reg)

        self.reg = reg
        self.indices = indices

        # Calculate actual size
        if isinstance(indices, slice):
            # Handle negative indices
            start = indices.start
            if start is None:
                start = 0
            elif start < 0:
                start = reg.size + start

            stop = indices.stop
            if stop is None:
                stop = reg.size
            elif stop < 0:
                stop = reg.size + stop

            step = indices.step or 1

            # Validate bounds
            if start < 0 or start > reg.size:
                msg = "Slice start index out of bounds"
                raise IndexError(msg)
            if stop < 0 or stop > reg.size:
                msg = f"Index {stop} is out of bounds for register of size {reg.size}"
                raise IndexError(msg)

            # Validate the actual size
            actual_size = len(range(start, stop, step))

            # Enforce the fixed size, if specified
            if self.size is not None and actual_size != self.size:
                msg = f"Slice must have size {self.size}, got {actual_size}"
                raise ValueError(msg)
        else:
            actual_size = len(indices)
            # Validate indices are within register bounds
            if any(i >= reg.size or i < 0 for i in indices):
                msg = f"Index out of bounds for register of size {reg.size}"
                raise IndexError(msg)

        # Validate size based on generic type parameter
        if hasattr(self.__class__, "__orig_bases__"):
            base = self.__class__.__orig_bases__[0]
            if hasattr(base, "__args__") and base.__args__[0] is not None:
                if base.__args__[0] is not N:  # Only check if not the generic N
                    expected_size = base.__args__[0]
                    if actual_size != expected_size:
                        msg = f"Slice must have size {expected_size}, got {actual_size}"
                        raise TypeError(msg)

        # Additional type-specific size validation
        if self.expected_size is not None:
            actual_size = self._calculate_size()
            if actual_size != self.expected_size:
                msg = f"Expected {type(self).__name__} of size {self.expected_size}, got {actual_size}"
                raise TypeError(msg)

    def _calculate_size(self) -> int:
        """Helper to calculate size of the slice."""
        if isinstance(self.indices, slice):
            # Handles negative and out-of-bounds indices
            start, stop, step = self.indices.indices(self.reg.size)
            return max(0, (stop - start + (step - 1)) // step)
        elif isinstance(self.indices, (list, tuple)):
            return len(self.indices)
        else:
            return 1  # Single index access

    def _normalize_index(self, idx: int | None, default: int) -> int:
        """Helper to handle None and negative indices."""
        if idx is None:
            return default
        if idx < 0:
            return self.reg.size + idx
        return idx

    def _validate_size(self, actual_size: int) -> None:
        # Additional size validation specific to slice
        if self.expected_size is not None and actual_size != self.expected_size:
            msg = f"Expected {type(self).__name__} of size {self.expected_size}, got {actual_size}"
            raise TypeError(msg)

    def _validate_register_type(self, reg: Reg) -> None:
        """Validate the type of the register based on the slice type."""
        slice_name = type(self).__name__
        if slice_name == "QubitSlice" and not isinstance(reg, QubitArray):
            msg = f"Expected QubitArray for QubitSlice, got {type(reg).__name__}"
            raise TypeError(msg)
        elif slice_name == "BitSlice" and not isinstance(reg, BitArray):
            msg = f"Expected BitArray for BitSlice, got {type(reg).__name__}"
            raise TypeError(msg)

    def _validate_array_type(self, reg: Reg, expected_type: type[Reg]) -> None:
        """Helper to validate the type of the array being sliced."""
        if not isinstance(reg, expected_type):
            msg = f"Expected {type(self).__name__}, got {type(reg).__name__}"
            raise TypeError(msg)

    @classmethod
    def __class_getitem__(cls, item):
        """Support for generic slice sizing."""
        return type(f"{cls.__name__}[{item}]", (cls,), {"expected_size": item})

    def __len__(self) -> int:
        """Return length calculation."""
        return self._calculate_size()

    def reorder(self, new_order: Sequence[int]) -> Reorder:
        """Returns a command to reorder slice elements.

        Args:
            new_order: Sequence of indices specifying new ordering

        Returns:
            Command indicating desired reordering
        """
        return Reorder(self, new_order)

    def __iter__(self):
        if isinstance(self.indices, slice):
            start = self.indices.start
            if start is None:
                start = 0
            elif start < 0:
                start = self.reg.size + start

            stop = self.indices.stop
            if stop is None:
                stop = self.reg.size
            elif stop < 0:
                stop = self.reg.size + stop

            step = self.indices.step or 1
            for i in range(start, stop, step):
                yield self.reg[i]
        else:
            for i in self.indices:
                yield self.reg[i]

    def __getitem__(self, idx):
        if isinstance(self.indices, slice):
            start = self.indices.start
            if start is None:
                start = 0
            elif start < 0:
                start = self.reg.size + start

            step = self.indices.step or 1
            real_idx = start + (step * idx)
            return self.reg[real_idx]
        else:
            return self.reg[self.indices[idx]]

    def __add__(self, other: RegSlice) -> RegSlice:
        if not isinstance(other, RegSlice):
            msg = f"Can only concatenate with another RegSlice, not {type(other)}"
            raise TypeError(msg)

        if self.reg is not other.reg:
            msg = "Can only combine slices from the same register"
            raise ValueError(msg)

        def get_indices(slc):
            if isinstance(slc, slice):
                start = slc.start
                if start is None:
                    start = 0
                elif start < 0:
                    start = self.reg.size + start

                stop = slc.stop
                if stop is None:
                    stop = self.reg.size
                elif stop < 0:
                    stop = self.reg.size + stop

                step = slc.step or 1
                return list(range(start, stop, step))
            return list(slc)

        combined_indices = get_indices(self.indices) + get_indices(other.indices)
        return type(self)(self.reg, combined_indices)


class QubitSlice(RegSlice[N]):
    """Slice type for quantum arrays."""

    expected_size: int | None = None


class BitSlice(RegSlice[N]):
    """Slice type for classical arrays."""

    expected_size: int | None = None


def check_qubit_slice(
    value: Any,
    origin_type: Any,
    args: tuple[Any, ...],
    memo: TypeCheckMemo,
) -> None:
    # Check if the value is a QubitSlice
    if not isinstance(value, QubitSlice):
        msg = f"Expected QubitSlice, got {type(value).__name__}"
        raise TypeCheckError(msg)

    # Check if the generic argument (size) matches
    if args:
        expected_size = args[0]
        actual_size = len(list(value))

        if actual_size != expected_size:
            msg = f"Expected QubitSlice of size {expected_size}, got {actual_size}"
            raise TypeCheckError(msg)


def check_slice_type_and_size(
    value: Any,
    origin_type: type,
    args: tuple[Any, ...],
    memo: Any,
) -> None:
    """Check both the slice type and size."""
    # First check the type is correct
    if issubclass(origin_type, QubitSlice):
        if not isinstance(value, QubitSlice):
            msg = f"Expected QubitSlice, got {type(value).__name__}"
            raise TypeCheckError(msg)
    elif issubclass(origin_type, BitSlice):
        if not isinstance(value, BitSlice):
            msg = f"Expected BitSlice, got {type(value).__name__}"
            raise TypeCheckError(msg)

    # Then check size
    if args and len(args) > 0:
        expected_size = args[0]
        actual_size = len(list(value))
        if actual_size != expected_size:
            msg = f"Expected {origin_type.__name__} of size {expected_size}, got {actual_size}"
            raise TypeCheckError(msg)


def slice_checker_lookup(
    origin_type: Any,
    args: tuple[Any, ...],
    extras: tuple[Any, ...],
) -> TypeCheckerCallable | None:
    """Combined lookup for slice type and size checking."""
    if isclass(origin_type) and (
        issubclass(origin_type, QubitSlice) or issubclass(origin_type, BitSlice)
    ):
        # Get size from the class name if args is empty
        if not args and "[" in origin_type.__name__:
            size = int(origin_type.__name__.split("[")[1].strip("]"))
            return lambda v, o, a, m: check_slice_type_and_size(v, o, (size,), m)
        return check_slice_type_and_size
    return None


# Just register one combined checker
checker_lookup_functions.append(slice_checker_lookup)
