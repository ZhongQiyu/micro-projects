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

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use databend_common_catalog::table_context::TableContext;
use databend_common_meta_app::principal::FileFormatParams;
use databend_common_pipeline_core::processors::ProcessorPtr;
use databend_common_pipeline_core::Pipeline;
use databend_common_pipeline_transforms::processors::AccumulatingTransformer;

use super::parquet_file::append_data_to_parquet_files;
use super::row_based_file::append_data_to_row_based_files;
use crate::append::output::SumSummaryTransform;
use crate::StageTable;

impl StageTable {
    pub(crate) fn do_append_data(
        &self,
        ctx: Arc<dyn TableContext>,
        pipeline: &mut Pipeline,
    ) -> databend_common_exception::Result<()> {
        let settings = ctx.get_settings();

        let single = self.table_info.stage_info.copy_options.single;
        let max_file_size = if single {
            usize::MAX
        } else {
            let max_file_size = self.table_info.stage_info.copy_options.max_file_size;
            if max_file_size == 0 {
                // 256M per file by default.
                256 * 1024 * 1024
            } else {
                let mem_limit = (settings.get_max_memory_usage()? / 2) as usize;
                max_file_size.min(mem_limit)
            }
        };
        let max_threads = settings.get_max_threads()? as usize;

        let op = StageTable::get_op(&self.table_info.stage_info)?;
        let fmt = self.table_info.stage_info.file_format_params.clone();
        let uuid = uuid::Uuid::new_v4().to_string();
        let group_id = AtomicUsize::new(0);
        match fmt {
            FileFormatParams::Parquet(_) => append_data_to_parquet_files(
                pipeline,
                ctx.clone(),
                self.table_info.clone(),
                op,
                max_file_size,
                max_threads,
                uuid,
                &group_id,
            )?,
            _ => append_data_to_row_based_files(
                pipeline,
                ctx.clone(),
                self.table_info.clone(),
                op,
                max_file_size,
                max_threads,
                uuid,
                &group_id,
            )?,
        };
        if !self.table_info.stage_info.copy_options.detailed_output {
            pipeline.try_resize(1)?;
            pipeline.add_transform(move |input, output| {
                Ok(ProcessorPtr::create(AccumulatingTransformer::create(
                    input,
                    output,
                    SumSummaryTransform::default(),
                )))
            })?;
        }
        Ok(())
    }
}
