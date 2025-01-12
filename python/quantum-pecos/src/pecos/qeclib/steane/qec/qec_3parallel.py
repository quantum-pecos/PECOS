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

from pecos.qeclib.steane.decoders.lookup import (
    FlagLookupQASMActiveCorrectionX,
    FlagLookupQASMActiveCorrectionZ,
)
from pecos.qeclib.steane.syn_extract.six_check_nonflagging import SixUnflaggedSyn
from pecos.qeclib.steane.syn_extract.three_parallel_flagging import (
    ThreeParallelFlaggingXZZ,
    ThreeParallelFlaggingZXX,
)
from pecos.slr import BitSlice, Block, If, QubitSlice


class ParallelFlagQECActiveCorrection(Block):
    """Defining QEC Block that does adaptive syndrome extraction, decodes, and updates the Paul frame."""

    def __init__(
        self,
        q: QubitSlice[7],
        a: QubitSlice[3],
        flag_x: BitSlice[3],
        flag_z: BitSlice[3],
        flags: BitSlice[3],
        syn_x: BitSlice[3],
        syn_z: BitSlice[3],
        last_raw_syn_x: BitSlice[3],
        last_raw_syn_z: BitSlice[3],
        syndromes: BitSlice[3],
        pf_x: BitSlice[1],
        pf_z: BitSlice[1],
        scratch: BitSlice[7],
    ):
        super().__init__(
            # flagging XZZ checks
            ThreeParallelFlaggingXZZ(
                q,
                a,
                flag_x,
                flag_z,
                flags,
                last_raw_syn_x,
                last_raw_syn_z,
            ),
            # flagging ZXX checks
            If(flags == 0).Then(
                ThreeParallelFlaggingZXX(
                    q,
                    a,
                    flag_x,
                    flag_z,
                    flags,
                    last_raw_syn_x,
                    last_raw_syn_z,
                ),
            ),
            # Remeasure all the checks unflagged
            If(flags != 0).Then(
                SixUnflaggedSyn(q, a, syn_x, syn_z),
            ),
            FlagLookupQASMActiveCorrectionX(
                q,
                syn_x,
                syndromes,
                last_raw_syn_x,
                pf_z,
                flag_x,
                flags,
                scratch,
            ),
            FlagLookupQASMActiveCorrectionZ(
                q,
                syn_z,
                syndromes,
                last_raw_syn_z,
                pf_x,
                flag_z,
                flags,
                scratch,
            ),
        )
