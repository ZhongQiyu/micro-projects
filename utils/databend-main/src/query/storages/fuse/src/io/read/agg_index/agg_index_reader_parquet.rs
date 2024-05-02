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

use std::sync::Arc;

use databend_common_arrow::arrow::io::parquet::read as pread;
use databend_common_catalog::plan::PartInfoPtr;
use databend_common_exception::Result;
use databend_common_expression::DataBlock;
use log::debug;

use super::AggIndexReader;
use crate::io::read::utils::build_columns_meta;
use crate::io::ReadSettings;
use crate::io::UncompressedBuffer;
use crate::FusePartInfo;
use crate::MergeIOReadResult;

impl AggIndexReader {
    pub fn sync_read_parquet_data_by_merge_io(
        &self,
        read_settings: &ReadSettings,
        loc: &str,
    ) -> Option<(PartInfoPtr, MergeIOReadResult)> {
        match self.reader.operator.blocking().reader(loc) {
            Ok(mut reader) => {
                let metadata = pread::read_metadata(&mut reader)
                    .inspect_err(|e| {
                        debug!("Read aggregating index `{loc}`'s metadata failed: {e}")
                    })
                    .ok()?;
                debug_assert_eq!(metadata.row_groups.len(), 1);
                let row_group = &metadata.row_groups[0];
                let columns_meta = build_columns_meta(row_group);
                let part = FusePartInfo::create(
                    loc.to_string(),
                    row_group.num_rows() as u64,
                    columns_meta,
                    None,
                    self.compression.into(),
                    None,
                    None,
                    None,
                );
                let res = self
                    .reader
                    .sync_read_columns_data_by_merge_io(read_settings, &part, &None)
                    .inspect_err(|e| debug!("Read aggregating index `{loc}` failed: {e}"))
                    .ok()?;
                Some((part, res))
            }
            Err(e) => {
                if e.kind() == opendal::ErrorKind::NotFound {
                    debug!("Aggregating index `{loc}` not found.")
                } else {
                    debug!("Read aggregating index `{loc}` failed: {e}");
                }
                None
            }
        }
    }

    pub async fn read_parquet_data_by_merge_io(
        &self,
        read_settings: &ReadSettings,
        loc: &str,
    ) -> Option<(PartInfoPtr, MergeIOReadResult)> {
        match self.reader.operator.reader(loc).await {
            Ok(mut reader) => {
                let metadata = pread::read_metadata_async(&mut reader)
                    .await
                    .inspect_err(|e| {
                        debug!("Read aggregating index `{loc}`'s metadata failed: {e}")
                    })
                    .ok()?;
                debug_assert_eq!(metadata.row_groups.len(), 1);
                let row_group = &metadata.row_groups[0];
                let columns_meta = build_columns_meta(row_group);
                let res = self
                    .reader
                    .read_columns_data_by_merge_io(read_settings, loc, &columns_meta, &None)
                    .await
                    .inspect_err(|e| debug!("Read aggregating index `{loc}` failed: {e}"))
                    .ok()?;
                let part = FusePartInfo::create(
                    loc.to_string(),
                    row_group.num_rows() as u64,
                    columns_meta,
                    None,
                    self.compression.into(),
                    None,
                    None,
                    None,
                );
                Some((part, res))
            }
            Err(e) => {
                if e.kind() == opendal::ErrorKind::NotFound {
                    debug!("Aggregating index `{loc}` not found.")
                } else {
                    debug!("Read aggregating index `{loc}` failed: {e}");
                }
                None
            }
        }
    }

    pub fn deserialize_parquet_data(
        &self,
        part: PartInfoPtr,
        data: MergeIOReadResult,
        buffer: Arc<UncompressedBuffer>,
    ) -> Result<DataBlock> {
        let columns_chunks = data.columns_chunks()?;
        let part = FusePartInfo::from_part(&part)?;
        let block = self.reader.deserialize_parquet_chunks_with_buffer(
            &part.location,
            part.nums_rows,
            &part.compression,
            &part.columns_meta,
            columns_chunks,
            Some(buffer),
        )?;

        self.apply_agg_info(block)
    }
}
