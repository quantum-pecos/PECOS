from pecos.qeclib import qubit
from pecos.slr import BitArray, QubitArray


def test_Measure(compare_qasm):
    q = QubitArray("q_test", 1)
    m = BitArray("m_test", 1)

    prog = qubit.Measure(q[0]) > m[0]
    compare_qasm(prog)
