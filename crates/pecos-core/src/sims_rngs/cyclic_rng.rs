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

use super::sim_rng::SimRng;
use rand::prelude::*;

const N: usize = 64;

#[derive(Debug, Clone)]
pub struct CyclicSeed(pub [u8; N]);

impl Default for CyclicSeed {
    #[inline]
    fn default() -> Self {
        Self([0; N])
    }
}

impl AsRef<[u8]> for CyclicSeed {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for CyclicSeed {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct CyclicRng {
    seed: CyclicSeed,
    bools: Vec<bool>,
}

impl CyclicRng {
    #[allow(dead_code)]
    #[inline]
    fn set_bools(&mut self, bools: &[bool]) {
        bools.clone_into(&mut self.bools);
    }
}

impl RngCore for CyclicRng {
    #[allow(unused)]
    #[inline]
    fn next_u32(&mut self) -> u32 {
        todo!()
    }

    #[allow(unused)]
    #[inline]
    fn next_u64(&mut self) -> u64 {
        todo!()
    }

    #[allow(unused)]
    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        todo!()
    }
}

impl SeedableRng for CyclicRng {
    type Seed = CyclicSeed;

    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            seed,
            bools: vec![],
        }
    }
}

impl SimRng for CyclicRng {}
