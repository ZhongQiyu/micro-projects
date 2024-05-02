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

//! Defines kvapi::KVApi key behaviors.

use std::convert::Infallible;
use std::fmt::Debug;
use std::string::FromUtf8Error;

use crate::kvapi;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeyError {
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Non-ascii char are not supported: '{non_ascii}'")]
    AsciiError { non_ascii: String },

    #[error("Expect {i}-th segment to be '{expect}', but: '{got}'")]
    InvalidSegment {
        i: usize,
        expect: String,
        got: String,
    },

    #[error("Expect {i}-th segment to be non-empty")]
    EmptySegment { i: usize },

    #[error("Expect {expect} segments, but: '{got}'")]
    WrongNumberOfSegments { expect: usize, got: String },

    #[error("Expect at least {expect} segments, but {actual} segments found")]
    AtleastSegments { expect: usize, actual: usize },

    #[error("Invalid id string: '{s}': {reason}")]
    InvalidId { s: String, reason: String },

    #[error("Unknown kvapi::Key prefix: '{prefix}'")]
    UnknownPrefix { prefix: String },
}

/// Convert structured key to a string key used by kvapi::KVApi and backwards
pub trait Key: Debug
where Self: Sized
{
    const PREFIX: &'static str;

    type ValueType: kvapi::Value;

    /// Return the root prefix of this key: `"<PREFIX>/"`.
    fn root_prefix() -> String {
        format!("{}/", Self::PREFIX)
    }

    /// Return the parent key of this key.
    ///
    /// For example, a table name's parent is db-id.
    fn parent(&self) -> Option<String>;

    /// Encode structured key into a string.
    fn to_string_key(&self) -> String;

    /// Decode str into a structured key.
    fn from_str_key(s: &str) -> Result<Self, kvapi::KeyError>;
}

impl kvapi::Key for String {
    const PREFIX: &'static str = "";

    /// For a non structured key, the value type can never be used.
    type ValueType = Infallible;

    fn parent(&self) -> Option<String> {
        unimplemented!("illegal to get parent of generic String key")
    }

    fn to_string_key(&self) -> String {
        self.clone()
    }

    fn from_str_key(s: &str) -> Result<Self, kvapi::KeyError> {
        Ok(s.to_string())
    }
}

/// The dir name of a key.
///
/// For example, the dir name of a key `a/b/c` is `a/b`.
///
/// Note that the dir name of `a` is still `a`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirName<K> {
    key: K,
    level: usize,
}

impl<K> DirName<K> {
    pub fn new(key: K) -> Self {
        DirName { key, level: 1 }
    }

    pub fn new_with_level(key: K, level: usize) -> Self {
        DirName { key, level }
    }

    pub fn with_level(&mut self, level: usize) -> &mut Self {
        self.level = level;
        self
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn into_key(self) -> K {
        self.key
    }
}

impl<K: Key> Key for DirName<K> {
    const PREFIX: &'static str = K::PREFIX;
    type ValueType = K::ValueType;

    fn parent(&self) -> Option<String> {
        unimplemented!("DirName is not a record thus it has no parent")
    }

    fn to_string_key(&self) -> String {
        let k = self.key.to_string_key();
        k.rsplitn(self.level + 1, '/').last().unwrap().to_string()
    }

    fn from_str_key(s: &str) -> Result<Self, KeyError> {
        let d = DirName::new_with_level(K::from_str_key(s)?, 0);
        Ok(d)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::DirName;
    use crate::kvapi::Key;
    use crate::kvapi::KeyError;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct FooKey {
        a: u64,
        b: String,
        c: u64,
    }

    impl Key for FooKey {
        const PREFIX: &'static str = "pref";
        type ValueType = Infallible;

        fn parent(&self) -> Option<String> {
            None
        }

        fn to_string_key(&self) -> String {
            format!("{}/{}/{}/{}", Self::PREFIX, self.a, self.b, self.c)
        }

        fn from_str_key(_s: &str) -> Result<Self, KeyError> {
            // dummy impl
            let k = FooKey {
                a: 9,
                b: "x".to_string(),
                c: 8,
            };
            Ok(k)
        }
    }

    #[test]
    fn test_dir_name_from_key() {
        let d = DirName::<FooKey>::from_str_key("").unwrap();
        assert_eq!(
            FooKey {
                a: 9,
                b: "x".to_string(),
                c: 8,
            },
            d.into_key()
        );
    }

    #[test]
    fn test_dir_name() {
        let k = FooKey {
            a: 1,
            b: "b".to_string(),
            c: 2,
        };

        let dir = DirName::new(k);
        assert_eq!("pref/1/b", dir.to_string_key());

        let dir = DirName::new(dir);
        assert_eq!("pref/1", dir.to_string_key());

        let dir = DirName::new(dir);
        assert_eq!("pref", dir.to_string_key());

        let dir = DirName::new(dir);
        assert_eq!("pref", dir.to_string_key(), "root dir should be the same");
    }

    #[test]
    fn test_dir_name_with_level() {
        let k = FooKey {
            a: 1,
            b: "b".to_string(),
            c: 2,
        };

        let mut dir = DirName::new(k);
        assert_eq!("pref/1/b", dir.to_string_key());

        dir.with_level(0);
        assert_eq!("pref/1/b/2", dir.to_string_key());

        dir.with_level(2);
        assert_eq!("pref/1", dir.to_string_key());

        dir.with_level(3);
        assert_eq!("pref", dir.to_string_key());

        dir.with_level(4);
        assert_eq!("pref", dir.to_string_key(), "root dir should be the same");
    }
}
