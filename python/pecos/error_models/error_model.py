# Copyright 2023 The PECOS Developers
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
# the License.You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
# specific language governing permissions and limitations under the License.

from typing import Callable, Optional, Union

from .error_model_abc import ErrorModel


class NoErrorModel(ErrorModel):
    """Represents having no error model."""

    def reset(self) -> None:
        """Reset state to initialization state."""
        pass

    def init(self, error_params, num_qubits, machine=None):
        super().init(error_params, num_qubits, machine)
        if self.error_params:
            raise Exception("No error model is being utilized but error parameters are being provided!")

    def shot_reinit(self):
        pass

    def process(self, qops: list, call_back: Optional[Callable] = None) -> Union[list, None]:
        return qops
