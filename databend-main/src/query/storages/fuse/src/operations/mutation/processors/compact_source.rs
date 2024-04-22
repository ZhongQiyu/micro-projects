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

use std::any::Any;
use std::sync::Arc;
use std::time::Instant;

use databend_common_base::base::ProgressValues;
use databend_common_catalog::plan::gen_mutation_stream_meta;
use databend_common_catalog::plan::PartInfoPtr;
use databend_common_catalog::plan::StreamColumn;
use databend_common_catalog::table_context::TableContext;
use databend_common_exception::ErrorCode;
use databend_common_exception::Result;
use databend_common_expression::DataBlock;
use databend_common_expression::FunctionContext;
use databend_common_metrics::storage::*;
use databend_common_pipeline_core::processors::Event;
use databend_common_pipeline_core::processors::OutputPort;
use databend_common_pipeline_core::processors::Processor;
use databend_common_pipeline_core::processors::ProcessorPtr;
use databend_common_sql::evaluator::BlockOperator;
use databend_storages_common_table_meta::meta::BlockMeta;

use crate::io::BlockReader;
use crate::io::ReadSettings;
use crate::operations::mutation::ClusterStatsGenType;
use crate::operations::mutation::CompactPartInfo;
use crate::operations::mutation::SerializeBlock;
use crate::operations::mutation::SerializeDataMeta;
use crate::operations::BlockMetaIndex;
use crate::FuseStorageFormat;
use crate::MergeIOReadResult;

enum State {
    ReadData(Option<PartInfoPtr>),
    Concat {
        read_res: Vec<MergeIOReadResult>,
        metas: Vec<Arc<BlockMeta>>,
        index: BlockMetaIndex,
    },
    Output(Option<PartInfoPtr>, DataBlock),
    Finish,
}

pub struct CompactSource {
    state: State,
    ctx: Arc<dyn TableContext>,
    func_ctx: FunctionContext,
    block_reader: Arc<BlockReader>,
    storage_format: FuseStorageFormat,
    output: Arc<OutputPort>,
    stream_columns: Vec<StreamColumn>,
    stream_operators: Vec<BlockOperator>,
}

impl CompactSource {
    pub fn try_create(
        ctx: Arc<dyn TableContext>,
        storage_format: FuseStorageFormat,
        block_reader: Arc<BlockReader>,
        stream_columns: Vec<StreamColumn>,
        stream_operators: Vec<BlockOperator>,
        output: Arc<OutputPort>,
    ) -> Result<ProcessorPtr> {
        let func_ctx = ctx.get_function_context()?;
        Ok(ProcessorPtr::create(Box::new(CompactSource {
            state: State::ReadData(None),
            ctx,
            func_ctx,
            block_reader,
            storage_format,
            output,
            stream_columns,
            stream_operators,
        })))
    }
}

#[async_trait::async_trait]
impl Processor for CompactSource {
    fn name(&self) -> String {
        "CompactSource".to_string()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn event(&mut self) -> Result<Event> {
        if matches!(self.state, State::ReadData(None)) {
            self.state = self
                .ctx
                .get_partition()
                .map_or(State::Finish, |part| State::ReadData(Some(part)));
        }

        if self.output.is_finished() {
            return Ok(Event::Finished);
        }

        if !self.output.can_push() {
            return Ok(Event::NeedConsume);
        }

        match self.state {
            State::ReadData(_) => Ok(Event::Async),
            State::Concat { .. } => Ok(Event::Sync),
            State::Output(_, _) => {
                if let State::Output(part, data_block) =
                    std::mem::replace(&mut self.state, State::Finish)
                {
                    self.state = part.map_or(State::Finish, |part| State::ReadData(Some(part)));

                    self.output.push_data(Ok(data_block));
                    Ok(Event::NeedConsume)
                } else {
                    Err(ErrorCode::Internal("It's a bug."))
                }
            }
            State::Finish => {
                self.output.finish();
                Ok(Event::Finished)
            }
        }
    }

    fn process(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.state, State::Finish) {
            State::Concat {
                read_res,
                metas,
                index,
            } => {
                let blocks = read_res
                    .into_iter()
                    .zip(metas.into_iter())
                    .map(|(data, meta)| {
                        let mut block = self.block_reader.deserialize_chunks_with_meta(
                            &meta,
                            &self.storage_format,
                            data,
                        )?;

                        if self.block_reader.update_stream_columns() {
                            let num_rows = block.num_rows();
                            let stream_meta = gen_mutation_stream_meta(None, &meta.location.0)?;
                            for stream_column in self.stream_columns.iter() {
                                let entry =
                                    stream_column.generate_column_values(&stream_meta, num_rows);
                                block.add_column(entry);
                            }
                            block = self
                                .stream_operators
                                .iter()
                                .try_fold(block, |input, op| op.execute(&self.func_ctx, input))?;
                        }
                        Ok(block)
                    })
                    .collect::<Result<Vec<_>>>()?;

                // concat blocks.
                let block = if blocks.len() == 1 {
                    blocks[0].convert_to_full()
                } else {
                    DataBlock::concat(&blocks)?
                };

                let meta = Box::new(SerializeDataMeta::SerializeBlock(SerializeBlock::create(
                    index,
                    ClusterStatsGenType::Generally,
                )));
                let new_block = block.add_meta(Some(meta))?;

                let progress_values = ProgressValues {
                    rows: new_block.num_rows(),
                    bytes: new_block.memory_size(),
                };
                self.ctx.get_write_progress().incr(&progress_values);

                self.state = State::Output(self.ctx.get_partition(), new_block);
            }
            _ => return Err(ErrorCode::Internal("It's a bug.")),
        }
        Ok(())
    }

    #[async_backtrace::framed]
    async fn async_process(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.state, State::Finish) {
            State::ReadData(Some(part)) => {
                let block_reader = self.block_reader.as_ref();

                // block read tasks.
                let mut task_futures = Vec::new();
                let part = CompactPartInfo::from_part(&part)?;
                match part {
                    CompactPartInfo::CompactExtraInfo(extra) => {
                        let meta = Box::new(SerializeDataMeta::CompactExtras(extra.clone()));
                        let block = DataBlock::empty_with_meta(meta);
                        self.state = State::Output(self.ctx.get_partition(), block);
                    }
                    CompactPartInfo::CompactTaskInfo(task) => {
                        for block in &task.blocks {
                            let settings = ReadSettings::from_ctx(&self.ctx)?;
                            // read block in parallel.
                            task_futures.push(async move {
                                // Perf
                                {
                                    metrics_inc_compact_block_read_nums(1);
                                    metrics_inc_compact_block_read_bytes(block.block_size);
                                }

                                block_reader
                                    .read_columns_data_by_merge_io(
                                        &settings,
                                        &block.location.0,
                                        &block.col_metas,
                                        &None,
                                    )
                                    .await
                            });
                        }

                        let start = Instant::now();

                        let read_res = futures::future::try_join_all(task_futures).await?;
                        // Perf.
                        {
                            metrics_inc_compact_block_read_milliseconds(
                                start.elapsed().as_millis() as u64,
                            );
                        }
                        self.state = State::Concat {
                            read_res,
                            metas: task.blocks.clone(),
                            index: task.index.clone(),
                        };
                    }
                }
                Ok(())
            }
            _ => Err(ErrorCode::Internal("It's a bug.")),
        }
    }
}
