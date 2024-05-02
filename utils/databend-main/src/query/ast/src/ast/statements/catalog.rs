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

use databend_common_meta_app::schema::CatalogType;
use derive_visitor::Drive;
use derive_visitor::DriveMut;

use crate::ast::write_comma_separated_map;
use crate::ast::Identifier;
use crate::ast::ShowLimit;

#[derive(Debug, Clone, PartialEq, Drive, DriveMut)]
pub struct ShowCatalogsStmt {
    pub limit: Option<ShowLimit>,
}

impl Display for ShowCatalogsStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SHOW CATALOGS")?;
        if let Some(limit) = &self.limit {
            write!(f, " {}", limit)?
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct ShowCreateCatalogStmt {
    pub catalog: Identifier,
}

impl Display for ShowCreateCatalogStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SHOW CREATE CATALOG {}", &self.catalog)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct CreateCatalogStmt {
    #[drive(skip)]
    pub if_not_exists: bool,
    #[drive(skip)]
    pub catalog_name: String,
    #[drive(skip)]
    pub catalog_type: CatalogType,
    #[drive(skip)]
    pub catalog_options: BTreeMap<String, String>,
}

impl Display for CreateCatalogStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CREATE CATALOG")?;
        if self.if_not_exists {
            write!(f, " IF NOT EXISTS")?;
        }
        write!(f, " {}", self.catalog_name)?;
        write!(f, " TYPE='{}'", self.catalog_type)?;
        write!(f, " CONNECTION = ( ")?;
        write_comma_separated_map(f, &self.catalog_options)?;
        write!(f, " )")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct DropCatalogStmt {
    #[drive(skip)]
    pub if_exists: bool,
    pub catalog: Identifier,
}

impl Display for DropCatalogStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DROP CATALOG ")?;
        if self.if_exists {
            write!(f, "IF EXISTS ")?;
        }
        write!(f, "{}", self.catalog)
    }
}
