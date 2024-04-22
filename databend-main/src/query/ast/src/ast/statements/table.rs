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

use std::collections::BTreeMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::format;
use std::time::Duration;

use databend_common_meta_app::schema::CreateOption;
use derive_visitor::Drive;
use derive_visitor::DriveMut;

use crate::ast::statements::show::ShowLimit;
use crate::ast::write_comma_separated_list;
use crate::ast::write_comma_separated_map;
use crate::ast::write_dot_separated_list;
use crate::ast::Expr;
use crate::ast::Identifier;
use crate::ast::Query;
use crate::ast::TableReference;
use crate::ast::TimeTravelPoint;
use crate::ast::TypeName;
use crate::ast::UriLocation;

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct ShowTablesStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    #[drive(skip)]
    pub full: bool,
    pub limit: Option<ShowLimit>,
    #[drive(skip)]
    pub with_history: bool,
}

impl Display for ShowTablesStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "SHOW")?;
        if self.full {
            write!(f, " FULL")?;
        }
        write!(f, " TABLES")?;
        if self.with_history {
            write!(f, " HISTORY")?;
        }
        if let Some(database) = &self.database {
            write!(f, " FROM ")?;
            if let Some(catalog) = &self.catalog {
                write!(f, "{catalog}.",)?;
            }
            write!(f, "{database}")?;
        }
        if let Some(limit) = &self.limit {
            write!(f, " {limit}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct ShowCreateTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
}

impl Display for ShowCreateTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "SHOW CREATE TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct ShowTablesStatusStmt {
    pub database: Option<Identifier>,
    pub limit: Option<ShowLimit>,
}

impl Display for ShowTablesStatusStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "SHOW TABLE STATUS")?;
        if let Some(database) = &self.database {
            write!(f, " FROM {database}")?;
        }
        if let Some(limit) = &self.limit {
            write!(f, " {limit}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct ShowDropTablesStmt {
    pub database: Option<Identifier>,
    pub limit: Option<ShowLimit>,
}

impl Display for ShowDropTablesStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "SHOW DROP TABLE")?;
        if let Some(database) = &self.database {
            write!(f, " FROM {database}")?;
        }
        if let Some(limit) = &self.limit {
            write!(f, " {limit}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct CreateTableStmt {
    #[drive(skip)]
    pub create_option: CreateOption,
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
    pub source: Option<CreateTableSource>,
    pub engine: Option<Engine>,
    pub uri_location: Option<UriLocation>,
    pub cluster_by: Vec<Expr>,
    #[drive(skip)]
    pub table_options: BTreeMap<String, String>,
    pub as_query: Option<Box<Query>>,
    #[drive(skip)]
    pub transient: bool,
}

impl Display for CreateTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "CREATE ")?;
        if let CreateOption::CreateOrReplace = self.create_option {
            write!(f, "OR REPLACE ")?;
        }
        if self.transient {
            write!(f, "TRANSIENT ")?;
        }
        write!(f, "TABLE ")?;
        if let CreateOption::CreateIfNotExists = self.create_option {
            write!(f, "IF NOT EXISTS ")?;
        }
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;

        if let Some(source) = &self.source {
            write!(f, " {source}")?;
        }

        if let Some(uri_location) = &self.uri_location {
            write!(f, " {uri_location}")?;
        }

        if let Some(engine) = &self.engine {
            write!(f, " ENGINE = {engine}")?;
        }

        if !self.cluster_by.is_empty() {
            write!(f, " CLUSTER BY (")?;
            write_comma_separated_list(f, &self.cluster_by)?;
            write!(f, ")")?
        }

        // Format table options
        write_comma_separated_map(f, &self.table_options)?;
        if let Some(as_query) = &self.as_query {
            write!(f, " AS {as_query}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct AttachTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
    pub uri_location: UriLocation,
    #[drive(skip)]
    pub read_only: bool,
}

impl Display for AttachTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "ATTACH TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;

        write!(f, " FROM {}", self.uri_location)?;

        if self.read_only {
            write!(f, " READ_ONLY")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub enum CreateTableSource {
    Columns(Vec<ColumnDefinition>),
    Like {
        catalog: Option<Identifier>,
        database: Option<Identifier>,
        table: Identifier,
    },
}

impl Display for CreateTableSource {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            CreateTableSource::Columns(columns) => {
                write!(f, "(")?;
                write_comma_separated_list(f, columns)?;
                write!(f, ")")
            }
            CreateTableSource::Like {
                catalog,
                database,
                table,
            } => {
                write!(f, "LIKE ")?;
                write_dot_separated_list(f, catalog.iter().chain(database).chain(Some(table)))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct DescribeTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
}

impl Display for DescribeTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "DESCRIBE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(self.database.iter().chain(Some(&self.table))),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct DropTableStmt {
    #[drive(skip)]
    pub if_exists: bool,
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
    #[drive(skip)]
    pub all: bool,
}

impl Display for DropTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "DROP TABLE ")?;
        if self.if_exists {
            write!(f, "IF EXISTS ")?;
        }
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;
        if self.all {
            write!(f, " ALL")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct UndropTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
}

impl Display for UndropTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "UNDROP TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct AlterTableStmt {
    #[drive(skip)]
    pub if_exists: bool,
    pub table_reference: TableReference,
    pub action: AlterTableAction,
}

