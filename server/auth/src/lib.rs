#![feature(error_reporter)]
#![feature(duration_constructors)]

mod auth_key;
mod claims;
mod user;

pub use auth_key::{AuthConfig, AuthKey};
pub use claims::{check_password, hash_password, new_token, CheckPasswordError};

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
