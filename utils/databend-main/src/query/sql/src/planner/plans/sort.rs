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

use databend_common_catalog::table_context::TableContext;
use databend_common_exception::Result;

use crate::optimizer::Distribution;
use crate::optimizer::PhysicalProperty;
use crate::optimizer::RelExpr;
use crate::optimizer::RelationalProperty;
use crate::optimizer::RequiredProperty;
use crate::optimizer::StatInfo;
use crate::plans::Operator;
use crate::plans::RelOp;
use crate::IndexType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sort {
    pub items: Vec<SortItem>,
    pub limit: Option<usize>,

    /// If the sort plan is after the exchange plan.
    /// It's [None] if the sorting plan is in single node mode.
    pub after_exchange: Option<bool>,

    /// The columns needed by the plan after the sort plan.
    /// It's used to build a projection operation before building the sort operator.
    pub pre_projection: Option<Vec<IndexType>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SortItem {
    pub index: IndexType,
    pub asc: bool,
    pub nulls_first: bool,
}

impl Operator for Sort {
    fn rel_op(&self) -> RelOp {
        RelOp::Sort
    }

    fn arity(&self) -> usize {
        1
    }

    fn derive_physical_prop(&self, rel_expr: &RelExpr) -> Result<PhysicalProperty> {
        rel_expr.derive_physical_prop_child(0)
    }

    fn compute_required_prop_child(
        &self,
        _ctx: Arc<dyn TableContext>,
        _rel_expr: &RelExpr,
        _child_index: usize,
        required: &RequiredProperty,
    ) -> Result<RequiredProperty> {
        let mut required = required.clone();
        required.distribution = Distribution::Serial;
        Ok(required)
    }

    fn compute_required_prop_children(
        &self,
        _ctx: Arc<dyn TableContext>,
        _rel_expr: &RelExpr,
        _required: &RequiredProperty,
    ) -> Result<Vec<Vec<RequiredProperty>>> {
        Ok(vec![vec![RequiredProperty {
            distribution: Distribution::Serial,
        }]])
    }

    fn derive_relational_prop(&self, rel_expr: &RelExpr) -> Result<Arc<RelationalProperty>> {
        let input_prop = rel_expr.derive_relational_prop_child(0)?;

        let output_columns = input_prop.output_columns.clone();
        let outer_columns = input_prop.outer_columns.clone();
        let used_columns = input_prop.used_columns.clone();

        // Derive orderings
        let orderings = self.items.clone();

        Ok(Arc::new(RelationalProperty {
            output_columns,
            outer_columns,
            used_columns,
            orderings,
        }))
    }

    fn derive_stats(&self, rel_expr: &RelExpr) -> Result<Arc<StatInfo>> {
        rel_expr.derive_cardinality_child(0)
    }
}
