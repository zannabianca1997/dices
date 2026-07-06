use std::path::PathBuf;

use figment::value::magic::{Either, RelativePathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryConfig {
    file: Either<RelativePathBuf, Option<PathBuf>>,
    pub capacity: usize,
}

impl HistoryConfig {
    pub fn file(&self) -> Option<PathBuf> {
        match &self.file {
            Either::Left(r) => Some(r.relative()),
            Either::Right(Some(f)) => Some(f.clone()),
            Either::Right(None) => None,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            file: Either::Right(
                super::directories().map(|dirs| dirs.data_dir().join("history.txt")),
            ),
            capacity: 1000,
        }
    }
}
