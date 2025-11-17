//! Custom server configuration that extends Dioxus with OpenCode-compatible REST API

use axum::Router;
use dioxus::prelude::*;

/// Extend the Axum router with OpenCode-compatible REST routes
pub fn extend_router(router: Router) -> Router {
    // Merge OpenCode-compatible REST routes
    let rest_api = api::create_router();
    router.merge(rest_api)
}