impl Display for AlterTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "ALTER TABLE ")?;
        if self.if_exists {
            write!(f, "IF EXISTS ")?;
        }
        write!(f, "{}", self.table_reference)?;
        write!(f, " {}", self.action)
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub enum AlterTableAction {
    RenameTable {
        new_table: Identifier,
    },
    AddColumn {
        column: ColumnDefinition,
        option: AddColumnOption,
    },
    RenameColumn {
        old_column: Identifier,
        new_column: Identifier,
    },
    ModifyColumn {
        action: ModifyColumnAction,
    },
    DropColumn {
        column: Identifier,
    },
    AlterTableClusterKey {
        cluster_by: Vec<Expr>,
    },
    DropTableClusterKey,
    ReclusterTable {
        #[drive(skip)]
        is_final: bool,
        selection: Option<Expr>,
        #[drive(skip)]
        limit: Option<u64>,
    },
    RevertTo {
        point: TimeTravelPoint,
    },
    SetOptions {
        #[drive(skip)]
        set_options: BTreeMap<String, String>,
    },
}

impl Display for AlterTableAction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AlterTableAction::SetOptions { set_options } => {
                write!(f, "SET OPTIONS (")?;
                write_comma_separated_map(f, set_options)?;
                write!(f, ")")?;
            }
            AlterTableAction::RenameTable { new_table } => {
                write!(f, "RENAME TO {new_table}")?;
            }
            AlterTableAction::RenameColumn {
                old_column,
                new_column,
            } => {
                write!(f, "RENAME COLUMN {old_column} TO {new_column}")?;
            }
            AlterTableAction::AddColumn { column, option } => {
                write!(f, "ADD COLUMN {column}{option}")?;
            }
            AlterTableAction::ModifyColumn { action } => {
                write!(f, "MODIFY COLUMN {action}")?;
            }
            AlterTableAction::DropColumn { column } => {
                write!(f, "DROP COLUMN {column}")?;
            }
            AlterTableAction::AlterTableClusterKey { cluster_by } => {
                write!(f, "CLUSTER BY ")?;
                write_comma_separated_list(f, cluster_by)?;
            }
            AlterTableAction::DropTableClusterKey => {
                write!(f, "DROP CLUSTER KEY")?;
            }
            AlterTableAction::ReclusterTable {
                is_final,
                selection,
                limit,
            } => {
                write!(f, "RECLUSTER")?;
                if *is_final {
                    write!(f, " FINAL")?;
                }
                if let Some(conditions) = selection {
                    write!(f, " WHERE {conditions}")?;
                }
                if let Some(limit) = limit {
                    write!(f, " LIMIT {limit}")?;
                }
            }
            AlterTableAction::RevertTo { point } => {
                write!(f, "REVERT TO {}", point)?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub enum AddColumnOption {
    End,
    First,
    After(Identifier),
}

impl Display for AddColumnOption {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AddColumnOption::First => write!(f, " FIRST"),
            AddColumnOption::After(ident) => write!(f, " AFTER {ident}"),
            AddColumnOption::End => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct RenameTableStmt {
    #[drive(skip)]
    pub if_exists: bool,
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
    pub new_catalog: Option<Identifier>,
    pub new_database: Option<Identifier>,
    pub new_table: Identifier,
}

impl Display for RenameTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "RENAME TABLE ")?;
        if self.if_exists {
            write!(f, "IF EXISTS ")?;
        }
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;
        write!(f, " TO ")?;
        write_dot_separated_list(
            f,
            self.new_catalog
                .iter()
                .chain(&self.new_database)
                .chain(Some(&self.new_table)),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct TruncateTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
}

impl Display for TruncateTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "TRUNCATE TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct VacuumTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
    pub option: VacuumTableOption,
}

