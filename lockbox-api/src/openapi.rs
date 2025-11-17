use utoipa::OpenApi;


#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "ping", description = "Server ping endpoint"),
        (name = "api_keys", description = "API keys management"),
        (name = "namespaces", description = "API key namespaces management"),
        (name = "tags", description = "API key namespace tags management"),
    )
)]
pub struct ApiDoc;
