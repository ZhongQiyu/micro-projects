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

use std::fmt::Display;
use std::fmt::Formatter;

use databend_common_exception::Span;
use derive_visitor::Drive;
use derive_visitor::DriveMut;

use crate::parser::quote::quote_ident;

// Identifier of table name or column name.
#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct Identifier {
    #[drive(skip)]
    pub span: Span,
    #[drive(skip)]
    pub name: String,
    #[drive(skip)]
    pub quote: Option<char>,
}

impl Identifier {
    pub fn is_quoted(&self) -> bool {
        self.quote.is_some()
    }

    pub fn from_name(name: impl Into<String>) -> Self {
        Self {
            span: Span::default(),
            name: name.into(),
            quote: None,
        }
    }

    pub fn from_name_with_quoted(name: impl Into<String>, quote: Option<char>) -> Self {
        Self {
            span: Span::default(),
            name: name.into(),
            quote,
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = self.quote {
            let quoted = quote_ident(&self.name, c, true);
            write!(f, "{}", quoted)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct ColumnPosition {
    #[drive(skip)]
    pub span: Span,
    #[drive(skip)]
    pub pos: usize,
    #[drive(skip)]
    pub name: String,
}

impl ColumnPosition {
    pub fn create(span: Span, pos: usize) -> ColumnPosition {
        ColumnPosition {
            pos,
            name: format!("${}", pos),
            span,
        }
    }
    pub fn name(&self) -> String {
        format!("${}", self.pos)
    }
}

impl Display for ColumnPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.pos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub enum ColumnID {
    Name(Identifier),
    Position(ColumnPosition),
}

impl ColumnID {
    pub fn name(&self) -> &str {
        match self {
            ColumnID::Name(id) => &id.name,
            ColumnID::Position(id) => &id.name,
        }
    }
}

impl Display for ColumnID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnID::Name(id) => write!(f, "{}", id),
            ColumnID::Position(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct DatabaseRef {
    pub catalog: Option<Identifier>,
    pub database: Identifier,
}

impl Display for DatabaseRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(catalog) = &self.catalog {
            write!(f, "{}.", catalog)?;
        }
        write!(f, "{}", self.database)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct TableRef {
    pub catalog: Option<Identifier>,
    pub database: Option<Identifier>,
    pub table: Identifier,
}

impl Display for TableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        assert!(self.catalog.is_none() || (self.catalog.is_some() && self.database.is_some()));
        if let Some(catalog) = &self.catalog {
            write!(f, "{}.", catalog)?;
        }
        if let Some(database) = &self.database {
            write!(f, "{}.", database)?;
        }
        write!(f, "{}", self.table)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct ColumnRef {
    pub database: Option<Identifier>,
    pub table: Option<Identifier>,
    pub column: ColumnID,
}

impl Display for ColumnRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        assert!(self.database.is_none() || (self.database.is_some() && self.table.is_some()));

        if f.alternate() {
            write!(f, "{}", self.column)?;
            return Ok(());
        }

        if let Some(database) = &self.database {
            write!(f, "{}.", database)?;
        }
        if let Some(table) = &self.table {
            write!(f, "{}.", table)?;
        }
        write!(f, "{}", self.column)?;
        Ok(())
    }
}

pub(crate) fn write_dot_separated_list(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = impl Display>,
) -> std::fmt::Result {
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            write!(f, ".")?;
        }
        write!(f, "{}", item)?;
    }
    Ok(())
}

/// Write input items into `a, b, c`
pub(crate) fn write_comma_separated_list(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = impl Display>,
) -> std::fmt::Result {
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{item}")?;
    }
    Ok(())
}

/// Write input items into `'a', 'b', 'c'`
pub(crate) fn write_comma_separated_quoted_list(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = impl Display>,
) -> std::fmt::Result {
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "'{item}'")?;
    }
    Ok(())
}

/// Write input map items into `field_a=x, field_b=y`
pub(crate) fn write_comma_separated_map(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = (impl Display, impl Display)>,
) -> std::fmt::Result {
    for (i, (k, v)) in items.into_iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{k} = '{v}'")?;
    }
    Ok(())
}