impl Display for VacuumTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "VACUUM TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;
        write!(f, " {}", &self.option)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct VacuumDropTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub option: VacuumDropTableOption,
}

impl Display for VacuumDropTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "VACUUM DROP TABLE ")?;
        if self.catalog.is_some() || self.database.is_some() {
            write!(f, "FROM ")?;
            write_dot_separated_list(f, self.catalog.iter().chain(&self.database))?;
            write!(f, " ")?;
        }
        write!(f, "{}", &self.option)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct VacuumTemporaryFiles {
    #[drive(skip)]
    pub limit: Option<u64>,
    #[drive(skip)]
    pub retain: Option<Duration>,
}

impl Display for crate::ast::VacuumTemporaryFiles {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "VACUUM TEMPORARY FILES ")?;
        if let Some(retain) = &self.retain {
            let days = Duration::from_secs(60 * 60 * 24);
            if retain >= &days {
                let days = retain.as_secs() / (60 * 60 * 24);
                write!(f, "RETAIN {days} DAYS ")?;
            } else {
                let seconds = retain.as_secs();
                write!(f, "RETAIN {seconds} SECONDS ")?;
            }
        }

        if let Some(limit) = &self.limit {
            write!(f, " LIMIT {limit}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct OptimizeTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
    pub action: OptimizeTableAction,
    #[drive(skip)]
    pub limit: Option<u64>,
}

impl Display for OptimizeTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "OPTIMIZE TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;
        write!(f, " {}", &self.action)?;
        if let Some(limit) = self.limit {
            write!(f, " LIMIT {limit}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct AnalyzeTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
}

impl Display for AnalyzeTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "ANALYZE TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct ExistsTableStmt {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
}

impl Display for ExistsTableStmt {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "EXISTS TABLE ")?;
        write_dot_separated_list(
            f,
            self.catalog
                .iter()
                .chain(&self.database)
                .chain(Some(&self.table)),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Drive, DriveMut)]
pub enum Engine {
    Null,
    Memory,
    Fuse,
    View,
    Random,
    Iceberg,
    Delta,
}

