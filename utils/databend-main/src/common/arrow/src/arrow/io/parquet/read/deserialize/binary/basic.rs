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
use std::default::Default;

use parquet2::deserialize::SliceFilteredIter;
use parquet2::encoding::delta_length_byte_array;
use parquet2::encoding::hybrid_rle;
use parquet2::encoding::Encoding;
use parquet2::page::split_buffer;
use parquet2::page::DataPage;
use parquet2::page::DictPage;
use parquet2::schema::Repetition;

use super::super::utils;
use super::super::utils::extend_from_decoder;
use super::super::utils::get_selected_rows;
use super::super::utils::next;
use super::super::utils::DecodedState;
use super::super::utils::FilteredOptionalPageValidity;
use super::super::utils::MaybeNext;
use super::super::utils::OptionalPageValidity;
use super::super::Pages;
use super::utils::*;
use crate::arrow::array::Array;
use crate::arrow::array::BinaryArray;
use crate::arrow::array::Utf8Array;
use crate::arrow::bitmap::MutableBitmap;
use crate::arrow::datatypes::DataType;
use crate::arrow::datatypes::PhysicalType;
use crate::arrow::error::Error;
use crate::arrow::error::Result;
use crate::arrow::offset::Offset;

#[derive(Debug)]
pub(crate) struct Required<'a> {
    pub values: SizedBinaryIter<'a>,
}

impl<'a> Required<'a> {
    pub fn try_new(page: &'a DataPage) -> Result<Self> {
        let (_, _, values) = split_buffer(page)?;
        let values = SizedBinaryIter::new(values, page.num_values());

        Ok(Self { values })
    }

    pub fn len(&self) -> usize {
        self.values.size_hint().0
    }
}

#[derive(Debug)]
pub(crate) struct Delta<'a> {
    pub lengths: std::vec::IntoIter<usize>,
    pub values: &'a [u8],
}

impl<'a> Delta<'a> {
    pub fn try_new(page: &'a DataPage) -> Result<Self> {
        let (_, _, values) = split_buffer(page)?;

        let mut lengths_iter = delta_length_byte_array::Decoder::try_new(values)?;

        #[allow(clippy::needless_collect)] // we need to consume it to get the values
        let lengths = lengths_iter
            .by_ref()
            .map(|x| x.map(|x| x as usize).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;

        let values = lengths_iter.into_values();
        Ok(Self {
            lengths: lengths.into_iter(),
            values,
        })
    }

    pub fn len(&self) -> usize {
        self.lengths.size_hint().0
    }
}

impl<'a> Iterator for Delta<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let length = self.lengths.next()?;
        let (item, remaining) = self.values.split_at(length);
        self.values = remaining;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lengths.size_hint()
    }
}

#[derive(Debug)]
pub(crate) struct FilteredRequired<'a> {
    pub values: SliceFilteredIter<SizedBinaryIter<'a>>,
}

impl<'a> FilteredRequired<'a> {
    pub fn new(page: &'a DataPage) -> Self {
        let values = SizedBinaryIter::new(page.buffer(), page.num_values());

        let rows = get_selected_rows(page);
        let values = SliceFilteredIter::new(values, rows);

        Self { values }
    }

    pub fn len(&self) -> usize {
        self.values.size_hint().0
    }
}

#[derive(Debug)]
pub(crate) struct FilteredDelta<'a> {
    pub values: SliceFilteredIter<Delta<'a>>,
}

impl<'a> FilteredDelta<'a> {
    pub fn try_new(page: &'a DataPage) -> Result<Self> {
        let values = Delta::try_new(page)?;

        let rows = get_selected_rows(page);
        let values = SliceFilteredIter::new(values, rows);

        Ok(Self { values })
    }

    pub fn len(&self) -> usize {
        self.values.size_hint().0
    }
}

pub(crate) type BinaryDict = Vec<Vec<u8>>;

#[derive(Debug)]
pub(crate) struct RequiredDictionary<'a> {
    pub values: hybrid_rle::HybridRleDecoder<'a>,
    pub dict: &'a BinaryDict,
}

