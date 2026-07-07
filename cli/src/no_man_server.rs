//! Dummy manual server for binary without it

use std::{io, sync::Arc};

use snafu::Snafu;
use url::Url;

use crate::config::{man::Links, skin::Skin};

pub enum ManServer {}

#[derive(Debug, Snafu)]
#[snafu(display("This binary does not support serving manual pages"))]
pub struct Error;

impl ManServer {
    /// Start the manual server
    pub fn spawn(_: Links, _: Arc<Skin>) -> Result<Self, Error> {
        Err(Error)
    }

    /// Close the manual server
    pub fn join(self) -> io::Result<()> {
        match self {}
    }

    /// Get the server base url
    pub fn base_url(&self) -> &Url {
        match *self {}
    }
}
