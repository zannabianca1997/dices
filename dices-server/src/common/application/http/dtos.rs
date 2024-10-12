use derive_more::derive::{Constructor, From};
use dices_ast::version::Version;

#[derive(Debug, serde::Serialize, Constructor, From)]
pub(super) struct VersionDto(Version);
