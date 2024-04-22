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
use std::default::Default;
use std::fmt::Display;
use std::fmt::Formatter;

use databend_common_meta_app::schema::CreateOption;
use derive_visitor::Drive;
use derive_visitor::DriveMut;

use crate::ast::write_comma_separated_map;
use crate::ast::write_comma_separated_quoted_list;
use crate::ast::UriLocation;

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub struct CreateStageStmt {
    #[drive(skip)]
    pub create_option: CreateOption,
    #[drive(skip)]
    pub stage_name: String,

    pub location: Option<UriLocation>,

    #[drive(skip)]
    pub file_format_options: BTreeMap<String, String>,
    #[drive(skip)]
    pub on_error: String,
    #[drive(skip)]
    pub size_limit: usize,
    #[drive(skip)]
    pub validation_mode: String,
    #[drive(skip)]
    pub comments: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Drive, DriveMut)]
pub enum SelectStageOption {
    Files(#[drive(skip)] Vec<String>),
    Pattern(#[drive(skip)] String),
    FileFormat(#[drive(skip)] String),
    Connection(#[drive(skip)] BTreeMap<String, String>),
}

impl SelectStageOptions {
    pub fn from(opts: Vec<SelectStageOption>) -> Self {
        let mut options: SelectStageOptions = Default::default();
        for opt in opts.into_iter() {
            match opt {
                SelectStageOption::Files(v) => options.files = Some(v),
                SelectStageOption::Pattern(v) => options.pattern = Some(v),
                SelectStageOption::FileFormat(v) => options.file_format = Some(v),
                SelectStageOption::Connection(v) => options.connection = v,
            }
        }
        options
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Drive, DriveMut)]
pub struct SelectStageOptions {
    #[drive(skip)]
    pub files: Option<Vec<String>>,
    #[drive(skip)]
    pub pattern: Option<String>,
    #[drive(skip)]
    pub file_format: Option<String>,
    #[drive(skip)]
    pub connection: BTreeMap<String, String>,
}

impl SelectStageOptions {
    pub fn is_empty(&self) -> bool {
        self.files.is_none()
            && self.pattern.is_none()
            && self.file_format.is_none()
            && self.connection.is_empty()
    }
}

impl Display for CreateStageStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CREATE")?;
        if let CreateOption::CreateOrReplace = self.create_option {
            write!(f, " OR REPLACE")?;
        }
        write!(f, " STAGE")?;
        if let CreateOption::CreateIfNotExists = self.create_option {
            write!(f, " IF NOT EXISTS")?;
        }
        write!(f, " {}", self.stage_name)?;

        if let Some(ul) = &self.location {
            write!(f, " {ul}")?;
        }

        if !self.file_format_options.is_empty() {
            write!(f, " FILE_FORMAT = (")?;
            write_comma_separated_map(f, &self.file_format_options)?;
            write!(f, " )")?;
        }

        if !self.on_error.is_empty() {
            write!(f, " ON_ERROR = '{}'", self.on_error)?;
        }

        if self.size_limit != 0 {
            write!(f, " SIZE_LIMIT = {}", self.size_limit)?;
        }

        if !self.validation_mode.is_empty() {
            write!(f, " VALIDATION_MODE = {}", self.validation_mode)?;
        }

        if !self.comments.is_empty() {
            write!(f, " COMMENTS = '{}'", self.comments)?;
        }

        Ok(())
    }
}

// SELECT <columns> FROM
// {@<stage_name>[/<path>] | '<uri>'} [(
// [ PATTERN => '<regex_pattern>']
// [ FILE_FORMAT => '<format_name>']
// [ FILES => ( 'file_name' [ , 'file_name' ... ] ) ]
// [ ENDPOINT_URL => <'url'> ]
// [ AWS_KEY_ID => <'aws_key_id'> ]
// [ AWS_KEY_SECRET => <'aws_key_secret'> ]
// [ ACCESS_KEY_ID => <'access_key_id'> ]
// [ ACCESS_KEY_SECRET => <'access_key_secret'> ]
// [ SECRET_ACCESS_KEY => <'secret_access_key'> ]
// [ SESSION_TOKEN => <'session_token'> ]
// [ REGION => <'region'> ]
// [ ENABLE_VIRTUAL_HOST_STYLE => true|false ]
// )]
impl Display for SelectStageOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, " (")?;

        if let Some(files) = self.files.as_ref() {
            write!(f, " FILES => (")?;
            write_comma_separated_quoted_list(f, files)?;
            write!(f, "),")?;
        }

        if let Some(file_format) = self.file_format.as_ref() {
            write!(f, " FILE_FORMAT => '{}',", file_format)?;
        }

        if let Some(pattern) = self.pattern.as_ref() {
            write!(f, " PATTERN => '{}',", pattern)?;
        }

        if !self.connection.is_empty() {
            write!(f, " CONNECTION => (")?;
            write_comma_separated_map(f, &self.connection)?;
            write!(f, " )")?;
        }

        write!(f, " )")?;

        Ok(())
    }
}
