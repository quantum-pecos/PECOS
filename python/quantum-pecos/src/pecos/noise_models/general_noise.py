from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np

from pecos.error_models.error_model_abc import ErrorModel
from pecos.error_models.noise_impl.noise_meas_bitflip_leakage import (
    noise_meas_bitflip_leakage,
)
from pecos.error_models.noise_impl.noise_sq_depolarizing_leakage import (
    noise_sq_depolarizing_leakage,
)
from pecos.error_models.noise_impl.noise_tq_depolarizing_leakage import (
    noise_tq_depolarizing_leakage,
)
from pecos.error_models.noise_impl_old.gate_groups import one_qubits, two_qubits
from pecos.reps.pypmir.op_types import QOp

if TYPE_CHECKING:
    from pecos.reps.pypmir.block_types import SeqBlock
    from pecos.reps.pypmir.op_types import MOp

two_qubit_paulis = {
    "IX",
    "IY",
    "IZ",
    "XI",
    "XX",
    "XY",
    "XZ",
    "YI",
    "YX",
    "YY",
    "YZ",
    "ZI",
    "ZX",
    "ZY",
    "ZZ",
}
SYMMETRIC_P2_PAULI_MODEL = {p: 1 / 15 for p in two_qubit_paulis}

one_qubit_paulis = {
    "X",
    "Y",
    "Z",
}
SYMMETRIC_P1_PAULI_MODEL = {p: 1 / 3 for p in one_qubit_paulis}


