#![feature(error_reporter)]

use std::error::Report;

pub mod engine;
pub mod session;
pub mod session_user;
pub mod user;

fn internal_server_error<E: std::error::Error>(error: &E) {
    tracing::error!("Internal server error: {}", Report::new(error).pretty(true));
}
