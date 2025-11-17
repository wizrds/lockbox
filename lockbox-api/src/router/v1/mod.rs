pub mod api_key;
pub mod namespace;
pub mod tag;

use utoipa_axum::router::OpenApiRouter;


pub fn router() -> OpenApiRouter {
    OpenApiRouter::new()
        .nest("/api_keys", api_key::router())
        .nest("/namespaces", namespace::router())
}
