// SPDX-FileCopyrightText: 2026 Timothy Pogue
//
// SPDX-License-Identifier: LicenseRef-Proprietary

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::errors::{Error, ErrorBody};


const DEFAULT_BASE_URL: &str = "http://localhost:8087";

pub struct ClientConfig {
    pub base_url: String,
    pub default_headers: HeaderMap,
    pub client: Option<reqwest_middleware::ClientWithMiddleware>,
}

impl ClientConfig {
    pub fn new() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            default_headers: HeaderMap::new(),
            client: None,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.default_headers.insert(
            HeaderName::from_bytes(key.as_ref().as_bytes()).unwrap(),
            HeaderValue::from_str(value.as_ref()).unwrap(),
        );
        self
    }

    pub fn with_client(mut self, client: reqwest_middleware::ClientWithMiddleware) -> Self {
        self.client = Some(client);
        self
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// The low-level transport engine that [`Call`](crate::base::Call)s drive.
#[derive(Clone)]
pub(crate) struct ApiClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base_url: String,
    default_headers: HeaderMap,
}

impl ApiClient {
    pub(crate) fn from_config(config: ClientConfig) -> Self {
        Self {
            client: config.client.unwrap_or_else(|| {
                reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build()
            }),
            base_url: config.base_url,
            default_headers: config.default_headers,
        }
    }

    /// Builds a request against `base_url + url` with the default headers,
    /// appending `?query` when a non-empty query string is given.
    pub(crate) fn builder(
        &self,
        method: reqwest::Method,
        url: &str,
        query: Option<&str>,
    ) -> reqwest_middleware::RequestBuilder {
        self.client
            .request(method, match query {
                Some(q) if !q.is_empty() => format!("{}{}?{}", self.base_url, url, q),
                _ => format!("{}{}", self.base_url, url),
            })
            .headers(self.default_headers.clone())
    }

    /// Sends a request, converting any non-2xx response into [`Error::Api`].
    pub(crate) async fn send(
        &self,
        builder: reqwest_middleware::RequestBuilder,
    ) -> Result<reqwest::Response, Error> {
        let response = builder.send().await?;

        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();

        Err(Error::Api {
            status,
            body: response
                .json::<ErrorBody>()
                .await
                .unwrap_or(ErrorBody::Generic {
                    code: status as u32,
                    message: "unknown error".to_string(),
                    metadata: None,
                }),
        })
    }
}
