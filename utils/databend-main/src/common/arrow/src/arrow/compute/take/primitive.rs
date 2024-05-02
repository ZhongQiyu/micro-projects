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

use arrow_buffer::bit_util::unset_bit_raw;

use super::Index;
use crate::arrow::array::Array;
use crate::arrow::array::PrimitiveArray;
use crate::arrow::bitmap::Bitmap;
use crate::arrow::bitmap::MutableBitmap;
use crate::arrow::buffer::Buffer;
use crate::arrow::types::NativeType;

// take implementation when neither values nor indices contain nulls
fn take_no_validity<T: NativeType, I: Index>(
    values: &[T],
    indices: &[I],
) -> (Buffer<T>, Option<Bitmap>) {
    let values = indices
        .iter()
        .map(|index| values[index.to_usize()])
        .collect::<Vec<_>>();

    (values.into(), None)
}

// take implementation when only values contain nulls
fn take_values_validity<T: NativeType, I: Index>(
    values: &PrimitiveArray<T>,
    indices: &[I],
) -> (Buffer<T>, Option<Bitmap>) {
    let values_validity = values.validity().unwrap();

    let validity = indices
        .iter()
        .map(|index| values_validity.get_bit(index.to_usize()));
    let validity = MutableBitmap::from_trusted_len_iter(validity);

    let values_values = values.values();

    let values = indices
        .iter()
        .map(|index| values_values[index.to_usize()])
        .collect::<Vec<_>>();

    (values.into(), validity.into())
}

// take implementation when only indices contain nulls
fn take_indices_validity<T: NativeType, I: Index>(
    values: &[T],
    indices: &PrimitiveArray<I>,
) -> (Buffer<T>, Option<Bitmap>) {
    let validity = indices.validity().unwrap();
    let values = indices
        .values()
        .iter()
        .enumerate()
        .map(|(i, index)| {
            let index = index.to_usize();
            match values.get(index) {
                Some(value) => *value,
                None => {
                    if !validity.get_bit(i) {
                        T::default()
                    } else {
                        panic!("Out-of-bounds index {index}")
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    (values.into(), indices.validity().cloned())
}

// take implementation when both values and indices contain nulls
fn take_values_indices_validity<T: NativeType, I: Index>(
    values: &PrimitiveArray<T>,
    indices: &PrimitiveArray<I>,
) -> (Buffer<T>, Option<Bitmap>) {
    let mut bitmap = MutableBitmap::with_capacity(indices.len());

    let values_validity = values.validity().unwrap();

    let values_values = values.values();
    let values = indices
        .iter()
        .map(|index| match index {
            Some(index) => {
                let index = index.to_usize();
                bitmap.push(values_validity.get_bit(index));
                values_values[index]
            }
            None => {
                bitmap.push(false);
                T::default()
            }
        })
        .collect::<Vec<_>>();
    (values.into(), bitmap.into())
}

pub(super) unsafe fn take_values_and_validity_unchecked<T: NativeType, I: Index>(
    values: &[T],
    validity_values: Option<&Bitmap>,
    indices: &PrimitiveArray<I>,
) -> (Vec<T>, Option<Bitmap>) {
    let index_values = indices.values().as_slice();

    let null_count = validity_values.map(|b| b.unset_bits()).unwrap_or(0);

    // first take the values, these are always needed
    let values: Vec<T> = if indices.null_count() == 0 {
        index_values
            .iter()
            .map(|idx| *values.get_unchecked(idx.to_usize()))
            .collect()
    } else {
        indices
            .iter()
            .map(|idx| match idx {
                Some(idx) => *values.get_unchecked(idx.to_usize()),
                None => T::default(),
            })
            .collect()
    };

    if null_count > 0 {
        let validity_values = validity_values.unwrap();
        // the validity buffer we will fill with all valid. And we unset the ones that are null
        // in later checks
        // this is in the assumption that most values will be valid.
        // Maybe we could add another branch based on the null count
        let mut validity = MutableBitmap::with_capacity(indices.len());
        validity.extend_constant(indices.len(), true);
        let validity_ptr = validity.as_slice().as_ptr() as *mut u8;

        if let Some(validity_indices) = indices.validity().as_ref() {
            index_values.iter().enumerate().for_each(|(i, idx)| {
                // i is iteration count
                // idx is the index that we take from the values array.
                let idx = idx.to_usize();
                if !validity_indices.get_bit_unchecked(i) || !validity_values.get_bit_unchecked(idx)
                {
                    unset_bit_raw(validity_ptr, i);
                }
            });
        } else {
            index_values.iter().enumerate().for_each(|(i, idx)| {
                let idx = idx.to_usize();
                if !validity_values.get_bit_unchecked(idx) {
                    unset_bit_raw(validity_ptr, i);
                }
            });
        };
        (values, Some(validity.freeze()))
    } else {
        (values, indices.validity().cloned())
    }
}

/// `take` implementation for primitive arrays
pub fn take<T: NativeType, I: Index>(
    values: &PrimitiveArray<T>,
    indices: &PrimitiveArray<I>,
) -> PrimitiveArray<T> {
    let indices_has_validity = indices.null_count() > 0;
    let values_has_validity = values.null_count() > 0;
    let (buffer, validity) = match (values_has_validity, indices_has_validity) {
        (false, false) => take_no_validity::<T, I>(values.values(), indices.values()),
        (true, false) => take_values_validity::<T, I>(values, indices.values()),
        (false, true) => take_indices_validity::<T, I>(values.values(), indices),
        (true, true) => take_values_indices_validity::<T, I>(values, indices),
    };

    PrimitiveArray::<T>::new(values.data_type().clone(), buffer, validity)
}
