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

use std::collections::VecDeque;

use parquet2::encoding::Encoding;
use parquet2::page::split_buffer;
use parquet2::page::DataPage;
use parquet2::page::DictPage;
use parquet2::schema::Repetition;

use super::super::nested_utils::*;
use super::super::utils;
use super::super::utils::MaybeNext;
use super::basic::deserialize_plain;
use super::basic::finish;
use super::basic::BinaryDict;
use super::basic::ValuesDictionary;
use super::utils::*;
use crate::arrow::array::Array;
use crate::arrow::bitmap::MutableBitmap;
use crate::arrow::datatypes::DataType;
use crate::arrow::error::Result;
use crate::arrow::io::parquet::read::Pages;
use crate::arrow::offset::Offset;

#[derive(Debug)]
pub(crate) enum BinaryNestedState<'a> {
    Optional(BinaryIter<'a>),
    Required(BinaryIter<'a>),
    RequiredDictionary(ValuesDictionary<'a>),
    OptionalDictionary(ValuesDictionary<'a>),
}

impl<'a> utils::PageState<'a> for BinaryNestedState<'a> {
    fn len(&self) -> usize {
        match self {
            BinaryNestedState::Optional(validity) => validity.size_hint().0,
            BinaryNestedState::Required(state) => state.size_hint().0,
            BinaryNestedState::RequiredDictionary(required) => required.len(),
            BinaryNestedState::OptionalDictionary(optional) => optional.len(),
        }
    }
}

#[derive(Debug, Default)]
struct BinaryDecoder<O: Offset> {
    phantom_o: std::marker::PhantomData<O>,
}

impl<'a, O: Offset> NestedDecoder<'a> for BinaryDecoder<O> {
    type State = BinaryNestedState<'a>;
    type Dictionary = BinaryDict;
    type DecodedState = (Binary<O>, MutableBitmap);

    fn build_state(
        &self,
        page: &'a DataPage,
        dict: Option<&'a Self::Dictionary>,
    ) -> Result<Self::State> {
        build_nested_state(page, dict)
    }

    fn with_capacity(&self, capacity: usize) -> Self::DecodedState {
        (
            Binary::<O>::with_capacity(capacity),
            MutableBitmap::with_capacity(capacity),
        )
    }

    fn push_valid(&self, state: &mut Self::State, decoded: &mut Self::DecodedState) -> Result<()> {
        let (values, validity) = decoded;
        match state {
            BinaryNestedState::Optional(page) => {
                let value = page.next().unwrap_or_default();
                values.push(value);
                validity.push(true);
            }
            BinaryNestedState::Required(page) => {
                let value = page.next().unwrap_or_default();
                values.push(value);
            }
            BinaryNestedState::RequiredDictionary(page) => {
                let dict_values = &page.dict;
                let item = page
                    .values
                    .next()
                    .map(|index| dict_values[index.unwrap() as usize].as_ref())
                    .unwrap_or_default();
                values.push(item);
            }
            BinaryNestedState::OptionalDictionary(page) => {
                let dict_values = &page.dict;
                let item = page
                    .values
                    .next()
                    .map(|index| dict_values[index.unwrap() as usize].as_ref())
                    .unwrap_or_default();
                values.push(item);
                validity.push(true);
            }
        }
        Ok(())
    }

    fn push_null(&self, decoded: &mut Self::DecodedState) {
        let (values, validity) = decoded;
        values.push(&[]);
        validity.push(false);
    }

    fn deserialize_dict(&self, page: &DictPage) -> Self::Dictionary {
        deserialize_plain(&page.buffer, page.num_values)
    }
}

pub(crate) fn build_nested_state<'a>(
    page: &'a DataPage,
    dict: Option<&'a BinaryDict>,
) -> Result<BinaryNestedState<'a>> {
    let is_optional = page.descriptor.primitive_type.field_info.repetition == Repetition::Optional;
    let is_filtered = page.selected_rows().is_some();

    match (page.encoding(), dict, is_optional, is_filtered) {
        (Encoding::PlainDictionary | Encoding::RleDictionary, Some(dict), false, false) => {
            ValuesDictionary::try_new(page, dict).map(BinaryNestedState::RequiredDictionary)
        }
        (Encoding::PlainDictionary | Encoding::RleDictionary, Some(dict), true, false) => {
            ValuesDictionary::try_new(page, dict).map(BinaryNestedState::OptionalDictionary)
        }
        (Encoding::Plain, _, true, false) => {
            let (_, _, values) = split_buffer(page)?;

            let values = BinaryIter::new(values);

            Ok(BinaryNestedState::Optional(values))
        }
        (Encoding::Plain, _, false, false) => {
            let (_, _, values) = split_buffer(page)?;

            let values = BinaryIter::new(values);

            Ok(BinaryNestedState::Required(values))
        }
        _ => Err(utils::not_implemented(page)),
    }
}

pub struct NestedIter<O: Offset, I: Pages> {
    iter: I,
    data_type: DataType,
    init: Vec<InitNested>,
    items: VecDeque<(NestedState, (Binary<O>, MutableBitmap))>,
    dict: Option<BinaryDict>,
    chunk_size: Option<usize>,
    remaining: usize,
}

impl<O: Offset, I: Pages> NestedIter<O, I> {
    pub fn new(
        iter: I,
        init: Vec<InitNested>,
        data_type: DataType,
        num_rows: usize,
        chunk_size: Option<usize>,
    ) -> Self {
        Self {
            iter,
            data_type,
            init,
            items: VecDeque::new(),
            dict: None,
            chunk_size,
            remaining: num_rows,
        }
    }
}

impl<O: Offset, I: Pages> Iterator for NestedIter<O, I> {
    type Item = Result<(NestedState, Box<dyn Array>)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let maybe_state = next(
                &mut self.iter,
                &mut self.items,
                &mut self.dict,
                &mut self.remaining,
                &self.init,
                self.chunk_size,
                &BinaryDecoder::<O>::default(),
            );
            match maybe_state {
                MaybeNext::Some(Ok((nested, decoded))) => {
                    return Some(
                        finish(&self.data_type, decoded.0, decoded.1).map(|array| (nested, array)),
                    );
                }
                MaybeNext::Some(Err(e)) => return Some(Err(e)),
                MaybeNext::None => return None,
                MaybeNext::More => continue, /* Using continue in a loop instead of calling next helps prevent stack overflow. */
            }
        }
    }
}
