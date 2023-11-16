# TODO: Include license information?

import pecos.simulators.custatevec.gates_one_qubit as one_q
import pecos.simulators.custatevec.gates_two_qubit as two_q
from pecos.simulators.custatevec.gates_init import init_zero
from pecos.simulators.custatevec.gates_meas import meas_z

# Supporting gates from table:
#   https://github.com/CQCL/phir/blob/main/spec.md#table-ii---quantum-operations

gate_dict = {
    "Init": init_zero,
    "Measure": meas_z,
    "I": one_q.identity,
    "X": one_q.X,
    "Y": one_q.Y,
    "Z": one_q.Z,
    "RX": one_q.RX,
    "RY": one_q.RY,
    "RZ": one_q.RZ,
    "R1XY": one_q.R1XY,
    "SX": one_q.SX,
    "SXdg": one_q.SXdg,
    "SY": one_q.SY,
    "SYdg": one_q.SYdg,
    "SZ": one_q.SZ,
    "SZdg": one_q.SZdg,
    "H": one_q.H,
    "F": one_q.F,
    "Fdg": one_q.Fdg,
    "T": one_q.T,
    "Tdg": one_q.Tdg,
    "CX": two_q.CX,
    "CY": two_q.CY,
    "CZ": two_q.CZ,
    "RXX": two_q.RXX,
    "RYY": two_q.RYY,
    "RZZ": two_q.RZZ,
    "R2XXYYZZ": two_q.R2XXYYZZ,
    "SXX": two_q.SXX,
    "SXXdg": two_q.SXXdg,
    "SYY": two_q.SYY,
    "SYYdg": two_q.SYYdg,
    "SZZ": two_q.SZZ,
    "SZZdg": two_q.SZZdg,
    "SWAP": two_q.SWAP,
}
