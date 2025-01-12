from pecos.qeclib.steane.syn_extract.six_check_nonflagging import SixUnflaggedSyn
from pecos.slr import BitArray, QubitArray


def test_SixUnflaggedSyn(compare_qasm):
    q = QubitArray("q_test", 7)
    a = QubitArray("a_test", 3)
    syn_x = BitArray("syn_x_test", 3)
    syn_z = BitArray("syn_z_test", 3)

    block = SixUnflaggedSyn(q[:], a[:], syn_x[:], syn_z[:])
    compare_qasm(block)
