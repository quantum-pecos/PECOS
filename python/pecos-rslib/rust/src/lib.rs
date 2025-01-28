// Copyright 2024 The PECOS Developers
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License.You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

// PECOS/python/pecos-rslib/rust/src/lib.rs

pub mod phir_bridge;
mod sparse_sim;

use sparse_sim::SparseSim;
mod sparse_stab_bindings;
mod state_vec_bindings;

use sparse_stab_bindings::SparseSim;
use state_vec_bindings::RsStateVec;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn _pecos_rslib(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SparseSim>()?;
    m.add_class::<phir_bridge::PHIREngine>()?;
    m.add_class::<RsStateVec>()?;
    Ok(())
}
