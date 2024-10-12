//! Versioning of the AST

use derive_more::derive::{Display, Error};
pub use konst::{primitive::parse_u16, unwrap_ctx};

/// Identifies the version of the AST used
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
impl Version {
    /// Construct a new version
    pub const fn new(major: &str, minor: &str, patch: &str) -> Self {
        Self {
            major: unwrap_ctx!(parse_u16(major)),
            minor: unwrap_ctx!(parse_u16(minor)),
            patch: unwrap_ctx!(parse_u16(patch)),
        }
    }

    /// Check if this version is compatible with the remote one
    pub fn is_compatible_with(&self, remote: &Self) -> Result<(), IncompatibilityReason> {
        if self.major != remote.major {
            return Err(IncompatibilityReason::Major {
                local: self.major,
                remote: remote.major,
            });
        }
        if self.minor > remote.minor {
            return Err(IncompatibilityReason::Minor {
                local: self.minor,
                remote: remote.minor,
            });
        }
        // patch cannot add incompatibilities, so no need to check
        Ok(())
    }
}

#[derive(Debug, Clone, Display, Error)]
pub enum IncompatibilityReason {
    #[display("The local major version ({local}) is diffent from the remote one ({remote})")]
    Major { local: u16, remote: u16 },
    #[display("The local minor version ({local}) is greather of the remote one ({remote})")]
    Minor { local: u16, remote: u16 },
}

pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);
