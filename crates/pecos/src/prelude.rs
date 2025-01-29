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

// re-exporting pecos-core
pub use pecos_core::{IndexableElement, Set, VecSet};
// re-exporting pecos-engines
pub use pecos_core::types::{CommandBatch, GateType, QuantumCommand, ShotResult};
// re-exporting pecos-engines
pub use pecos_engines::{
    channels::stdio::StdioChannel,
    channels::{Message, MessageChannel},
    engines::{
        phir_engine::PHIREngine,
        quantum::{new_quantum_engine, new_quantum_engine_full, CliffordEngine, FullEngine},
        ClassicalEngine, HybridEngine,
    },
    errors::QueueError,
    qir::engine::QirClassicalEngine,
};
// re-exporting pecos-engines
pub use pecos_noise::{DepolarizingNoise, NoiseModel};
// re-exporting pecos-qsim
pub use pecos_qsim::ArbitraryRotationGateable;
pub use pecos_qsim::CliffordGateable;
pub use pecos_qsim::QuantumSimulator;
pub use pecos_qsim::SparseStab;
pub use pecos_qsim::StateVec;
