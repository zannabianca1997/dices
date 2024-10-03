#![doc = include_str!("../README.md")]
#![feature(box_patterns)]
#![feature(never_type)]
#![feature(step_trait)]
#![feature(ptr_as_ref_unchecked)]

pub mod fmt;
pub mod ident;
pub mod intrisics;

pub mod value;
pub use value::Value;

pub mod expression;
#[cfg(feature = "parse_expression")]
pub use expression::parse_file;
pub use expression::Expression;

#[cfg(feature = "matcher")]
pub mod matcher;
#[cfg(feature = "matcher")]
pub use matcher::Matcher;

pub mod version {
    //! Versioning of the AST

    use std::borrow::Cow;

    use derive_more::derive::{Display, Error};
    use konst::{primitive::parse_u16, unwrap_ctx};

    /// Identifies the version of the AST used
    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct Version {
        pub major: u16,
        pub minor: u16,
        pub patch: u16,
        pub features: Cow<'static, [Cow<'static, str>]>,
    }
    impl Version {
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
            for feature in &*self.features {
                if !remote.features.contains(feature) {
                    return Err(IncompatibilityReason::MissingRemoteFeature {
                        feature: feature.clone(),
                    });
                }
            }
            for feature in &*remote.features {
                if !self.features.contains(feature) {
                    return Err(IncompatibilityReason::MissingLocalFeature {
                        feature: feature.clone(),
                    });
                }
            }
            Ok(())
        }
    }

    #[cfg(feature = "bincode")]
    impl bincode::Encode for Version {
        fn encode<E: bincode::enc::Encoder>(
            &self,
            encoder: &mut E,
        ) -> Result<(), bincode::error::EncodeError> {
            let Self {
                major,
                minor,
                patch,
                features,
            } = self;

            // encoding versions
            major.encode(encoder)?;
            minor.encode(encoder)?;
            patch.encode(encoder)?;

            // encoding features
            (features.len() as u64).encode(encoder)?;
            for feature in &**features {
                (&**feature).encode(encoder)?;
            }

            Ok(())
        }
    }
    #[cfg(feature = "bincode")]
    impl bincode::Decode for Version {
        fn decode<D: bincode::de::Decoder>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
            // decoding versions
            let major = u16::decode(decoder)?;
            let minor = u16::decode(decoder)?;
            let patch = u16::decode(decoder)?;

            // decoding features
            let features_len = u64::decode(decoder)?
                .try_into()
                .map_err(|_| bincode::error::DecodeError::Other("Invalid number of features"))?;
            let mut features = Vec::with_capacity(features_len);
            for _ in 0..features_len {
                features.push(Cow::Owned(String::decode(decoder)?));
            }

            Ok(Version {
                major,
                minor,
                patch,
                features: Cow::Owned(features),
            })
        }
    }

    #[derive(Debug, Clone, Display, Error)]
    pub enum IncompatibilityReason {
        #[display("The local major version ({local}) is diffent from the remote one ({remote})")]
        Major { local: u16, remote: u16 },
        #[display("The local minor version ({local}) is greather of the remote one ({remote})")]
        Minor { local: u16, remote: u16 },
        #[display("The remote is missing the `{feature}` feature")]
        MissingRemoteFeature { feature: Cow<'static, str> },
        #[display("The local is missing the `{feature}` feature")]
        MissingLocalFeature { feature: Cow<'static, str> },
    }

    pub const VERSION: Version = Version {
        major: unwrap_ctx!(parse_u16(env!("CARGO_PKG_VERSION_MAJOR"))),
        minor: unwrap_ctx!(parse_u16(env!("CARGO_PKG_VERSION_MINOR"))),
        patch: unwrap_ctx!(parse_u16(env!("CARGO_PKG_VERSION_PATCH"))),
        features: Cow::Borrowed(&[
            #[cfg(feature = "json")]
            Cow::Borrowed("json"),
        ]),
    };
}
