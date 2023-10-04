# Copyright 2022 The PECOS Developers
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
# the License.You may obtain a copy of the License at
#
#     https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
# specific language governing permissions and limitations under the License.
import contextlib

with contextlib.suppress(ImportError):
    import wasm3


def read_pywasm3(wasm, stack_size=1000000):
    env = wasm3.Environment()
    rt = env.new_runtime(stack_size)
    mod = env.parse_module(wasm)
    rt.load(mod)

    class Reader:
        def __init__(self, rt) -> None:
            self.rt = rt

        def exec(self, func, args, debug=False):
            args = [int(b) for _, b in args]
            return self.rt.find_function(func)(*args)

    return Reader(rt)
