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

use databend_common_catalog::catalog_kind::CATALOG_DEFAULT;
use databend_common_catalog::plan::DataSourcePlan;
use databend_common_catalog::plan::Filters;
use databend_common_catalog::plan::PartStatistics;
use databend_common_catalog::plan::Partitions;
use databend_common_catalog::plan::PushDownInfo;
use databend_common_exception::Result;
use databend_common_expression::DataBlock;
use databend_common_meta_app::schema::TableIdent;
use databend_common_meta_app::schema::TableInfo;
use databend_common_meta_app::schema::TableMeta;
use databend_common_pipeline_core::processors::OutputPort;
use databend_common_pipeline_core::processors::ProcessorPtr;
use databend_common_pipeline_core::Pipeline;
use databend_common_pipeline_sources::AsyncSource;
use databend_common_pipeline_sources::AsyncSourcer;

use super::FuseEncoding;
use crate::sessions::TableContext;
use crate::table_functions::parse_db_tb_col_args;
use crate::table_functions::string_literal;
use crate::table_functions::TableArgs;
use crate::table_functions::TableFunction;
use crate::FuseTable;
use crate::Table;

const FUSE_FUNC_ENCODING: &str = "fuse_encoding";

pub struct FuseEncodingTable {
    table_info: TableInfo,
    arg_database_name: String,
}

impl FuseEncodingTable {
    pub fn create(
        database_name: &str,
        table_func_name: &str,
        table_id: u64,
        table_args: TableArgs,
    ) -> Result<Arc<dyn TableFunction>> {
        let arg_database_name = parse_db_tb_col_args(&table_args, FUSE_FUNC_ENCODING)?;

        let engine = FUSE_FUNC_ENCODING.to_owned();

        let table_info = TableInfo {
            ident: TableIdent::new(table_id, 0),
            desc: format!("'{}'.'{}'", database_name, table_func_name),
            name: table_func_name.to_string(),
            meta: TableMeta {
                schema: FuseEncoding::schema(),
                engine,
                ..Default::default()
            },
            ..Default::default()
        };

        Ok(Arc::new(FuseEncodingTable {
            table_info,
            arg_database_name,
        }))
    }
}

#[async_trait::async_trait]
impl Table for FuseEncodingTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_table_info(&self) -> &TableInfo {
        &self.table_info
    }

    #[async_backtrace::framed]
    async fn read_partitions(
        &self,
        _ctx: Arc<dyn TableContext>,
        _push_downs: Option<PushDownInfo>,
        _dry_run: bool,
    ) -> Result<(PartStatistics, Partitions)> {
        Ok((PartStatistics::default(), Partitions::default()))
    }

    fn table_args(&self) -> Option<TableArgs> {
        let args = vec![string_literal(self.arg_database_name.as_str())];
        Some(TableArgs::new_positioned(args))
    }

    fn read_data(
        &self,
        ctx: Arc<dyn TableContext>,
        plan: &DataSourcePlan,
        pipeline: &mut Pipeline,
        _put_cache: bool,
    ) -> Result<()> {
        pipeline.add_source(
            |output| {
                FuseEncodingSource::create(
                    ctx.clone(),
                    output,
                    self.arg_database_name.to_owned(),
                    plan.push_downs.as_ref().and_then(|x| x.limit),
                    plan.push_downs.as_ref().and_then(|x| x.filters.clone()),
                )
            },
            1,
        )?;

        Ok(())
    }
}

struct FuseEncodingSource {
    finish: bool,
    ctx: Arc<dyn TableContext>,
    arg_database_name: String,
    limit: Option<usize>,
    filters: Option<Filters>,
}

impl FuseEncodingSource {
    pub fn create(
        ctx: Arc<dyn TableContext>,
        output: Arc<OutputPort>,
        arg_database_name: String,
        limit: Option<usize>,
        filters: Option<Filters>,
    ) -> Result<ProcessorPtr> {
        AsyncSourcer::create(ctx.clone(), output, Self {
            ctx,
            finish: false,
            arg_database_name,
            limit,
            filters,
        })
    }
}

#[async_trait::async_trait]
impl AsyncSource for FuseEncodingSource {
    const NAME: &'static str = "fuse_encoding";

    #[async_trait::unboxed_simple]
    #[async_backtrace::framed]
    async fn generate(&mut self) -> Result<Option<DataBlock>> {
        if self.finish {
            return Ok(None);
        }

        self.finish = true;
        let tenant_id = self.ctx.get_tenant();
        let tbls = self
            .ctx
            .get_catalog(CATALOG_DEFAULT)
            .await?
            .get_database(tenant_id.as_str(), self.arg_database_name.as_str())
            .await?
            .list_tables()
            .await?;

        let fuse_tables = tbls
            .iter()
            .map(|tbl| {
                let tbl = FuseTable::try_from_table(tbl.as_ref()).unwrap();
                tbl
            })
            .collect::<Vec<_>>();
        Ok(Some(
            FuseEncoding::new(
                self.ctx.clone(),
                fuse_tables,
                self.limit,
                self.filters.clone(),
            )
            .get_blocks()
            .await?,
        ))
    }
}

impl TableFunction for FuseEncodingTable {
    fn function_name(&self) -> &str {
        self.name()
    }

    fn as_table<'a>(self: Arc<Self>) -> Arc<dyn Table + 'a>
    where Self: 'a {
        self
    }
}
