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

use databend_common_catalog::plan::StageTableInfo;
use databend_common_compress::CompressAlgorithm;

pub fn unload_path(
    stage_table_info: &StageTableInfo,
    uuid: &str,
    group_id: usize,
    batch_id: usize,
    compression: Option<CompressAlgorithm>,
) -> String {
    let format_name = format!(
        "{:?}",
        stage_table_info.stage_info.file_format_params.get_type()
    )
    .to_ascii_lowercase();

    let suffix: &str = &compression
        .map(|c| format!(".{}", c.extension()))
        .unwrap_or_default();

    let path = &stage_table_info.files_info.path;

    if path.ends_with("data_") {
        format!(
            "{}{}_{:0>4}_{:0>8}.{}{}",
            path, uuid, group_id, batch_id, format_name, suffix
        )
    } else {
        let (path, sep) = if path == "/" {
            ("", "")
        } else if path.ends_with('/') {
            (path.as_str(), "")
        } else {
            (path.as_str(), "/")
        };
        format!(
            "{}{}data_{}_{:0>4}_{:0>8}.{}{}",
            path, sep, uuid, group_id, batch_id, format_name, suffix
        )
    }
}
