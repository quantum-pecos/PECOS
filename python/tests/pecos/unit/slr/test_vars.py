import pytest
from pecos.slr import (
    Bit,
    BitArray,
    BitSlice,
    If,
    Qubit,
    QubitArray,
    QubitSlice,
    Reorder,
)
from typeguard import TypeCheckError, typechecked


def test_basic_slicing():
    """Tests fundamental slicing operations.

    To test the core functionality of slicing and confirm both continuous ranges and discrete indices work correctly.

    Tests:
    1. Continuous slice syntax (0:4) returns correct type and length
    2. Discrete index syntax ([0,2,4]) returns correct type and length
    """
    qarr = QubitArray("q", 10)

    slice1 = qarr[0:4]
    assert isinstance(slice1, QubitSlice)
    assert len(list(slice1)) == 4

    slice2 = qarr[[0, 2, 4]]
    assert isinstance(slice2, QubitSlice)
    assert len(list(slice2)) == 3


def test_single_element():
    """Tests that single element access preserves original behavior.

    Single element access should still return a Qubit/Bit, not a slice, maintaining backward compatibility with existing
    code.

    Tests:
    1. Single indexing a QubitArray returns a Qubit, not a QSlice
    2. Single indexing a BitArray returns a Bit, not a CSlice
    """
    qarray = QubitArray("q", 10)
    qubit = qarray[0]
    assert not isinstance(qubit, QubitSlice)
    assert isinstance(qubit, Qubit)

    barr = BitArray("c", 10)
    bit = barr[0]
    assert not isinstance(bit, BitSlice)
    assert isinstance(bit, Bit)


def test_negative_indices():
    """Tests Python's negative index support.

    We need this test because negative indices are a fundamental Python feature
    and should work the same way as standard Python slicing.

    Tests:
    1. Slice from negative index to end (-4:)
    2. Slice from start to negative index (:-2)
    3. Slice between negative indices (-6:-2)
    """
    qarr = QubitArray("q", 10)

    slice1 = qarr[-4:]
    assert len(list(slice1)) == 4

    slice2 = qarr[:-2]
    assert len(list(slice2)) == 8

    slice3 = qarr[-6:-2]
    assert len(list(slice3)) == 4


def test_step_slicing():
    """Tests slicing with step values.

    We need this test because step slicing is a standard Python feature and should
    work for both positive and negative steps.

    Tests:
    1. Positive step (0:6:2) returns correct elements
    2. Negative step (6:0:-2) returns correct elements
    """
    qarr = QubitArray("q", 10)

    slice1 = qarr[0:6:2]
    elements = list(slice1)
    assert len(elements) == 3

    slice2 = qarr[6:0:-2]
    elements = list(slice2)
    assert len(elements) == 3


def test_slice_addition():
    """Tests combining different types of slices.

    We need this test because users need to be able to combine slices to create
    more complex selections of qubits/bits.

    Tests:
    1. Continuous + continuous slice
    2. Discrete + discrete indices
    3. Continuous + discrete mixing
    """
    qarr = QubitArray("q", 10)

    s1 = qarr[0:2]
    s2 = qarr[4:6]
    combined = s1 + s2
    assert len(list(combined)) == 4

    s3 = qarr[[0, 2]]
    s4 = qarr[[4, 6]]
    combined = s3 + s4
    assert len(list(combined)) == 4

    s5 = qarr[0:2]
    s6 = qarr[[4, 6]]
    combined = s5 + s6
    assert len(list(combined)) == 4


def test_out_of_bounds():
    """Tests handling of out-of-bounds indices.

    We need this test because the system should gracefully handle invalid indices
    instead of accessing memory out of bounds.

    Tests:
    1. Continuous slice beyond register size
    2. Discrete indices beyond register size
    3. Negative indices beyond register size
    """
    qarr = QubitArray("q", 5)

    with pytest.raises(
        IndexError,
        match="Index 6 is out of bounds for register of size 5",
    ):
        qarr[0:6]

    with pytest.raises(IndexError, match="Index out of bounds for register of size 5"):
        qarr[[0, 5]]

    with pytest.raises(IndexError, match="Slice start index out of bounds"):
        qarr[-6:]


