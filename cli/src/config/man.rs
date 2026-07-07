use std::net::{IpAddr, Ipv4Addr};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ManConfig {
    #[serde(deserialize_with = "deserialize_bool_or_struct")]
    pub links: Option<Links>,
}

impl Default for ManConfig {
    fn default() -> Self {
        Self {
            links: if cfg!(feature = "man-server") {
                Some(Links::default())
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Links {
    /// Address for the manual server to bind to
    pub address: IpAddr,
    /// Port for the manual server
    ///
    /// If [`None`], bind to a random port
    pub port: Option<u16>,
}

impl Default for Links {
    fn default() -> Self {
        Self {
            address: Ipv4Addr::LOCALHOST.into(),
            port: None,
        }
    }
}

fn deserialize_bool_or_struct<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum BoolOrStruct<T> {
        Bool(bool),
        Struct(T),
    }

    Ok(match Deserialize::deserialize(deserializer)? {
        Some(BoolOrStruct::Bool(false)) | None => None,
        Some(BoolOrStruct::Bool(true)) => Some(T::default()),
        Some(BoolOrStruct::Struct(t)) => Some(t),
    })
}
