# lockbox-sdk

Rust client SDK for the Lockbox API service.

Lockbox manages namespaces, tags, and API keys. This SDK provides a typed,
async client for each of those resources.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lockbox-sdk = { git = "https://github.com/wizrds/lockbox.git", rev = "main" }
tokio = { version = "1", features = ["full"] }
```

## Quickstart

```rust
use lockbox_sdk::{ClientConfig, LockboxApiClient};
use lockbox_sdk::types::api_keys::CreateApiKeyRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lockbox = LockboxApiClient::new(
        ClientConfig::new()
            .with_base_url("https://service.example.com")
            .with_header("X-Tenant-Id", "my-tenant"),
    );

    // Requests are awaited directly; there is no separate `.send()` step.
    let created = lockbox
        .api_keys()
        .create_api_key(&CreateApiKeyRequest {
            owner: "service-account".to_string(),
            ..Default::default()
        })
        .await?;

    let key = lockbox.api_keys().get_api_key(&created.id).await?;

    println!("{key:?}");

    Ok(())
}
```

## Clients

Each resource has its own client, reached from the main SDK client:

- `lockbox.namespaces()`
- `lockbox.tags()`
- `lockbox.api_keys()`
- `lockbox.ping()`, which pings the API and returns its name and version

The request and response types for each resource live under
`lockbox_sdk::types::{namespaces, tags, api_keys}` and are named
`X..Request` / `X..Response`. See the rustdoc for the full set of methods on
each client and the fields on each type.

The SDK client accepts a `ClientConfig`:

```rust
use lockbox_sdk::ClientConfig;

let config = ClientConfig::new()
    .with_base_url("https://service.example.com")
    .with_header("X-Tenant-Id", "my-tenant");
```

Tenant ID and auth tokens should be passed via `with_header`. The SDK does not
inject them automatically.

A custom `reqwest_middleware::ClientWithMiddleware` can be provided via
`with_client` to attach middleware (retry, tracing, etc.).

## Requests

Every client method returns an awaitable `Call`. Awaiting it sends the request;
there is no separate `.send()` step. Before awaiting, a single request can be
customized:

```rust
use std::time::Duration;

let key = lockbox
    .api_keys()
    .get_api_key(&id)
    .header("X-Request-Id", "abc-123")
    .timeout(Duration::from_secs(5))
    .await?;
```

Use `.with(|req| ...)` to apply an arbitrary closure to the underlying
`reqwest` request builder when the typed helpers are not enough.

## Pagination

`find_*` methods take an optional `Find*Params`, built with a fluent API, and
return a `Page<T>`:

```rust
use lockbox_sdk::types::api_keys::FindApiKeysParams;

let page = lockbox
    .api_keys()
    .find_api_keys(Some(
        &FindApiKeysParams::new()
            .owners(["service-account"])
            .per_page(50),
    ))
    .await?;

for key in &page.items {
    println!("{}", key.id);
}

// `page.next_page` and `page.previous_page` are `Some` when more pages exist.
```

Passing `None` uses the server's defaults.

## Error Handling

All methods return `Result<T, lockbox_sdk::errors::Error>`.

```rust
use lockbox_sdk::errors::{Error, ErrorBody};

match lockbox.api_keys().get_api_key(&key_id).await {
    Err(Error::Api { status, body: ErrorBody::Validation { fields, .. } }) => {
        for field in fields {
            eprintln!("{}: {:?}", field.field, field.errors);
        }
    }
    Err(Error::Api { status, body: ErrorBody::Generic { message, .. } }) => {
        eprintln!("API error {status}: {message}");
    }
    Err(e) => eprintln!("{e}"),
    Ok(key) => println!("{key:?}"),
}
```
