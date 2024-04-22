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

use databend_common_ast::ast::TableReference;
use databend_common_ast::ast::UpdateStmt;
use databend_common_exception::ErrorCode;
use databend_common_exception::Result;
use databend_common_expression::types::NumberScalar;
use databend_common_expression::Scalar;
use databend_common_expression::ROW_VERSION_COL_NAME;

use crate::binder::Binder;
use crate::binder::ScalarBinder;
use crate::normalize_identifier;
use crate::plans::BoundColumnRef;
use crate::plans::ConstantExpr;
use crate::plans::FunctionCall;
use crate::plans::Plan;
use crate::plans::UpdatePlan;
use crate::BindContext;
use crate::ScalarExpr;

impl Binder {
    #[async_backtrace::framed]
    pub(in crate::planner::binder) async fn bind_update(
        &mut self,
        bind_context: &mut BindContext,
        stmt: &UpdateStmt,
    ) -> Result<Plan> {
        let UpdateStmt {
            table,
            update_list,
            selection,
            ..
        } = stmt;

        let (catalog_name, database_name, table_name) = if let TableReference::Table {
            catalog,
            database,
            table,
            ..
        } = table
        {
            (
                catalog
                    .as_ref()
                    .map_or_else(|| self.ctx.get_current_catalog(), |i| i.name.clone()),
                database
                    .as_ref()
                    .map_or_else(|| self.ctx.get_current_database(), |i| i.name.clone()),
                table.name.clone(),
            )
        } else {
            // we do not support USING clause yet
            return Err(ErrorCode::Internal(
                "should not happen, parser should have report error already",
            ));
        };

        let (table_expr, mut context) = self.bind_single_table(bind_context, table).await?;

        let table = self
            .ctx
            .get_table(&catalog_name, &database_name, &table_name)
            .await?;

        context.allow_internal_columns(false);
        let mut scalar_binder = ScalarBinder::new(
            &mut context,
            self.ctx.clone(),
            &self.name_resolution_ctx,
            self.metadata.clone(),
            &[],
            self.m_cte_bound_ctx.clone(),
            self.ctes_map.clone(),
        );
        let schema = table.schema();
        let mut update_columns = HashMap::with_capacity(update_list.len());
        for update_expr in update_list {
            let col_name = normalize_identifier(&update_expr.name, &self.name_resolution_ctx).name;
            let index = schema.index_of(&col_name)?;
            if update_columns.contains_key(&index) {
                return Err(ErrorCode::BadArguments(format!(
                    "Multiple assignments in the single statement to column `{}`",
                    col_name
                )));
            }
            let field = schema.field(index);
            if field.computed_expr().is_some() {
                return Err(ErrorCode::BadArguments(format!(
                    "The value specified for computed column '{}' is not allowed",
                    field.name()
                )));
            }

            // TODO(zhyass): update_list support subquery.
            let (scalar, _) = scalar_binder.bind(&update_expr.expr).await?;
            if !self.check_allowed_scalar_expr(&scalar)? {
                return Err(ErrorCode::SemanticError(
                    "update_list in update statement can't contain subquery|window|aggregate|udf functions".to_string(),
                )
                .set_span(scalar.span()));
            }

            update_columns.insert(index, scalar);
        }

        let (selection, subquery_desc) = self
            .process_selection(selection, table_expr, &mut scalar_binder)
            .await?;

        if let Some(selection) = &selection {
            if !self.check_allowed_scalar_expr_with_subquery(selection)? {
                return Err(ErrorCode::SemanticError(
                    "selection in update statement can't contain window|aggregate|udf functions"
                        .to_string(),
                )
                .set_span(selection.span()));
            }
        }

        let bind_context = Box::new(context.clone());
        if table.change_tracking_enabled() {
            let schema = table.schema_with_stream();
            let col_name = ROW_VERSION_COL_NAME;
            let index = schema.index_of(col_name)?;
            let mut row_version = None;
            for column_binding in bind_context.columns.iter() {
                if BindContext::match_column_binding(
                    Some(&database_name),
                    Some(&table_name),
                    col_name,
                    column_binding,
                ) {
                    row_version = Some(ScalarExpr::BoundColumnRef(BoundColumnRef {
                        span: None,
                        column: column_binding.clone(),
                    }));
                    break;
                }
            }
            let col = row_version.ok_or_else(|| ErrorCode::Internal("It's a bug"))?;
            let scalar = ScalarExpr::FunctionCall(FunctionCall {
                span: None,
                func_name: "plus".to_string(),
                params: vec![],
                arguments: vec![
                    col,
                    ConstantExpr {
                        span: None,
                        value: Scalar::Number(NumberScalar::UInt64(1)),
                    }
                    .into(),
                ],
            });
            update_columns.insert(index, scalar);
        }

        let plan = UpdatePlan {
            catalog: catalog_name,
            database: database_name,
            table: table_name,
            update_list: update_columns,
            selection,
            bind_context,
            metadata: self.metadata.clone(),
            subquery_desc,
        };
        Ok(Plan::Update(Box::new(plan)))
    }
}
