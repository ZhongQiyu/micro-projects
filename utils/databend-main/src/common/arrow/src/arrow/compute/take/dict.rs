// Copyright 2020-2022 Jorge C. Leitão
// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::primitive::take as take_primitive;
use super::Index;
use crate::arrow::array::DictionaryArray;
use crate::arrow::array::DictionaryKey;
use crate::arrow::array::PrimitiveArray;

/// `take` implementation for dictionary arrays
///
/// applies `take` to the keys of the dictionary array and returns a new dictionary array
/// with the same dictionary values and reordered keys
pub fn take<K, I>(values: &DictionaryArray<K>, indices: &PrimitiveArray<I>) -> DictionaryArray<K>
where
    K: DictionaryKey,
    I: Index,
{
    let keys = take_primitive::<K, I>(values.keys(), indices);
    // safety - this operation takes a subset of keys and thus preserves the dictionary's invariant
    unsafe {
        DictionaryArray::<K>::try_new_unchecked(
            values.data_type().clone(),
            keys,
            values.values().clone(),
        )
        .unwrap()
    }
}
