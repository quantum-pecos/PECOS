from pecos.qeclib.steane.preps.plus_h_state import PrepHStateFT
from pecos.slr import BitArray, QubitArray


def test_PrepHStateFT(compare_qasm):
    q = QubitArray("q_test", 7)
    a = QubitArray("a_test", 3)
    out = BitArray("out_test", 2)
    reject = BitArray("reject_test", 1)
    flag_x = BitArray("flag_x_test", 3)
    flag_z = BitArray("flag_z_test", 3)
    flags = BitArray("flags_test", 3)
    last_raw_syn_x = BitArray("last_raw_syn_x_test", 3)
    last_raw_syn_z = BitArray("last_raw_syn_z_test", 3)
    block = PrepHStateFT(
        q,
        a,
        out,
        reject,
        flag_x,
        flag_z,
        flags,
        last_raw_syn_x,
        last_raw_syn_z,
    )
    compare_qasm(block)
