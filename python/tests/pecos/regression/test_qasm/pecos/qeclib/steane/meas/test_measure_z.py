from pecos.qeclib.steane.meas.measure_z import NoFlagMeasureZ
from pecos.slr import CReg, QubitArray


def test_MeasureX(compare_qasm):
    q = QubitArray("q_test", 7)
    a = QubitArray("a_test", 1)
    out = CReg("out_test", 1)

    block = NoFlagMeasureZ(q[0:7], a[0:1], out)
    compare_qasm(block)
