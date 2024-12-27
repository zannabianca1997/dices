#![feature(error_reporter)]
#![feature(duration_constructors)]
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify,
};

mod auth_key;
mod claims;
mod user;

pub use auth_key::{AuthConfig, AuthKey};
pub use claims::{check_password, hash_password, new_token, CheckPasswordError};
pub use user::RequireUserToken;

/// An object that was requested by an autenticated user
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Autenticated<T>(T);

impl<T> std::ops::Deref for Autenticated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> Autenticated<T> {
    pub fn inner(&self) -> &T {
        &self.0
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();

        let mut http = Http::new(HttpAuthScheme::Bearer);
        http.bearer_format = Some("JWT".to_owned());
        http.description =
            Some("A jwt token obtained either from the `/signin` or `/signup` endpoint".to_owned());
        components.add_security_scheme("user_token", SecurityScheme::Http(http))
    }
}
