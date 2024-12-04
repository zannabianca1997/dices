use utoipa::OpenApi as _;

use super::Domain;

mod application;

mod domain;

pub const DOMAIN: Domain = Domain {
    name: "sessions",
    version: 1,
    api: application::router,
    api_docs: application::ApiDocs::openapi,
};
