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

use super::Index;
use crate::arrow::array::Array;
use crate::arrow::array::PrimitiveArray;
use crate::arrow::array::StructArray;
use crate::arrow::bitmap::Bitmap;
use crate::arrow::bitmap::MutableBitmap;
use crate::arrow::error::Result;

#[inline]
fn take_validity<I: Index>(
    validity: Option<&Bitmap>,
    indices: &PrimitiveArray<I>,
) -> Result<Option<Bitmap>> {
    let indices_validity = indices.validity();
    match (validity, indices_validity) {
        (None, _) => Ok(indices_validity.cloned()),
        (Some(validity), None) => {
            let iter = indices.values().iter().map(|index| {
                let index = index.to_usize();
                validity.get_bit(index)
            });
            Ok(MutableBitmap::from_trusted_len_iter(iter).into())
        }
        (Some(validity), _) => {
            let iter = indices.iter().map(|x| match x {
                Some(index) => {
                    let index = index.to_usize();
                    validity.get_bit(index)
                }
                None => false,
            });
            Ok(MutableBitmap::from_trusted_len_iter(iter).into())
        }
    }
}

pub fn take<I: Index>(array: &StructArray, indices: &PrimitiveArray<I>) -> Result<StructArray> {
    let values: Vec<Box<dyn Array>> = array
        .values()
        .iter()
        .map(|a| super::take(a.as_ref(), indices))
        .collect::<Result<_>>()?;
    let validity = take_validity(array.validity(), indices)?;
    Ok(StructArray::new(
        array.data_type().clone(),
        values,
        validity,
    ))
}
