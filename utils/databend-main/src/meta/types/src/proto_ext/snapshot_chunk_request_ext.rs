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

use crate::protobuf::SnapshotChunkRequest;
use crate::protobuf::SnapshotChunkV1;
use crate::InstallSnapshotRequest;

impl SnapshotChunkRequest {
    /// Return the length of the data in the chunk.
    pub fn data_len(&self) -> u64 {
        self.chunk.as_ref().map_or(0, |x| x.data.len()) as u64
    }

    pub fn new_v1(r: InstallSnapshotRequest) -> Self {
        let meta = (r.vote, r.meta);
        let rpc_meta = serde_json::to_string(&meta).unwrap();

        let chunk_v1 = SnapshotChunkV1 {
            offset: r.offset,
            done: r.done,
            data: r.data,
        };

        SnapshotChunkRequest {
            ver: 1,
            rpc_meta,
            chunk: Some(chunk_v1),
        }
    }
}
