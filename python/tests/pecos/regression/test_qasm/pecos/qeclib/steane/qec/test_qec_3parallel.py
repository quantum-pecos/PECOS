from pecos.qeclib.steane.qec.qec_3parallel import ParallelFlagQECActiveCorrection
from pecos.slr import BitArray, QubitArray


def test_ParallelFlagQECActiveCorrection(compare_qasm):
    q = QubitArray("q_test", 7)
    a = QubitArray("a_test", 3)
    flag_x = BitArray("flag_x_test", 3)
    flag_z = BitArray("flag_z_test", 3)
    flags = BitArray("flags_test", 3)
    syn_x = BitArray("syn_x_test", 3)
    syn_z = BitArray("syn_z_test", 3)
    last_raw_syn_x = BitArray("last_raw_syn_x_test", 3)
    last_raw_syn_z = BitArray("last_raw_syn_z_test", 3)
    syndromes = BitArray("syndromes_test", 3)
    pf = BitArray("pf_test", 2)
    scratch = BitArray("scratch_test", 7)

    block = ParallelFlagQECActiveCorrection(
        q[:],
        a[:],
        flag_x[:],
        flag_z[:],
        flags[:],
        syn_x[:],
        syn_z[:],
        last_raw_syn_x[:],
        last_raw_syn_z[:],
        syndromes[:],
        pf[[0]],
        pf[[1]],
        scratch[:],
    )
    compare_qasm(block)