class GeneralNoiseModel(ErrorModel):
    """Parameterized error mode."""

    def __init__(self, error_params: dict) -> None:
        super().__init__(error_params=error_params)
        self._eparams = None

        self.qubit_set = set()
        self.leaked_qubits = set()

    def reset(self):
        """Reset error generator for another round of syndrome extraction."""
        return GeneralNoiseModel(error_params=self.error_params)

    def init(self, num_qubits, machine=None, reset_leakage=True):
        self.machine = machine

        self.qubit_set = set(range(num_qubits))

        if reset_leakage:
            self.leaked_qubits = set()

        if not self.error_params:
            msg = "Error params not set!"
            raise Exception(msg)

        self._eparams = dict(self.error_params)
        self._set_eparams_default()
        self._scale()

        if "p1_error_model" not in self._eparams:
            self._eparams["p1_error_model"] = SYMMETRIC_P1_PAULI_MODEL

        if "p2_error_model" not in self._eparams:
            self._eparams["p2_error_model"] = SYMMETRIC_P2_PAULI_MODEL

        if "p2_mem" in self._eparams and "p2_mem_error_model" not in self._eparams:
            self._eparams["p2_mem_error_model"] = SYMMETRIC_P2_PAULI_MODEL

    def _set_eparams_default(self):
        for key in [
            "p1",
            "p2",
            "p_meas",
            "p_init",
            "quadratic_dephasing_rate",
            "linear_dephasing_rate",
            "p_crosstalk_meas",
            "p_crosstalk_init",
        ]:
            if key not in self._eparams:
                self._eparams[key] = 0.0

        for key in ["coherent_dephasing"]:
            if key not in self._eparams:
                self._eparams[key] = False

        for key in [
            "coherent_to_incoherent_factor",
            "p1_emission_rescale",
            "emission_scale",
            "scale",
            "init_scale",
            "leakage_scale",
            "memory_scale",
            "p1_scale",
            "p2_scale",
            "crosstalk_scale",
            "meas_scale",
            "p_crosstalk_meas_rescale",
            "p_crosstalk_init_rescale",
        ]:
            if key not in self._eparams:
                self._eparams[key] = 1.0

        for key in ["p_init_leak_ratio", "p1_emission_ratio", "p2_emission_ratio"]:
            if key not in self._eparams:
                self._eparams[key] = 0.5

        if (
            "p1_pauli_model" not in self._eparams
            or self._eparams["p1_pauli_model"] == "symmetric"
        ):
            self._eparams["p1_pauli_model"] = SYMMETRIC_P1_PAULI_MODEL

        if (
            "p2_pauli_model" not in self._eparams
            or self._eparams["p2_pauli_model"] == "symmetric"
        ):
            self._eparams["p2_pauli_model"] = SYMMETRIC_P2_PAULI_MODEL

    def _scale(self):

        if not self._eparams["coherent_dephasing"]:
            self._eparams["quadratic_dephasing_rate"] *= self._eparams[
                "coherent_to_incoherent_factor"
            ]
            # to get rid of the 0.5 factor in (rate x duration x 0.5)^2 calcs
            self._eparams[
                "quadratic_dephasing_rate"
            ] *= 0.5  # << added only for the incoherent approximation

        self._eparams["quadratic_dephasing_rate"] *= 2 * np.pi

        if self._eparams.get("linear_dephasing_rate") is None:
            self._eparams["linear_dephasing_rate"] = 0.0

        # ==============================================================================================================
        # Begin scaling
        # ==============================================================================================================
        scale = self._eparams["scale"]

        self._eparams["quadratic_dephasing_rate"] *= np.sqrt(
            self._eparams["memory_scale"] * scale,
        )
        self._eparams["linear_dephasing_rate"] *= self._eparams["memory_scale"] * scale

        cxscale = self._eparams["crosstalk_scale"] * scale

        self._eparams["p_crosstalk_meas"] *= self._eparams["p_crosstalk_meas_rescale"]
        self._eparams["p_crosstalk_init"] *= self._eparams["p_crosstalk_init_rescale"]

        self._eparams["p_crosstalk_meas"] *= self._eparams["meas_scale"] * cxscale
        self._eparams["p_crosstalk_init"] *= self._eparams["init_scale"] * cxscale

        self._eparams["p1"] *= self._eparams["p1_scale"] * scale
        self._eparams["p2"] *= self._eparams["p2_scale"] * scale

        if isinstance(self._eparams["p_meas"], (tuple, list)):
            m1, m2 = self._eparams["p_meas"]

            m1 *= self._eparams["meas_scale"] * scale
            m2 *= self._eparams["meas_scale"] * scale

            self._eparams["p_meas"] = (m1, m2)
        else:
            self._eparams["p_meas"] *= self._eparams["meas_scale"] * scale

        self._eparams["p_init"] *= self._eparams["init_scale"] * scale

        self._eparams["p_init_leak_ratio"] *= self._eparams["leakage_scale"]

        self._eparams["p1_emission_ratio"] *= self._eparams[
            "p1_emission_rescale"
        ]  # tomograph to average pi/2
        self._eparams["p1_emission_ratio"] *= self._eparams["emission_scale"] * scale
        self._eparams["p1_emission_ratio"] = min(
            self._eparams["p1_emission_ratio"],
            1.0,
        )

        self._eparams["p2_emission_ratio"] *= self._eparams["emission_scale"] * scale
        self._eparams["p2_emission_ratio"] = min(
            self._eparams["p2_emission_ratio"],
            1.0,
        )
        # ==============================================================================================================
        # End scaling
        # ==============================================================================================================

        # Rescaling from average error to total error
        self._eparams["p1"] *= 3 / 2
        self._eparams["p2"] *= 5 / 4
        self._eparams["p_crosstalk_meas"] *= 18 / 5
        self._eparams["p_crosstalk_init"] *= 18 / 5

        # Experimentalists are reporting average error rate for dephasing, need to convert to total.
        self._eparams["quadratic_dephasing_rate"] *= np.sqrt(3 / 2)
        self._eparams["linear_dephasing_rate"] *= 3 / 2

        if self._eparams.get("biased_tq_pauli_noise"):
            self._eparams["p2_pauli_model"] = self._eparams["p2_pauli_model_exp"]

    def shot_reinit(self) -> None:
        """Run all code needed at the beginning of each shot, e.g., resetting state."""

    def process(self, qops: list[QOp], call_back=None) -> list[QOp | SeqBlock]:
        noisy_ops = []

        for op in qops:
            qops_after = None
            qops_before = None
            erroneous_ops = None

            match op.name:

                case x if (
                    "noiseless_gates" in self._eparams
                    and x in self._eparams["noiseless_gates"]
                ):
                    pass

                case "init |0>" | "Init" | "Init +Z":
                    qops_after = self.faults_init(op, flip="X")

                case x if x in one_qubits:
                    erroneous_ops = noise_sq_depolarizing_leakage(
                        op,
                        p=self._eparams["p1"],
                        noise_dict=self._eparams["p1_error_model"],
                        machine=self.machine,
                    )

                case x if x in two_qubits:
                    qops_after = noise_tq_depolarizing_leakage(
                        op,
                        p=self._eparams["p2"],
                        noise_dict=self._eparams["p2_error_model"],
                        machine=self.machine,
                    )

                    if self._eparams.get("p2_mem"):
                        qops_mem = noise_tq_depolarizing_leakage(
                            op,
                            p=self._eparams["p2_mem"],
                            noise_dict=self._eparams["p2_mem_error_model"],
                            machine=self.machine,
                        )
                        if qops_after:
                            qops_after = qops_after.extend(qops_mem)

                case "measure Z" | "Measure" | "Measure +Z":
                    erroneous_ops = noise_meas_bitflip_leakage(
                        op,
                        p=self._eparams["p_meas"],
                        machine=self.machine,
                    )

                case "Idle" | "Sleep":
                    if self._eparams.get("idle_dephasing", True):
                        erroneous_ops = self.faults_dephasing(
                            op.args,
                            op.metadata["duration"],
                            rate=self._eparams["quadratic_dephasing_rate"],
                        )

                    if erroneous_ops is None:
                        erroneous_ops = [QOp(name="I", args=[])]

                case _:
                    msg = f"This error model doesn't handle gate: {op.name}!"
                    raise Exception(msg)

            if qops_before:
                noisy_ops.extend(qops_before)

            if erroneous_ops is None:
                noisy_ops.append(op)
            else:
                noisy_ops.extend(erroneous_ops)

            if qops_after:
                noisy_ops.extend(qops_after)

        return noisy_ops

    def faults_dephasing(
        self,
        op,
        duration: float,
        rate: float | None = None,
    ) -> list[QOp] | None:
        """Applies both coherent dephasing and linear incoherent dephasing."""

        if rate:
            if self._eparams["coherent_dephasing"]:
                return self.faults_dephase_coherent(op, duration, rate)
            else:  # quadratic incoherent dephasing
                return self.faults_dephase_incoherent(op, duration, rate, linear=False)

        linear_rate = self._eparams.get("linear_dephasing_rate")

        if linear_rate:
            return self.faults_dephase_incoherent(
                op,
                duration,
                rate=linear_rate,
                linear=True,
            )

    def faults_dephase_coherent(
        self,
        op: MOp,
        duration: float,
        rate: float | None = None,
    ) -> list[QOp]:
        """The dephasing noise model for idling qubits.

        Args:
            op: A machine operation, e.g., "Idle" or "Sleep".
            duration: The time spent dephasing.
            rate: custom linear dephasing rate
        """

        notleaked = set(op.args) - self.leaked_qubits

        return [
            QOp(
                name="RZ",
                args=list(notleaked),
                angles=(rate * duration,),
            ),
        ]

    def faults_dephase_incoherent(
        self,
        op: MOp,
        duration: float,
        rate: float | None = None,
        linear=False,
    ) -> list[QOp] | None:
        """The dephasing noise model for idling qubits.

        Args:
            op: A machine operation, e.g., "Idle" or "Sleep".
            duration: The time spent dephasing.
            rate: custom linear dephasing rate
            linear: Whether the scaling should be linear.
        """

        pdeph = rate * duration

        if pdeph:

            if not linear:
                pdeph = np.power(np.sin(pdeph), 2)

            notleaked = set(op.args) - self.leaked_qubits

            # ---------------------------
            # dephasing noise
            # ---------------------------
            rand_nums = np.random.random(len(notleaked)) <= pdeph

            err_qubits = []
            for r, loc in zip(rand_nums, notleaked, strict=False):

                if r:
                    err_qubits.append(loc)

            if err_qubits:
                return [QOp(name="Z", args=err_qubits)]
            else:
                return None

    def faults_init(self, op: QOp, flip: str) -> list[QOp]:
        """The noise model for qubit (re)initialization.

        Args:
            op: A quantum operator
            flip: The symbol for what Pauli operator should be applied if an initialization fault occurs.
        """

        # remove leaked qubits
        self.leaked_qubits -= set(op.args)

        rand_nums = np.random.random(len(op.args)) <= self.error_params["p_init"]

        after = []
        toleak = set()
        for r, loc in zip(rand_nums, op.args, strict=False):
            if r:

                if np.random.random() <= self.error_params["p_init_leak_ratio"]:
                    toleak.add(loc)
                else:
                    after.append(QOp(name=flip, args=[loc]))

        # Leakage noise
        # -------------
        self.leak(toleak, p_leak=self.error_params["leakage_scale"], trigger="init")

        # crosstalk
        # ---------
        if not op.metadata.get("start_init", False) and op.metadata.get("z2qs"):
            noise = self.init_crosstalk(op)
            after.extend(noise)

        return after

    def init_crosstalk(self, op) -> list[QOp]:
        """
        Probability of going down 0 or 1 branch == probability of meas 0 or 1.
        If the qubit is in the 0 state, reset does nothing
        If the qubit is in the 1 state, there are three possibilities all with equal probability of 1/3
        stays in 1, goes to 0, goes to the leaked state

        meas Z -> if 1, 1/3 apply X, 1/3 leak
        """

        var = ("__pecos_scratch", 0)
        p_cross = self.error_params["p_crosstalk_init"]

        # Apply crosstalk equally to all qubits not being measured
        qs = set()
        q_zone = {}
        for gz in self.error_params["crosstalk_zones"]:
            qsz = op.metadata.get("z2qs", {}).get(gz, [])
            for q in qsz:
                q_zone[q] = gz

            qs |= set(qsz)

        qs -= set(op.args)
        ls = (qs & self.leaked_qubits) & self.qubit_set
        qs -= self.leaked_qubits
        qs &= self.qubit_set

        rand_nums = np.random.random(len(qs))

        num_cross = len(op.args) if self.error_params["crosstalk_per_gate"] else 1

        after = []
        for _ in range(num_cross):
            for r, q in zip(rand_nums, qs, strict=False):

                if self.error_params.get("zones"):
                    gz = q_zone[q]
                    p = self.error_params["zones"][gz]["p_crosstalk_init"]
                else:
                    p = p_cross

                if r <= p:

                    after.append(
                        QOp(
                            name="measure Z",
                            args=[q],
                            metadata={
                                "cond": op.metadata.get("cond"),
                                "var": ("__pecos_scratch", 0),
                            },
                        ),
                    )

                    if np.random.random() <= 1 / 3:
                        after.append(
                            QOp(
                                name="init |0>",
                                args=[q],
                                metadata={
                                    "cond": op.metadata.get("cond"),
                                    "cond2": {"a": var, "op": "==", "b": 1},
                                    "init_crosstalk": True,
                                },
                            ),
                        )

                    elif (
                        np.random.random() <= 2 / 3 * self.error_params["leakage_scale"]
                    ):
                        if self.error_params.get("leak2depolar"):
                            if np.random.random() <= 0.75:
                                err = np.random.choice(one_qubit_paulis)
                                after.append(QOp(name=err, args=[q]))
                        else:
                            after.append(
                                QOp(
                                    name="leak",
                                    args=[q],
                                    metadata={
                                        "cond": op.metadata.get("cond"),
                                        "cond2": {"a": var, "op": "==", "b": 1},
                                        "trigger": "init_crosstalk",
                                    },
                                ),
                            )

            if ls and self.error_params.get("seepage", True):
                #  remain leaked w/ 1/3 prob. but will go to |1> 1/3 of the time and |0> 1/3 of the time.
                #  other polarization, it would remain leaked 2/3 of the time, but go to |1> 1/3 of the time.

                rand_nums = np.random.random(len(ls))

                # Leaked qubits
                # 1/3 leaked, 1/3 |1>, 1/3 |0>
                # 2/3 leaked, 1/3 |1>
                # assume an unpolarized beam and average the two polarization results together.
                for r, q in zip(rand_nums, ls, strict=False):

                    if r <= p_cross:

                        if np.random.random() <= 0.5:
                            # 1/3 still leaked
                            if np.random.random() <= 2 / 3:
                                noise = self.unleak(
                                    {q},
                                    pop0_prob=0.5,
                                    trigger="init_crosstalk",
                                )  # go to |0> or |1>
                                after.extend(noise)
                        else:
                            # 2/3 still leaked
                            if np.random.random() <= 1 / 3:
                                noise = self.unleak(
                                    {q},
                                    pop0_prob=0.0,
                                    trigger="init_crosstalk",
                                )  # go to |1>
                                after.extend(noise)

        return after

    def leak(self, locations: set[int], p_leak: float, **meta) -> list[QOp]:
        """The method that leaks qubits.

        Args:
            locations: Set of qubits the ideal gates act on.
            p_leak: Probability to leak.
        """

        error_circ = []
        if locations:

            if self.error_params.get(
                "leak2depolar",
            ):  # Whether to replace leakage with depolarizing noise
                for q in locations:
                    if np.random.random() <= 0.75 * p_leak:
                        err = np.random.choice(one_qubit_paulis)
                        error_circ.append(QOp(name=err, args=[q], metadata=meta))
            else:

                for loc in locations:
                    if np.random.random() <= p_leak:
                        self.leaked_qubits.add(loc)
                        meta["leak"] = True
                        error_circ.append(
                            QOp(name="init |1>", args=[loc], metadata=meta),
                        )

        return error_circ

    def unleak(
        self,
        locations: set[int],
        pop0_prob: float = 0.5,
        trigger=None,
    ) -> list[QOp]:
        """The method that returns leaked qubits to the computation space.

        Args:
            locations: Set of qubits the ideal gates act on.
            pop0_prob: The probability that a qubit returning to the computational space is re-prepared in |0> instead
                of |1>.
            trigger: What type of operation triggered the unleak.
        """

        error_circ = []
        if locations:

            self.leaked_qubits -= locations

            if pop0_prob == 0.0:

                error_circ.append(
                    QOp(
                        name="init |1>",
                        args=list(locations),
                        metadata={"unleak": True, "trigger": trigger},
                    ),
                )

            else:

                rand_nums = np.random.random(len(locations)) <= pop0_prob

                for r, loc in zip(rand_nums, locations, strict=False):

                    if r:
                        error_circ.append(
                            QOp(
                                name="init |0>",
                                args=[loc],
                                metadata={"unleak": True, "trigger": trigger},
                            ),
                        )

                    else:
                        error_circ.append(
                            QOp(
                                name="init |1>",
                                args=[loc],
                                metadata={"unleak": True, "trigger": trigger},
                            ),
                        )
        return error_circ
