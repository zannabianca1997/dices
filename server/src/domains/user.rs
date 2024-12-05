use utoipa::OpenApi as _;

use super::Domain;

mod application;
mod domain;
mod infrastructure;

pub use domain::security::AutenticatedUser;

pub const DOMAIN: Domain = Domain {
    name: "user",
    version: 1,
    api: application::router,
    api_docs: application::ApiDocs::openapi,
};