def test_wrong_classical_typing():
    """Tests runtime type safety between quantum and classical registers."""
    barr = BitArray("c", 10)

    with pytest.raises(TypeError):
        QubitSlice(barr, slice(0, 4))


def test_empty_slices():
    """Tests handling of empty slice results.

    We need this test because empty slices should be handled gracefully rather
    than raising errors.

    Tests:
    1. Slice with stop < start returns empty slice
    2. Empty list of indices returns empty slice
    """
    qarr = QubitArray("q", 10)

    slice1 = qarr[4:2]
    assert len(list(slice1)) == 0

    slice2 = qarr[[]]
    assert len(list(slice2)) == 0


def test_combining_different_registers():
    """Tests safety when combining slices from different registers.

    We need this test because combining slices from different registers should
    be prevented to avoid confusion and maintain register isolation.

    Tests:
    1. Attempting to combine slices from different registers raises ValueError
    """
    qarr1 = QubitArray("q1", 10)
    qarr2 = QubitArray("q2", 10)

    s1 = qarr1[0:2]
    s2 = qarr2[0:2]

    with pytest.raises(
        ValueError,
        match="Can only combine slices from the same register",
    ):
        s1 + s2


def test_iteration_and_indexing():
    """Tests iteration and indexing behavior of slices.

    We need this test because slices should support both iteration and indexing
    in a way that's consistent with Python sequences.

    Tests:
    1. Iteration over slice yields correct number of elements
    2. Indexing into slice returns correct elements
    """
    qarr = QubitArray("q", 10)
    slice1 = qarr[0:4]

    elements = list(slice1)
    assert len(elements) == 4

    element = slice1[2]
    assert element == qarr[2]


def test_reorder_command():
    """Tests creating reorder commands.

    Tests:
    1. Creating reorder command from continuous slice
    2. Creating reorder command from discrete slice
    """
    qarr = QubitArray("q", 4)

    # Continuous slice
    slice1 = qarr[0:4]
    cmd1 = slice1.reorder([3, 1, 0, 2])
    assert isinstance(cmd1, Reorder)
    assert cmd1.slice is slice1
    assert cmd1.permutation == [3, 1, 0, 2]

    # Discrete slice
    slice2 = qarr[[0, 1, 2, 3]]
    cmd2 = slice2.reorder([2, 3, 0, 1])
    assert isinstance(cmd2, Reorder)
    assert cmd2.slice is slice2
    assert cmd2.permutation == [2, 3, 0, 1]


def test_reorder_in_conditional():
    """Tests reorder commands in conditional blocks."""

    qarr = QubitArray("q", 4)
    slice1 = qarr[0:4]
    barr = BitArray("c", 4)

    # Should be able to create reorder command in conditional
    _ = If(barr[0] == 0).Then(
        slice1.reorder([3, 1, 0, 2]),
    )
    # Backend decides how to implement this


def test_fixed_size_qubit_slice_instantiation():
    """Tests direct instantiation of sized qubit slices."""
    qarr = QubitArray("q", 10)

    # Should work
    QubitSlice[4](qarr, slice(0, 4))

    # Should fail
    with pytest.raises(TypeError, match=r"Expected QubitSlice\[4\] of size 4, got 3"):
        QubitSlice[4](qarr, slice(0, 3))


def test_fixed_size_bit_slice_instantiation():
    """Tests direct instantiation of sized bit slices."""
    barr = BitArray("b", 10)

    # Should work
    BitSlice[4](barr, slice(0, 4))

    # Should fail
    with pytest.raises(TypeError, match=r"Expected BitSlice\[4\] of size 4, got 3"):
        BitSlice[4](barr, slice(0, 3))


def test_slice_reordering_size():
    """Tests that reordering preserves slice size."""
    qarr = QubitArray("q", 10)
    slice1: QubitSlice[4] = qarr[0:4]
    reorder = slice1.reorder([3, 1, 0, 2])
    assert len(list(reorder.slice)) == 4  # Size should be preserved


