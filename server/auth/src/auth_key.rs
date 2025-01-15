use std::{sync::Arc, time::Duration};

use hmac::{Hmac, Mac};
use jwt::{SigningAlgorithm, VerifyingAlgorithm};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
use sha2::Sha256;

#[serde_as]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AuthConfig {
    /// The validity of the token
    #[serde_as(as = "DurationSeconds")]
    pub token_validity: Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_validity: Duration::from_secs(5 * 60),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthKey {
    key: Arc<Hmac<Sha256>>,
    token_validity: Duration,
}
impl AuthKey {
    pub fn new(AuthConfig { token_validity }: AuthConfig) -> Self {
        tracing::info!("Generating authentication key");
        let secret: [u8; 32] = thread_rng().gen();
        Self {
            key: Arc::new(Hmac::new_from_slice(&secret).unwrap()),
            token_validity,
        }
    }

    pub(super) const fn token_validity(&self) -> Duration {
        self.token_validity
    }
}
impl SigningAlgorithm for AuthKey {
    fn algorithm_type(&self) -> jwt::AlgorithmType {
        SigningAlgorithm::algorithm_type(&*self.key)
    }

    fn sign(&self, header: &str, claims: &str) -> Result<String, jwt::Error> {
        self.key.sign(header, claims)
    }
}
impl VerifyingAlgorithm for AuthKey {
    fn algorithm_type(&self) -> jwt::AlgorithmType {
        VerifyingAlgorithm::algorithm_type(&*self.key)
    }

    fn verify_bytes(
        &self,
        header: &str,
        claims: &str,
        signature: &[u8],
    ) -> Result<bool, jwt::Error> {
        self.key.verify_bytes(header, claims, signature)
    }

    fn verify(&self, header: &str, claims: &str, signature: &str) -> Result<bool, jwt::Error> {
        VerifyingAlgorithm::verify(&*self.key, header, claims, signature)
    }
}