impl<'a> RequiredDictionary<'a> {
    pub fn try_new(page: &'a DataPage, dict: &'a BinaryDict) -> Result<Self> {
        let values = utils::dict_indices_decoder(page)?;

        Ok(Self { dict, values })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.size_hint().0
    }
}

#[derive(Debug)]
pub(crate) struct FilteredRequiredDictionary<'a> {
    pub values: SliceFilteredIter<hybrid_rle::HybridRleDecoder<'a>>,
    pub dict: &'a BinaryDict,
}

impl<'a> FilteredRequiredDictionary<'a> {
    pub fn try_new(page: &'a DataPage, dict: &'a BinaryDict) -> Result<Self> {
        let values = utils::dict_indices_decoder(page)?;

        let rows = get_selected_rows(page);
        let values = SliceFilteredIter::new(values, rows);

        Ok(Self { values, dict })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.size_hint().0
    }
}

#[derive(Debug)]
pub(crate) struct ValuesDictionary<'a> {
    pub values: hybrid_rle::HybridRleDecoder<'a>,
    pub dict: &'a BinaryDict,
}

impl<'a> ValuesDictionary<'a> {
    pub fn try_new(page: &'a DataPage, dict: &'a BinaryDict) -> Result<Self> {
        let values = utils::dict_indices_decoder(page)?;

        Ok(Self { dict, values })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.size_hint().0
    }
}

