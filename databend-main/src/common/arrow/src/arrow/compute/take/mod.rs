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

//! Defines take kernel for [`Array`]

use crate::arrow::array::new_empty_array;
use crate::arrow::array::Array;
use crate::arrow::array::NullArray;
use crate::arrow::array::PrimitiveArray;
use crate::arrow::array::Utf8ViewArray;
use crate::arrow::compute::take::binview::take_binview_unchecked;
use crate::arrow::datatypes::DataType;
use crate::arrow::error::Result;
use crate::arrow::types::Index;

mod binary;
mod binview;
mod boolean;
mod dict;
mod fixed_size_list;
mod generic_binary;
mod list;
mod primitive;
mod structure;
mod utf8;

/// Returns a new [`Array`] with only indices at `indices`. Null indices are taken as nulls.
/// The returned array has a length equal to `indices.len()`.
pub fn take<O: Index>(values: &dyn Array, indices: &PrimitiveArray<O>) -> Result<Box<dyn Array>> {
    if indices.is_empty() {
        return Ok(new_empty_array(values.data_type().clone()));
    }

    use crate::arrow::datatypes::PhysicalType::*;
    match values.data_type().to_physical_type() {
        Null => Ok(Box::new(NullArray::new(
            values.data_type().clone(),
            indices.len(),
        ))),
        Boolean => {
            let values = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(boolean::take::<O>(values, indices)))
        }
        Primitive(primitive) => with_match_primitive_type!(primitive, |$T| {
            let values = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(primitive::take::<$T, _>(&values, indices)))
        }),
        Utf8 => {
            let values = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(utf8::take::<i32, _>(values, indices)))
        }
        LargeUtf8 => {
            let values = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(utf8::take::<i64, _>(values, indices)))
        }
        Binary => {
            let values = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(binary::take::<i32, _>(values, indices)))
        }
        LargeBinary => {
            let values = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(binary::take::<i64, _>(values, indices)))
        }
        Dictionary(key_type) => {
            match_integer_type!(key_type, |$T| {
                let values = values.as_any().downcast_ref().unwrap();
                Ok(Box::new(dict::take::<$T, _>(&values, indices)))
            })
        }
        Struct => {
            let array = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(structure::take::<_>(array, indices)?))
        }
        List => {
            let array = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(list::take::<i32, O>(array, indices)))
        }
        LargeList => {
            let array = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(list::take::<i64, O>(array, indices)))
        }
        FixedSizeList => {
            let array = values.as_any().downcast_ref().unwrap();
            Ok(Box::new(fixed_size_list::take::<O>(array, indices)))
        }
        BinaryView => unsafe {
            Ok(take_binview_unchecked(values.as_any().downcast_ref().unwrap(), indices).boxed())
        },
        Utf8View => unsafe {
            let arr: &Utf8ViewArray = values.as_any().downcast_ref().unwrap();
            Ok(take_binview_unchecked(&arr.to_binview(), indices)
                .to_utf8view_unchecked()
                .boxed())
        },
        t => unimplemented!("Take not supported for data type {:?}", t),
    }
}

/// Checks if an array of type `datatype` can perform take operation
///
/// # Examples
/// ```
/// use arrow2::compute::take::can_take;
/// use arrow2::datatypes::DataType;
///
/// let data_type = DataType::Int8;
/// assert_eq!(can_take(&data_type), true);
/// ```
pub fn can_take(data_type: &DataType) -> bool {
    matches!(
        data_type,
        DataType::Null
            | DataType::Boolean
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Date32
            | DataType::Time32(_)
            | DataType::Interval(_)
            | DataType::Int64
            | DataType::Date64
            | DataType::Time64(_)
            | DataType::Duration(_)
            | DataType::Timestamp(_, _)
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float16
            | DataType::Float32
            | DataType::Float64
            | DataType::Decimal(_, _)
            | DataType::Utf8
            | DataType::LargeUtf8
            | DataType::Binary
            | DataType::LargeBinary
            | DataType::Struct(_)
            | DataType::List(_)
            | DataType::LargeList(_)
            | DataType::FixedSizeList(_, _)
            | DataType::Dictionary(..)
    )
}
