// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs: MIT
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// Based on:
// <https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs>
/* MIT License

Copyright (c) 2019–2025 axum Contributors

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.*/

//! Anyhow error handling support for Axum.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::AppState;
use crate::RequestState;
use crate::error_500;
use axum::response::{IntoResponse, Response};

/// A custom result type for web handlers.
pub type WebResult<T> = std::result::Result<T, WebError>;

// Make our own error that wraps `anyhow::Error` and includes state.
pub struct WebError {
    error: anyhow::Error,
    state: AppState,
    req: RequestState,
}

impl WebError {
    /// Create a new `WebError` with the given error, state, and request.
    pub fn new<E>(error: E, state: AppState, req: RequestState) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self {
            error: error.into(),
            state,
            req,
        }
    }
}

// Tell axum how to convert `WebError` into a response.
impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        error_500(&self.state, &self.req, self.error)
    }
}

/// A trait to extend `Result` and `Option` with error handling that injects `AppState` and `RequestState`.
pub trait WebErr<T> {
    fn web_err(self, state: &AppState, req: &RequestState) -> WebResult<T>;
}

impl<T, E> WebErr<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn web_err(self, state: &AppState, req: &RequestState) -> WebResult<T> {
        self.map_err(|e| WebError::new(e, state.clone(), req.clone()))
    }
}

impl<T> WebErr<T> for Option<T> {
    fn web_err(self, state: &AppState, req: &RequestState) -> WebResult<T> {
        self.ok_or_else(|| {
            WebError::new(
                anyhow::anyhow!("Unexpected None"),
                state.clone(),
                req.clone(),
            )
        })
    }
}
