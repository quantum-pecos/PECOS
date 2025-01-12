from pecos.qeclib.steane.preps.t_plus_state import (
    PrepEncodeTDagPlusNonFT,
    PrepEncodeTPlusFT,
    PrepEncodeTPlusFTRUS,
    PrepEncodeTPlusNonFT,
)
from pecos.slr import BitArray, QubitArray


def test_PrepEncodeTPlusNonFT(compare_qasm):
    q = QubitArray("q_test", 7)
    block = PrepEncodeTPlusNonFT(q)
    compare_qasm(block)


def test_PrepEncodeTDagPlusNonFT(compare_qasm):
    q = QubitArray("q_test", 7)
    block = PrepEncodeTDagPlusNonFT(q)
    compare_qasm(block)


def test_PrepEncodeTPlusFT(compare_qasm):
    q = QubitArray("q_test", 7)
    a = QubitArray("a_test", 3)
    out = BitArray("out_test", 2)
    reject = BitArray("reject_test", 1)
    flag_x = BitArray("flag_x_test", 3)
    flag_z = BitArray("flag_z_test", 3)
    flags = BitArray("flags_test", 3)
    last_raw_syn_x = BitArray("last_raw_syn_x_test", 3)
    last_raw_syn_z = BitArray("last_raw_syn_z_test", 3)
    block = PrepEncodeTPlusFT(
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


def test_PrepEncodeTPlusFTRUS(compare_qasm):
    q = QubitArray("q_test", 7)
    a = QubitArray("a_test", 3)
    out = BitArray("out_test", 2)
    reject = BitArray("reject_test", 1)
    flag_x = BitArray("flag_x_test", 3)
    flag_z = BitArray("flag_z_test", 3)
    flags = BitArray("flags_test", 3)
    last_raw_syn_x = BitArray("last_raw_syn_x_test", 3)
    last_raw_syn_z = BitArray("last_raw_syn_z_test", 3)

    for limit in [1, 2, 3]:
        block = PrepEncodeTPlusFTRUS(
            q,
            a,
            out,
            reject,
            flag_x,
            flag_z,
            flags,
            last_raw_syn_x,
            last_raw_syn_z,
            limit,
        )
        compare_qasm(block, limit)