#[derive(Debug)]
pub(crate) enum BinaryState<'a> {
    Optional(OptionalPageValidity<'a>, BinaryIter<'a>),
    Required(Required<'a>),
    RequiredDictionary(RequiredDictionary<'a>),
    OptionalDictionary(OptionalPageValidity<'a>, ValuesDictionary<'a>),
    Delta(Delta<'a>),
    OptionalDelta(OptionalPageValidity<'a>, Delta<'a>),
    FilteredRequired(FilteredRequired<'a>),
    FilteredDelta(FilteredDelta<'a>),
    FilteredOptionalDelta(FilteredOptionalPageValidity<'a>, Delta<'a>),
    FilteredOptional(FilteredOptionalPageValidity<'a>, BinaryIter<'a>),
    FilteredRequiredDictionary(FilteredRequiredDictionary<'a>),
    FilteredOptionalDictionary(FilteredOptionalPageValidity<'a>, ValuesDictionary<'a>),
}

impl<'a> utils::PageState<'a> for BinaryState<'a> {
    fn len(&self) -> usize {
        match self {
            BinaryState::Optional(validity, _) => validity.len(),
            BinaryState::Required(state) => state.len(),
            BinaryState::Delta(state) => state.len(),
            BinaryState::OptionalDelta(state, _) => state.len(),
            BinaryState::RequiredDictionary(values) => values.len(),
            BinaryState::OptionalDictionary(optional, _) => optional.len(),
            BinaryState::FilteredRequired(state) => state.len(),
            BinaryState::FilteredOptional(validity, _) => validity.len(),
            BinaryState::FilteredDelta(state) => state.len(),
            BinaryState::FilteredOptionalDelta(state, _) => state.len(),
            BinaryState::FilteredRequiredDictionary(values) => values.len(),
            BinaryState::FilteredOptionalDictionary(optional, _) => optional.len(),
        }
    }
}

impl<O: Offset> DecodedState for (Binary<O>, MutableBitmap) {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Default)]
struct BinaryDecoder<O: Offset> {
    phantom_o: std::marker::PhantomData<O>,
}

impl<'a, O: Offset> utils::Decoder<'a> for BinaryDecoder<O> {
    type State = BinaryState<'a>;
    type Dict = BinaryDict;
    type DecodedState = (Binary<O>, MutableBitmap);

    fn build_state(&self, page: &'a DataPage, dict: Option<&'a Self::Dict>) -> Result<Self::State> {
        build_binary_state(page, dict)
    }

    fn with_capacity(&self, capacity: usize) -> Self::DecodedState {
        (
            Binary::<O>::with_capacity(capacity),
            MutableBitmap::with_capacity(capacity),
        )
    }

    fn extend_from_state(
        &self,
        state: &mut Self::State,
        decoded: &mut Self::DecodedState,
        additional: usize,
    ) {
        let (values, validity) = decoded;
        match state {
            BinaryState::Optional(page_validity, page_values) => extend_from_decoder(
                validity,
                page_validity,
                Some(additional),
                values,
                page_values,
            ),
            BinaryState::Required(page) => {
                for x in page.values.by_ref().take(additional) {
                    values.push(x)
                }
            }
            BinaryState::Delta(page) => {
                values.extend_lengths(page.lengths.by_ref().take(additional), &mut page.values);
            }
            BinaryState::OptionalDelta(page_validity, page_values) => {
                let Binary {
                    offsets,
                    values: values_,
                } = values;

                let last_offset = *offsets.last();
                extend_from_decoder(
                    validity,
                    page_validity,
                    Some(additional),
                    offsets,
                    page_values.lengths.by_ref(),
                );

                let length = *offsets.last() - last_offset;

                let (consumed, remaining) = page_values.values.split_at(length.to_usize());
                page_values.values = remaining;
                values_.extend_from_slice(consumed);
            }
            BinaryState::FilteredRequired(page) => {
                for x in page.values.by_ref().take(additional) {
                    values.push(x)
                }
            }
            BinaryState::FilteredDelta(page) => {
                for x in page.values.by_ref().take(additional) {
                    values.push(x)
                }
            }
            BinaryState::OptionalDictionary(page_validity, page_values) => {
                let page_dict = &page_values.dict;
                utils::extend_from_decoder(
                    validity,
                    page_validity,
                    Some(additional),
                    values,
                    &mut page_values
                        .values
                        .by_ref()
                        .map(|index| page_dict[index.unwrap() as usize].as_ref()),
                )
            }
            BinaryState::RequiredDictionary(page) => {
                let page_dict = &page.dict;

                for x in page
                    .values
                    .by_ref()
                    .map(|index| page_dict[index.unwrap() as usize].as_ref())
                    .take(additional)
                {
                    values.push(x)
                }
            }
            BinaryState::FilteredOptional(page_validity, page_values) => {
                utils::extend_from_decoder(
                    validity,
                    page_validity,
                    Some(additional),
                    values,
                    page_values.by_ref(),
                );
            }
            BinaryState::FilteredOptionalDelta(page_validity, page_values) => {
                utils::extend_from_decoder(
                    validity,
                    page_validity,
                    Some(additional),
                    values,
                    page_values.by_ref(),
                );
            }
            BinaryState::FilteredRequiredDictionary(page) => {
                let page_dict = &page.dict;
                for x in page
                    .values
                    .by_ref()
                    .map(|index| page_dict[index.unwrap() as usize].as_ref())
                    .take(additional)
                {
                    values.push(x)
                }
            }
            BinaryState::FilteredOptionalDictionary(page_validity, page_values) => {
                let page_dict = &page_values.dict;
                utils::extend_from_decoder(
                    validity,
                    page_validity,
                    Some(additional),
                    values,
                    &mut page_values
                        .values
                        .by_ref()
                        .map(|index| page_dict[index.unwrap() as usize].as_ref()),
                )
            }
        }
    }

    fn deserialize_dict(&self, page: &DictPage) -> Self::Dict {
        deserialize_plain(&page.buffer, page.num_values)
    }
}

pub(super) fn finish<O: Offset>(
    data_type: &DataType,
    mut values: Binary<O>,
    mut validity: MutableBitmap,
) -> Result<Box<dyn Array>> {
    values.offsets.shrink_to_fit();
    values.values.shrink_to_fit();
    validity.shrink_to_fit();

    match data_type.to_physical_type() {
        PhysicalType::Binary | PhysicalType::LargeBinary => BinaryArray::<O>::try_new(
            data_type.clone(),
            values.offsets.into(),
            values.values.into(),
            validity.into(),
        )
        .map(|x| x.boxed()),
        PhysicalType::Utf8 | PhysicalType::LargeUtf8 => Utf8Array::<O>::try_new(
            data_type.clone(),
            values.offsets.into(),
            values.values.into(),
            validity.into(),
        )
        .map(|x| x.boxed()),
        _ => unreachable!(),
    }
}

pub struct Iter<O: Offset, I: Pages> {
    iter: I,
    data_type: DataType,
    items: VecDeque<(Binary<O>, MutableBitmap)>,
    dict: Option<BinaryDict>,
    chunk_size: Option<usize>,
    remaining: usize,
}

impl<O: Offset, I: Pages> Iter<O, I> {
    pub fn new(iter: I, data_type: DataType, chunk_size: Option<usize>, num_rows: usize) -> Self {
        Self {
            iter,
            data_type,
            items: VecDeque::new(),
            dict: None,
            chunk_size,
            remaining: num_rows,
        }
    }
}

impl<O: Offset, I: Pages> Iterator for Iter<O, I> {
    type Item = Result<Box<dyn Array>>;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_state = next(
            &mut self.iter,
            &mut self.items,
            &mut self.dict,
            &mut self.remaining,
            self.chunk_size,
            &BinaryDecoder::<O>::default(),
        );
        match maybe_state {
            MaybeNext::Some(Ok((values, validity))) => {
                Some(finish(&self.data_type, values, validity))
            }
            MaybeNext::Some(Err(e)) => Some(Err(e)),
            MaybeNext::None => None,
            MaybeNext::More => self.next(),
        }
    }
}

pub(crate) fn deserialize_plain(values: &[u8], num_values: usize) -> BinaryDict {
    SizedBinaryIter::new(values, num_values)
        .map(|x| x.to_vec())
        .collect()
}

pub(crate) fn build_binary_state<'a>(
    page: &'a DataPage,
    dict: Option<&'a BinaryDict>,
) -> Result<BinaryState<'a>> {
    let is_optional = page.descriptor.primitive_type.field_info.repetition == Repetition::Optional;
    let is_filtered = page.selected_rows().is_some();

    match (page.encoding(), dict, is_optional, is_filtered) {
        (Encoding::PlainDictionary | Encoding::RleDictionary, Some(dict), false, false) => Ok(
            BinaryState::RequiredDictionary(RequiredDictionary::try_new(page, dict)?),
        ),
        (Encoding::PlainDictionary | Encoding::RleDictionary, Some(dict), true, false) => {
            Ok(BinaryState::OptionalDictionary(
                OptionalPageValidity::try_new(page)?,
                ValuesDictionary::try_new(page, dict)?,
            ))
        }
        (Encoding::PlainDictionary | Encoding::RleDictionary, Some(dict), false, true) => {
            FilteredRequiredDictionary::try_new(page, dict)
                .map(BinaryState::FilteredRequiredDictionary)
        }
        (Encoding::PlainDictionary | Encoding::RleDictionary, Some(dict), true, true) => {
            Ok(BinaryState::FilteredOptionalDictionary(
                FilteredOptionalPageValidity::try_new(page)?,
                ValuesDictionary::try_new(page, dict)?,
            ))
        }
        (Encoding::Plain, _, true, false) => {
            let (_, _, values) = split_buffer(page)?;

            let values = BinaryIter::new(values);

            Ok(BinaryState::Optional(
                OptionalPageValidity::try_new(page)?,
                values,
            ))
        }
        (Encoding::Plain, _, false, false) => Ok(BinaryState::Required(Required::try_new(page)?)),
        (Encoding::Plain, _, false, true) => {
            Ok(BinaryState::FilteredRequired(FilteredRequired::new(page)))
        }
        (Encoding::Plain, _, true, true) => {
            let (_, _, values) = split_buffer(page)?;

            Ok(BinaryState::FilteredOptional(
                FilteredOptionalPageValidity::try_new(page)?,
                BinaryIter::new(values),
            ))
        }
        (Encoding::DeltaLengthByteArray, _, false, false) => {
            Delta::try_new(page).map(BinaryState::Delta)
        }
        (Encoding::DeltaLengthByteArray, _, true, false) => Ok(BinaryState::OptionalDelta(
            OptionalPageValidity::try_new(page)?,
            Delta::try_new(page)?,
        )),
        (Encoding::DeltaLengthByteArray, _, false, true) => {
            FilteredDelta::try_new(page).map(BinaryState::FilteredDelta)
        }
        (Encoding::DeltaLengthByteArray, _, true, true) => Ok(BinaryState::FilteredOptionalDelta(
            FilteredOptionalPageValidity::try_new(page)?,
            Delta::try_new(page)?,
        )),
        _ => Err(utils::not_implemented(page)),
    }
}
