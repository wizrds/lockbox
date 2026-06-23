// SPDX-FileCopyrightText: 2026 Timothy Pogue
//
// SPDX-License-Identifier: LicenseRef-Proprietary

pub mod calls;
pub mod client;

pub use client::ClientConfig;
pub use calls::*;

pub(crate) use client::ApiClient;

pub mod http {
    pub use reqwest::{
        Method,
        StatusCode,
        header,
        Response,
    };
}
