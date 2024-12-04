use utoipa::OpenApi as _;

use crate::Domain;

mod application;
mod domain;
mod infrastructure;

pub const DOMAIN: Domain = Domain {
    name: "user",
    version: 1,
    api: application::router,
    api_docs: application::ApiDocs::openapi,
};
