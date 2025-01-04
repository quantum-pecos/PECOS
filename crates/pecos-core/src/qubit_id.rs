// Copyright 2025 The PECOS Developers
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

use crate::IndexableElement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct QubitId(pub usize);

impl IndexableElement for QubitId {
    #[inline]
    fn to_index(&self) -> usize {
        self.0
    }

    #[inline]
    fn from_index(value: usize) -> Self {
        Self(value)
    }
}
