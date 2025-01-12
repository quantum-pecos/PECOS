# Copyright 2024 The PECOS Developers
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

from typing import TYPE_CHECKING
from warnings import warn

from pecos.qeclib.steane.decoders.lookup import (
    FlagLookupQASMActiveCorrectionX,
    FlagLookupQASMActiveCorrectionZ,
)
from pecos.qeclib.steane.gates_sq import paulis, sqrt_paulis
from pecos.qeclib.steane.gates_sq.hadamards import H
from pecos.qeclib.steane.gates_tq import transversal_tq
from pecos.qeclib.steane.meas.destructive_meas import MeasDecode
from pecos.qeclib.steane.preps.pauli_states import PrepRUS
from pecos.qeclib.steane.preps.t_plus_state import (
    PrepEncodeTPlusFTRUS,
    PrepEncodeTPlusNonFT,
)
from pecos.qeclib.steane.qec.qec_3parallel import ParallelFlagQECActiveCorrection
from pecos.slr import (
    BitArray,
    Block,
    If,
    Permute,
    QubitArray,
    Vars,
)

if TYPE_CHECKING:
    from pecos.slr import (
        BitSlice,
        QubitSlice,
    )


class Steane(Vars):
    """A generic implementation of a Steane code and operations.

    This represents one particular choice of Steane protocols. For finer control construct your own class
    or utilize the library of Steane code protocols directly."""

    def __init__(
        self,
        name: str,
        default_rus_limit: int = 3,
        ancillas: QubitSlice[3] | None = None,
    ):
        super().__init__()
        self.d = QubitArray(f"{name}_d", 7)
        self.a = ancillas or QubitArray(f"{name}_a", 3)
        self.c = BitArray(f"{name}_c", 32)

        if self.a.size < 3:
            msg = f"Steane ancilla registers must have >= 3 qubits (provided: {self.a.size})"
            raise ValueError(msg)

        # TODO: Make it so I can put these in self.c... need to convert things like if(c) and c = a ^ b, a = 0;
        #  to allow lists of bits
        self.syn_meas = BitArray(f"{name}_syn_meas", 32)
        self.last_raw_syn_x = BitArray(f"{name}_last_raw_syn_x", 32)
        self.last_raw_syn_z = BitArray(f"{name}_last_raw_syn_z", 32)
        self.scratch = BitArray(f"{name}_scratch", 32)
        self.flag_x = BitArray(f"{name}_flag_x", 3)
        self.flag_z = BitArray(f"{name}_flags_z", 3)

        self.flags = BitArray(f"{name}_flags", 3)  # weird error when using [c, c, c]

        self.raw_meas = BitArray(f"{name}_raw_meas", 7)

        self.syn_x = BitArray(f"{name}_syn_x", 3)
        self.syn_z = BitArray(f"{name}_syn_z", 3)
        self.syndromes = BitArray(f"{name}_syndromes", 3)
        self.verify_prep = BitArray(f"{name}_verify_prep", 32)

        self.vars = [
            self.d,
        ]

        if ancillas is None:
            self.vars.append(self.a)

        self.vars.extend(
            [
                self.c,
                self.syn_meas,
                self.last_raw_syn_x,
                self.last_raw_syn_z,
                self.scratch,
                self.flag_x,
                self.flag_z,
                self.flags,
                self.raw_meas,
                self.syn_x,
                self.syn_z,
                self.syndromes,
                self.verify_prep,
            ],
        )

        # derived classical registers
        self.log_raw = self.c[[1]]
        self.log = self.c[[2]]
        self.pf_x = self.c[[3]]
        self.pf_z = self.c[[4]]
        self.t_meas = self.c[[5]]
        self.tdg_meas = self.c[[6]]

        self.default_rus_limit = default_rus_limit

    def p(
        self,
        state: str,
        reject: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """Prepare a logical qubit in a logical Pauli basis state."""
        block = PrepRUS(
            q=self.d[:],
            a=self.a[[0]],
            init=self.verify_prep[[0]],
            limit=rus_limit or self.default_rus_limit,
            state=state,
            first_round_reset=True,
        )
        if reject is not None:
            block.extend(reject.set(self.verify_prep[[0]]))
        return block

    def px(self, reject: BitSlice[1] | None = None, rus_limit: int | None = None):
        """Prepare logical |+X>, a.k.a. |+>"""
        return self.p("+X", reject=reject, rus_limit=rus_limit)

    def pnx(self, reject: BitSlice[1] | None = None, rus_limit: int | None = None):
        """Prepare logical |-X>, a.k.a. |->"""
        return self.p("-X", reject=reject, rus_limit=rus_limit)

    def py(self, reject: BitSlice[1] | None = None, rus_limit: int | None = None):
        """Prepare logical |+Y>, a.k.a. |+i>"""
        return self.p("+Y", reject=reject, rus_limit=rus_limit)

    def pny(self, reject: BitSlice[1] | None = None, rus_limit: int | None = None):
        """Prepare logical |-Y>, a.k.a. |-i>"""
        return self.p("-Y", reject=reject, rus_limit=rus_limit)

    def pz(self, reject: BitSlice[1] | None = None, rus_limit: int | None = None):
        """Prepare logical |+Z>, a.k.a. |0>"""
        return self.p("+Z", reject=reject, rus_limit=rus_limit)

    def pnz(self, reject: BitSlice[1] | None = None, rus_limit: int | None = None):
        """Prepare logical |-Z>, a.k.a. |1>"""
        return self.p("-Z", reject=reject, rus_limit=rus_limit)

    def nonft_prep_t_plus_state(self):
        """Prepare logical T|+X> in a non-fault tolerant manner."""

        return PrepEncodeTPlusNonFT(
            q=self.d[:],
        )

    def prep_t_plus_state(
        self,
        reject: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """Prepare logical T|+X> in a fault-tolerant manner."""
        block = Block(
            self.scratch.set(0),
            PrepEncodeTPlusFTRUS(
                d=self.d[:],
                a=self.a,
                out=self.scratch[0:2],
                reject=self.scratch[[2]],
                flag_x=self.flag_x[:],
                flag_z=self.flag_z[:],
                flags=self.flags[:],
                last_raw_syn_x=self.last_raw_syn_x[:3],
                last_raw_syn_z=self.last_raw_syn_z[:3],
                limit=rus_limit or self.default_rus_limit,
            ),
        )
        if reject is not None:
            block.extend(reject.set(self.scratch[[2]]))
        return block

    def nonft_prep_tdg_plus_state(self):
        """Prepare logical Tdg|+X> in a non-fault tolerant manner."""
        return Block(
            self.nonft_prep_t_plus_state(),
            self.z(),
        )

    def prep_tdg_plus_state(
        self,
        reject: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """Prepare logical Tdg|+X> in a fault-tolerant manner."""
        return Block(
            self.prep_t_plus_state(reject=reject, rus_limit=rus_limit),
            self.szdg(),
        )

    def x(self):
        """Logical Pauli X gate"""
        return paulis.X(self.d)

    def y(self):
        """Logical Pauli Y gate"""
        return paulis.Y(self.d)

    def z(self):
        """Logical Pauli Z gate"""
        return paulis.Z(self.d)

    def h(self):
        """Logical Hadamard gate"""
        return H(self.d)

    def sx(self):
        """Sqrt of X."""
        return sqrt_paulis.SX(self.d)

    def sxdg(self):
        """Adjoint of sqrt of X."""
        return sqrt_paulis.SXdg(self.d)

    def sy(self):
        """Sqrt of Y."""
        return sqrt_paulis.SY(self.d)

    def sydg(self):
        """Adjoint of sqrt of Y."""
        return sqrt_paulis.SYdg(self.d)

    def sz(self):
        """Sqrt of Z. Also known as the S gate."""
        return sqrt_paulis.SZ(self.d)

    def nonft_t(self, aux: Steane):
        """T gate via teleportation using non-fault-tolerant initialization of the T|+> state."""
        return Block(
            aux.nonft_prep_t_plus_state(),
            self.cx(aux),
            aux.mz(self.t_meas),
            If(self.t_meas == 1).Then(self.sz()),
        )

    def t(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """T gate via teleportation using fault-tolerant initialization of the T|+> state."""
        return Block(
            aux.prep_t_plus_state(reject=reject, rus_limit=rus_limit),
            self.cx(aux),
            aux.mz(self.t_meas),
            If(self.t_meas == 1).Then(self.sz()),  # SZ/S correction.
        )

    def nonft_tdg(self, aux: Steane):
        """Tdg gate via teleportation using non-fault-tolerant initialization of the Tdg|+> state."""
        return Block(
            aux.nonft_prep_tdg_plus_state(),
            self.cx(aux),
            aux.mz(self.tdg_meas),
            If(self.tdg_meas == 1).Then(self.szdg()),
        )

    def tdg(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """Tdg gate via teleportation using fault-tolerant initialization of the Tdg|+> state."""
        return Block(
            aux.prep_tdg_plus_state(reject=reject, rus_limit=rus_limit),
            self.cx(aux),
            aux.mz(self.tdg_meas),
            If(self.tdg_meas == 1).Then(self.szdg()),  # SZdg/Sdg correction.
        )

    #  Begin Experimental: ------------------------------------

    def nonft_t_tel(self, aux: Steane):
        """Warning:
            This is experimental.

        T gate via teleportation using non-fault-tolerant initialization of the T|+> state.

        This version teleports the logical qubit from the original qubit to the auxiliary logical qubit. For
        convenience, the qubits are relabeled, so you can continue to use the original Steane code logical qubit.
        """
        warn("Using experimental feature: nonft_t_tel", stacklevel=2)
        return Block(
            aux.nonft_prep_t_plus_state(),
            aux.cx(self),
            self.mz(self.t_meas),
            If(self.t_meas == 1).Then(aux.x(), aux.sz()),
            self.permute(aux),
        )

    def t_tel(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """Warning:
            This is experimental.

        T gate via teleportation using fault-tolerant initialization of the T|+> state.

        This version teleports the logical qubit from the original qubit to the auxiliary logical qubit. For
        convenience, the qubits are relabeled, so you can continue to use the original Steane code logical qubit.
        """
        warn("Using experimental feature: t_tel", stacklevel=2)
        return Block(
            aux.prep_t_plus_state(reject=reject, rus_limit=rus_limit),
            aux.cx(self),
            self.mz(self.t_meas),
            If(self.t_meas == 1).Then(aux.x(), aux.sz()),  # SZ/S correction.
            self.permute(aux),
        )

    def nonft_tdg_tel(self, aux: Steane):
        """Warning:
            This is experimental.

        Tdg gate via teleportation using non-fault-tolerant initialization of the Tdg|+> state.

        This version teleports the logical qubit from the original qubit to the auxiliary logical qubit. For
        convenience, the qubits are relabeled, so you can continue to use the original Steane code logical qubit.
        """
        warn("Using experimental feature: nonft_tdg_tel", stacklevel=2)
        return Block(
            aux.nonft_prep_tdg_plus_state(),
            aux.cx(self),
            self.mz(self.tdg_meas),
            If(self.tdg_meas == 1).Then(aux.x(), aux.szdg()),
            self.permute(aux),
        )

    def tdg_tel(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """Warning:
            This is experimental.

        Tdg gate via teleportation using fault-tolerant initialization of the Tdg|+> state.

        This version teleports the logical qubit from the original qubit to the auxiliary logical qubit. For
        convenience, the qubits are relabeled, so you can continue to use the original Steane code logical qubit.
        """
        warn("Using experimental feature: tdg_tel", stacklevel=2)
        return Block(
            aux.prep_tdg_plus_state(reject=reject, rus_limit=rus_limit),
            aux.cx(self),
            self.mz(self.tdg_meas),
            If(self.t_meas == 1).Then(aux.x(), aux.szdg()),  # SZdg/Sdg correction.
            self.permute(aux),
        )

    def t_cor(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        flag: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """T gate via teleportation using fault-tolerant initialization of the T|+> state.

        Applies active corrections of errors diagnozed by the measurement for gate teleportation.
        """
        warn("Using experimental feature: t_cor", stacklevel=2)
        block = Block(
            # gate teleportation without logical correction
            aux.prep_t_plus_state(reject=reject, rus_limit=rus_limit),
            self.cx(aux),
            aux.mz(self.t_meas),
            # active error correction
            self.syn_z.set(aux.syn_meas),
            self.last_raw_syn_z.set(0),
            self.pf_x.set(0),
            FlagLookupQASMActiveCorrectionZ(
                self.d[:],
                self.syn_z[:],
                self.syn_z[:],
                self.last_raw_syn_z[:],
                self.pf_x,
                self.syn_z[:],  # TODO: why are these both the same. give reason
                self.syn_z[:],
                self.scratch[:7],
            ),
            # logical correction
            If(self.t_meas == 1).Then(self.sz()),
        )
        if flag is not None:
            block.extend(If(self.syn_z != 0).Then(flag.set(1)))
        return block

    def tdg_cor(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        flag: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ):
        """Tdg gate via teleportation using fault-tolerant initialization of the Tdg|+> state.

        Applies active corrections of errors diagnozed by the measurement for gate teleportation.
        """
        warn("Using experimental feature: t_cor", stacklevel=2)
        block = Block(
            # gate teleportation without logical correction
            aux.prep_tdg_plus_state(reject=reject, rus_limit=rus_limit),
            self.cx(aux),
            aux.mz(self.tdg_meas),
            # active error correction
            self.syn_z.set(aux.syn_meas),
            self.last_raw_syn_z.set(0),
            self.pf_x.set(0),
            FlagLookupQASMActiveCorrectionZ(
                self.d[:],
                self.syn_z[:],
                self.syn_z[:],
                self.last_raw_syn_z[:],
                self.pf_x,
                self.syn_z[:],
                self.syn_z[:],
                self.scratch[:7],
            ),
            # logical correction
            If(self.tdg_meas == 1).Then(self.szdg()),
        )
        if flag is not None:
            block.extend(If(self.syn_z != 0).Then(flag.set(1)))
        return block

    # End Experimental: ------------------------------------

    def szdg(self):
        """Adjoint of Sqrt of Z. Also known as the Sdg gate."""
        return sqrt_paulis.SZdg(self.d)

    def cx(self, target: Steane):
        """Logical CX"""
        return transversal_tq.CX(self.d[:], target.d[:])

    def cy(self, target: Steane):
        """Logical CY"""
        return transversal_tq.CY(self.d[:], target.d[:])

    def cz(self, target: Steane):
        """Logical CZ"""
        return transversal_tq.CZ(self.d[:], target.d[:])

    def m(self, meas_basis: str, log: BitSlice[1] | None = None):
        """Destructively measure the logical qubit in some Pauli basis."""
        block = MeasDecode(
            q=self.d[:],
            meas_basis=meas_basis,
            meas=self.raw_meas[:],
            log_raw=self.log_raw,
            log=self.log,
            syn_meas=self.syn_meas[:],
            pf_x=self.pf_x,
            pf_z=self.pf_z,
            last_raw_syn_x=self.last_raw_syn_x[:3],
            last_raw_syn_z=self.last_raw_syn_z[:3],
        )
        if log is not None:
            block.extend(log.set(self.log))
        return block

    def mx(self, log: BitSlice[1] | None = None):
        """Logical destructive measurement of the logical X operator."""
        return self.m("X", log=log)

    def my(self, log: BitSlice[1] | None = None):
        """Logical destructive measurement of the logical Y operator."""
        return self.m("Y", log=log)

    def mz(self, log: BitSlice[1] | None = None):
        """Logical destructive measurement of the logical Z operator."""
        return self.m("Z", log=log)

    def qec(self, flag: BitSlice[1] | None = None):
        block = ParallelFlagQECActiveCorrection(
            q=self.d[:],
            a=self.a,
            flag_x=self.flag_x[:],
            flag_z=self.flag_z[:],
            flags=self.flags[:],
            syn_x=self.syn_x[:],
            syn_z=self.syn_z[:],
            last_raw_syn_x=self.last_raw_syn_x[:3],
            last_raw_syn_z=self.last_raw_syn_z[:3],
            syndromes=self.syndromes[:],
            pf_x=self.pf_x,
            pf_z=self.pf_z,
            scratch=self.scratch[:7],
        )
        if flag is not None:
            block.extend(If(self.flags != 0).Then(flag.set(1)))
        return block

    def qec_steane(
        self,
        aux: Steane,
        reject_x: BitSlice[1] | None = None,
        reject_z: BitSlice[1] | None = None,
        flag_x: BitSlice[1] | None = None,
        flag_z: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ) -> Block:
        """Run a Steane-type error-correction cycle of this code."""
        return Block(
            self.qec_steane_x(
                aux,
                reject=reject_x,
                flag=flag_x,
                rus_limit=rus_limit,
            ),
            self.qec_steane_z(
                aux,
                reject=reject_z,
                flag=flag_z,
                rus_limit=rus_limit,
            ),
        )

    def qec_steane_x(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        flag: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ) -> Block:
        """Run a Steane-type error-correction cycle for X errors."""
        warn("Using experimental feature: qec_steane_x", stacklevel=2)
        block = Block(
            aux.px(reject=reject, rus_limit=rus_limit),
            self.cx(aux),
            aux.mz(),
            self.syn_z.set(aux.syn_meas),
            self.last_raw_syn_z.set(0),
            self.pf_x.set(0),
            FlagLookupQASMActiveCorrectionZ(
                self.d[:],
                self.syn_z[:],
                self.syn_z[:],
                self.last_raw_syn_z[:],
                self.pf_x,
                self.syn_z[:],
                self.syn_z[:],
                self.scratch[:7],
            ),
        )
        if flag is not None:
            block.extend(If(self.syn_z != 0).Then(flag.set(1)))
        return block

    def qec_steane_z(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        flag: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ) -> Block:
        """Run a Steane-type error-correction cycle for Z errors."""
        warn("Using experimental feature: qec_steane_z", stacklevel=2)
        block = Block(
            aux.pz(reject=reject, rus_limit=rus_limit),
            aux.cx(self),
            aux.mx(),
            self.syn_x.set(aux.syn_meas),
            self.last_raw_syn_x.set(0),
            self.pf_z.set(0),
            FlagLookupQASMActiveCorrectionX(
                self.d[:],
                self.syn_x[:],
                self.syn_x[:],
                self.last_raw_syn_x[:],
                self.pf_z,
                self.syn_x[:],
                self.syn_x[:],
                self.scratch[:7],
            ),
        )
        if flag is not None:
            block.extend(If(self.syn_x != 0).Then(flag.set(1)))
        return block

    def qec_tel(
        self,
        aux: Steane,
        reject_x: BitSlice[1] | None = None,
        reject_z: BitSlice[1] | None = None,
        flag_x: BitSlice[1] | None = None,
        flag_z: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ) -> Block:
        """Run a teleportation-based error correction cycle."""
        return Block(
            self.qec_tel_x(aux, reject_x, flag_x, rus_limit),
            self.qec_tel_z(aux, reject_z, flag_z, rus_limit),
        )

    def qec_tel_x(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        flag: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ) -> Block:
        """Run a teleportation-based error correction cycle for X errors."""
        warn("Using experimental feature: qec_tel_x", stacklevel=2)
        block = Block(
            # teleport
            aux.px(reject=reject, rus_limit=rus_limit),
            aux.cx(self),
            self.mz(),
            If(self.log == 1).Then(aux.x()),
            Permute(self.d, aux.d),
            # update syndromes and pauli frame
            self.last_raw_syn_x.set(0),
            self.last_raw_syn_z.set(0),
            self.syn_z.set(self.syn_meas),
            self.pf_x.set(0),
        )
        if flag is not None:
            block.extend(If(self.syn_meas != 0).Then(flag.set(1)))
        return block

    def qec_tel_z(
        self,
        aux: Steane,
        reject: BitSlice[1] | None = None,
        flag: BitSlice[1] | None = None,
        rus_limit: int | None = None,
    ) -> Block:
        """Run a teleportation-based error correction cycle for Z errors."""
        warn("Using experimental feature: qec_tel_z", stacklevel=2)
        block = Block(
            # teleport
            aux.pz(reject=reject, rus_limit=rus_limit),
            self.cx(aux),
            self.mx(),
            If(self.log == 1).Then(aux.z()),
            Permute(self.d, aux.d),
            # update syndromes and pauli frame
            self.last_raw_syn_x.set(0),
            self.last_raw_syn_z.set(0),
            self.syn_x.set(self.syn_meas),
            self.pf_z.set(0),
        )
        if flag is not None:
            block.extend(If(self.syn_meas != 0).Then(flag.set(1)))
        return block

    def permute(self, other: Steane):
        """Permute this code block (including both quantum and classical registers) with another."""
        block = Block(
            Permute(self.d, other.d),
            Permute(self.a, other.a),
        )
        for var_a, var_b in zip(self.vars, other.vars):
            if isinstance(var_a, BitArray):
                block.extend(
                    var_a.set(var_a ^ var_b),
                    var_b.set(var_b ^ var_a),
                    var_a.set(var_a ^ var_b),
                )
        return block