def test_bit_slice_type_safety():
    """Tests type safety for BitSlice similar to QubitSlice."""
    qarr = QubitArray("q", 10)

    with pytest.raises(
        TypeError,
        match="Expected BitArray for BitSlice, got QubitArray",
    ):
        BitSlice(qarr, slice(0, 4))  # Can't create BitSlice from QubitArray


def test_qubit_slice_type_safety():
    """Tests type safety for BitSlice similar to QubitSlice."""
    barr = BitArray("c", 10)

    with pytest.raises(
        TypeError,
        match="Expected QubitArray for QubitSlice, got BitArray",
    ):
        QubitSlice(barr, slice(0, 4))  # Can't create QubitSlice from BitArray


def test_slice_type_safety_for_blocks_wrong_sized_slice():
    from pecos.qeclib.qubit import H
    from pecos.slr import Block

    class MyBlock(Block):
        def __init__(self, qubits: QubitSlice[4]):
            super().__init__()
            self.extend(
                H(qubits),
            )

    qarr = QubitArray("q", 10)

    # This should raise a TypeError
    with pytest.raises(
        TypeCheckError,
        match=r"argument \"qubits\" \(pecos.slr.vars.QubitSlice\) Expected QubitSlice\[4\] of size 4, got 7",
    ):
        MyBlock(qarr[0:7])


def test_slice_type_safety_for_blocks_with_wrong_slice_type():
    from pecos.qeclib.qubit import H
    from pecos.slr import Block

    class MyBlock(Block):
        def __init__(self, qubits: QubitSlice[4]):
            super().__init__()
            self.extend(
                H(qubits),
            )

    barr = BitArray("b", 10)

    # This should raise a TypeError
    with pytest.raises(
        TypeCheckError,
        match=r"argument \"qubits\" \(pecos.slr.vars.BitSlice\) Expected QubitSlice\[4\] of size 4, got 7",
    ):
        MyBlock(barr[0:7])


def test_slice_type_safety_for_blocks_with_qubit_slice():
    from pecos.qeclib.qubit import H
    from pecos.slr import Block

    class MyBlock(Block):
        def __init__(self, qubits: QubitSlice[4]):
            super().__init__()
            self.extend(
                H(qubits),
            )

    qarr = QubitArray("q", 10)
    MyBlock(qarr[0:4])  # Should just work


def test_various_slice_inputs():
    """Tests Rust-like flexibility in accepting different types that provide N elements."""
    qarr = QubitArray("q", 4)  # Exactly 4 qubits
    qarr_big = QubitArray("q", 10)
    slice_four = qarr_big[0:4]  # Slice of 4

    @typechecked
    def process_four(qubits: QubitSlice[4]):
        pass

    # These should work:
    process_four(qarr)  # Direct array of right size
    process_four(slice_four)  # Slice of right size
    process_four(qarr_big[2:6])  # Different slice of right size
    process_four(qarr_big[[0, 2, 4, 6]])  # Discrete indices of right size
    process_four(qarr_big[6:2:-1])  # Negative step but right size

    # Edge cases that should fail:
    with pytest.raises(TypeCheckError):
        process_four(qarr_big[0:5])  # Too many elements

    with pytest.raises(TypeCheckError):
        process_four(qarr_big[0:3])  # Too few elements

    with pytest.raises(TypeCheckError):
        process_four(BitArray("c", 4))  # Wrong type even though right size

    with pytest.raises(TypeError):
        process_four(None)  # None should fail clearly

    # Test empty array/slices
    with pytest.raises(TypeCheckError):
        process_four(QubitArray("q", 0))  # Empty array

    with pytest.raises(TypeCheckError):
        process_four(qarr_big[4:4])  # Empty slice


# TODO: This shouldn't happen!!!
def test_weird_dynamic_type_checking():
    """Tests that we can use dynamic type checking to handle various types of slicing."""
    from pecos.qeclib.qubit import H
    from pecos.slr import Main

    Main(
        qslice := QubitSlice,
        H(qslice),
    )
