use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ManConfig {
    pub links: Links,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Links {
    #[serde(deserialize_with = "deserialize_url_can_be_a_base")]
    pub base: Url,
}

impl Default for Links {
    fn default() -> Self {
        Self {
            base: Url::parse("https://dices.zannabianca1997.site/man").unwrap(),
        }
    }
}

fn deserialize_url_can_be_a_base<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let url = Url::deserialize(deserializer)?;
    if url.cannot_be_a_base() {
        return Err(serde::de::Error::custom(format_args!(
            "base url {url} cannot be a base"
        )));
    }
    Ok(url)
}