impl Display for Engine {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Engine::Null => write!(f, "NULL"),
            Engine::Memory => write!(f, "MEMORY"),
            Engine::Fuse => write!(f, "FUSE"),
            Engine::View => write!(f, "VIEW"),
            Engine::Random => write!(f, "RANDOM"),
            Engine::Iceberg => write!(f, "ICEBERG"),
            Engine::Delta => write!(f, "DELTA"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub enum CompactTarget {
    Block,
    Segment,
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct VacuumTableOption {
    #[drive(skip)]
    pub dry_run: bool,
}

impl Display for VacuumTableOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.dry_run {
            write!(f, "DRY RUN")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct VacuumDropTableOption {
    #[drive(skip)]
    pub dry_run: bool,
    #[drive(skip)]
    pub limit: Option<usize>,
}

impl Display for VacuumDropTableOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.dry_run {
            write!(f, "DRY RUN")?;
        }
        if let Some(limit) = self.limit {
            write!(f, " LIMIT {}", limit)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub enum OptimizeTableAction {
    All,
    Purge { before: Option<TimeTravelPoint> },
    Compact { target: CompactTarget },
}

impl Display for OptimizeTableAction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OptimizeTableAction::All => write!(f, "ALL"),
            OptimizeTableAction::Purge { before } => {
                write!(f, "PURGE")?;
                if let Some(point) = before {
                    write!(f, " BEFORE {}", point)?;
                }
                Ok(())
            }
            OptimizeTableAction::Compact { target } => {
                match target {
                    CompactTarget::Block => {
                        write!(f, "COMPACT BLOCK")?;
                    }
                    CompactTarget::Segment => {
                        write!(f, "COMPACT SEGMENT")?;
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub enum ColumnExpr {
    Default(Box<Expr>),
    Virtual(Box<Expr>),
    Stored(Box<Expr>),
}

impl Display for ColumnExpr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ColumnExpr::Default(expr) => {
                write!(f, " DEFAULT {expr}")?;
            }
            ColumnExpr::Virtual(expr) => {
                write!(f, " AS ({expr}) VIRTUAL")?;
            }
            ColumnExpr::Stored(expr) => {
                write!(f, " AS ({expr}) STORED")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub enum NullableConstraint {
    Null,
    NotNull,
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct ColumnDefinition {
    pub name: Identifier,
    pub data_type: TypeName,
    pub expr: Option<ColumnExpr>,
    #[drive(skip)]
    pub comment: Option<String>,
}

impl Display for ColumnDefinition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.data_type)?;
        if let Some(expr) = &self.expr {
            write!(f, "{expr}")?;
        }
        if let Some(comment) = &self.comment {
            write!(f, " COMMENT '{comment}'")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub enum ModifyColumnAction {
    // (column name id, masking policy name)
    SetMaskingPolicy(Identifier, #[drive(skip)] String),
    // column name id
    UnsetMaskingPolicy(Identifier),
    // vec<ColumnDefinition>
    SetDataType(Vec<ColumnDefinition>),
    // column name id
    ConvertStoredComputedColumn(Identifier),
}

impl Display for ModifyColumnAction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self {
            ModifyColumnAction::SetMaskingPolicy(column, name) => {
                write!(f, "{} SET MASKING POLICY {}", column, name)?
            }
            ModifyColumnAction::UnsetMaskingPolicy(column) => {
                write!(f, "{} UNSET MASKING POLICY", column)?
            }
            ModifyColumnAction::SetDataType(column_def_vec) => {
                let ret = column_def_vec
                    .iter()
                    .enumerate()
                    .map(|(i, column_def)| {
                        let default_expr_str = match &column_def.expr {
                            Some(default_expr) => default_expr.to_string(),
                            None => "".to_string(),
                        };
                        let comment = match &column_def.comment {
                            Some(comment) => format!(" COMMENT {}", comment),
                            None => "".to_string(),
                        };
                        if i > 0 {
                            format!(
                                " COLUMN {} {}{}{}",
                                column_def.name, column_def.data_type, default_expr_str, comment
                            )
                        } else {
                            format!(
                                "{} {}{}{}",
                                column_def.name, column_def.data_type, default_expr_str, comment
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",");

                write!(f, "{}", ret)?
            }
            ModifyColumnAction::ConvertStoredComputedColumn(column) => {
                write!(f, "{} DROP STORED", column)?
            }
        }

        Ok(())
    }
}