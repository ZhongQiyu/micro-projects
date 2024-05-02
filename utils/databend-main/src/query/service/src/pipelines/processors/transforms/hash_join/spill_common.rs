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

// Define some methods that are used by both the build and probe spilling of the hash join.

use databend_common_arrow::arrow::bitmap::Bitmap;
use databend_common_exception::Result;
use databend_common_expression::types::DataType;
use databend_common_expression::Column;
use databend_common_expression::DataBlock;
use databend_common_expression::Evaluator;
use databend_common_expression::Expr;
use databend_common_expression::FunctionContext;
use databend_common_expression::HashMethodKind;
use databend_common_functions::BUILTIN_FUNCTIONS;
use databend_common_sql::plans::JoinType;

use crate::pipelines::processors::transforms::hash_join::common::wrap_true_validity;
use crate::pipelines::processors::transforms::hash_join::util::hash_by_method;

pub fn get_hashes(
    func_ctx: &FunctionContext,
    block: &DataBlock,
    keys: &[Expr],
    method: &HashMethodKind,
    join_type: Option<&JoinType>,
    hashes: &mut Vec<u64>,
) -> Result<()> {
    let mut block = block.clone();
    let mut evaluator = Evaluator::new(&block, func_ctx, &BUILTIN_FUNCTIONS);
    if let Some(join_type) = join_type {
        if matches!(
            join_type,
            JoinType::Left | JoinType::LeftSingle | JoinType::Full
        ) {
            let validity = Bitmap::new_constant(true, block.num_rows());
            block = DataBlock::new(
                block
                    .columns()
                    .iter()
                    .map(|c| wrap_true_validity(c, block.num_rows(), &validity))
                    .collect::<Vec<_>>(),
                block.num_rows(),
            );
            evaluator = Evaluator::new(&block, func_ctx, &BUILTIN_FUNCTIONS);
        }
    }
    let columns: Vec<(Column, DataType)> = keys
        .iter()
        .map(|expr| {
            let return_type = expr.data_type();
            Ok((
                evaluator
                    .run(expr)?
                    .convert_to_full_column(return_type, block.num_rows()),
                return_type.clone(),
            ))
        })
        .collect::<Result<_>>()?;
    hash_by_method(method, &columns, block.num_rows(), hashes)?;
    Ok(())
}

pub fn spilling_supported_join_type(join_type: &JoinType) -> bool {
    matches!(
        *join_type,
        JoinType::Inner
            | JoinType::Left
            | JoinType::LeftSemi
            | JoinType::LeftAnti
            | JoinType::LeftSingle
    )
}
