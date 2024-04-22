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

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;

use databend_common_base::base::GlobalUniqName;
use databend_common_base::base::ProgressValues;
use databend_common_catalog::table_context::TableContext;
use databend_common_exception::Result;
use databend_common_expression::arrow::deserialize_column;
use databend_common_expression::arrow::serialize_column;
use databend_common_expression::DataBlock;
use opendal::Operator;

use crate::sessions::QueryContext;

/// Spiller type, currently only supports HashJoin
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpillerType {
    HashJoinBuild,
    HashJoinProbe,
    OrderBy, /* Todo: Add more spillers type
              * Aggregation */
}

impl Display for SpillerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpillerType::HashJoinBuild => write!(f, "HashJoinBuild"),
            SpillerType::HashJoinProbe => write!(f, "HashJoinProbe"),
            SpillerType::OrderBy => write!(f, "OrderBy"),
        }
    }
}

/// Spiller configuration
#[derive(Clone)]
pub struct SpillerConfig {
    pub location_prefix: String,
}

impl SpillerConfig {
    pub fn create(location_prefix: String) -> Self {
        Self { location_prefix }
    }
}

/// Spiller is a unified framework for operators which need to spill data from memory.
/// It provides the following features:
/// 1. Collection data that needs to be spilled.
/// 2. Partition data by the specified algorithm which specifies by operator
/// 3. Serialization and deserialization input data
/// 4. Interact with the underlying storage engine to write and read spilled data
#[derive(Clone)]
pub struct Spiller {
    ctx: Arc<QueryContext>,
    operator: Operator,
    config: SpillerConfig,
    _spiller_type: SpillerType,
    pub join_spilling_partition_bits: usize,
    /// 1 partition -> N partition files
    pub partition_location: HashMap<u8, Vec<String>>,
    /// Record columns layout for spilled data, will be used when read data from disk
    pub columns_layout: HashMap<String, Vec<usize>>,
}

impl Spiller {
    /// Create a new spiller
    pub fn create(
        ctx: Arc<QueryContext>,
        operator: Operator,
        config: SpillerConfig,
        spiller_type: SpillerType,
    ) -> Result<Self> {
        let join_spilling_partition_bits = ctx.get_settings().get_join_spilling_partition_bits()?;
        Ok(Self {
            ctx,
            operator,
            config,
            _spiller_type: spiller_type,
            join_spilling_partition_bits,
            partition_location: Default::default(),
            columns_layout: Default::default(),
        })
    }

    pub fn spilled_partitions(&self) -> HashSet<u8> {
        self.partition_location.keys().copied().collect()
    }

    /// Read a certain file to a [`DataBlock`].
    /// We should guarantee that the file is managed by this spiller.
    pub async fn read_spilled(&self, file: &str) -> Result<(DataBlock, u64)> {
        debug_assert!(self.columns_layout.contains_key(file));
        let data = self.operator.read(file).await?;
        let bytes = data.len() as u64;

        let mut begin = 0;
        let mut columns = Vec::with_capacity(self.columns_layout.len());
        let columns_layout = self.columns_layout.get(file).unwrap();
        for column_layout in columns_layout.iter() {
            columns.push(deserialize_column(&data[begin..begin + column_layout]).unwrap());
            begin += column_layout;
        }
        let block = DataBlock::new_from_columns(columns);
        Ok((block, bytes))
    }

    /// Write a [`DataBlock`] to storage.
    pub async fn spill_block(&mut self, data: DataBlock) -> Result<(String, u64)> {
        let unique_name = GlobalUniqName::unique();
        let location = format!("{}/{}", self.config.location_prefix, unique_name);
        let mut write_bytes = 0;

        let mut writer = self
            .operator
            .writer_with(&location)
            .buffer(8 * 1024 * 1024)
            .await?;
        let columns = data.columns().to_vec();
        let mut columns_data = Vec::with_capacity(columns.len());
        for column in columns.into_iter() {
            let column = column.value.as_column().unwrap();
            let column_data = serialize_column(column);
            self.columns_layout
                .entry(location.to_string())
                .and_modify(|layouts| {
                    layouts.push(column_data.len());
                })
                .or_insert(vec![column_data.len()]);
            write_bytes += column_data.len() as u64;
            columns_data.push(column_data);
        }

        for data in columns_data.into_iter() {
            writer.write(data).await?;
        }
        writer.close().await?;

        Ok((location, write_bytes))
    }

    #[async_backtrace::framed]
    /// Spill data block with location
    pub async fn spill_with_partition(&mut self, p_id: u8, data: DataBlock) -> Result<()> {
        let progress_val = ProgressValues {
            rows: data.num_rows(),
            bytes: data.memory_size(),
        };

        let (location, _) = self.spill_block(data).await?;
        self.partition_location
            .entry(p_id)
            .and_modify(|locs| {
                locs.push(location.clone());
            })
            .or_insert(vec![location.clone()]);

        self.ctx.get_join_spill_progress().incr(&progress_val);
        Ok(())
    }

    #[async_backtrace::framed]
    /// Read spilled data with partition id
    pub async fn read_spilled_data(&self, p_id: &u8) -> Result<Vec<DataBlock>> {
        debug_assert!(self.partition_location.contains_key(p_id));
        let files = self.partition_location.get(p_id).unwrap();
        let mut spilled_data = Vec::with_capacity(files.len());
        for file in files.iter() {
            let (block, _) = self.read_spilled(file).await?;
            if block.num_rows() != 0 {
                spilled_data.push(block);
            }
        }
        Ok(spilled_data)
    }

    #[async_backtrace::framed]
    // Directly spill input data without buffering.
    // Need to compute hashes for data block advanced.
    // For probe, only need to spill rows in build spilled partitions.
    pub(crate) async fn spill_input(
        &mut self,
        data_block: DataBlock,
        hashes: &[u64],
        spilled_partitions: Option<&HashSet<u8>>,
    ) -> Result<()> {
        // Key is partition, value is row indexes
        let mut partition_rows = HashMap::new();
        // Classify rows to spill or not spill.
        for (row_idx, hash) in hashes.iter().enumerate() {
            let partition_id =
                get_partition_id(*hash as usize, self.join_spilling_partition_bits) as u8;
            if let Some(spilled_partitions) = spilled_partitions {
                if !spilled_partitions.contains(&partition_id) {
                    continue;
                }
            }
            // the row can be directly spilled to corresponding partition
            partition_rows
                .entry(partition_id)
                .and_modify(|v: &mut Vec<usize>| v.push(row_idx))
                .or_insert(vec![row_idx]);
        }
        for (p_id, row_indexes) in partition_rows.iter() {
            let block_row_indexes = row_indexes
                .iter()
                .map(|idx| (0_u32, *idx as u32, 1_usize))
                .collect::<Vec<_>>();
            let block = DataBlock::take_blocks(
                &[data_block.clone()],
                &block_row_indexes,
                row_indexes.len(),
            );
            // Spill block with partition id
            self.spill_with_partition(*p_id, block).await?;
        }
        Ok(())
    }
}

fn get_partition_id(hash: usize, bits: usize) -> usize {
    (hash >> (32 - bits)) & ((1 << bits) - 1)
}
