// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! Axum extractor that can handle both `application/x-www-form-urlencoded` and
//! `multipart/form-data` content types, using `axum::Form` and
//! `axum_typed_multipart::TypedMultipart` internally.
//!
//! It will only work for forms that do not have file uploads, as those are
//! represented as Bytes objects which can't be deserialized. For those, just
//! use `TypedMultipart`.

use axum::extract::{FromRequest, Request};
use axum_extra::extract::{Form, FormRejection};
use axum_typed_multipart::{
    TryFromMultipartWithState, TypedMultipart, TypedMultipartError,
};

pub struct FlexibleForm<T>(pub T);

/// Custom rejection for unsupported content types.
#[derive(Debug)]
pub struct UnsupportedContentTypeRejection {
    pub error: String,
}

// Custom rejection type to handle both errors
#[derive(Debug)]
pub enum FlexibleFormRejection {
    Multipart(TypedMultipartError),
    Form(FormRejection),
    UnsupportedContentType(UnsupportedContentTypeRejection),
}

impl<T, S> FromRequest<S> for FlexibleForm<T>
where
    T: TryFromMultipartWithState<S> + serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = FlexibleFormRejection;

    /// Attempts to extract the form data first as a Form, then as `TypedMultipart`.
    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            // Reason for fallback: request missing Content-Type header defaults to empty content type string
            .unwrap_or("");

        match content_type {
            ct if ct.starts_with("application/x-www-form-urlencoded") => {
                // Try as form
                match Form::<T>::from_request(req, state).await {
                    Ok(Form(data)) => Ok(FlexibleForm(data)),
                    Err(form_err) => Err(FlexibleFormRejection::Form(form_err)),
                }
            }
            ct if ct.starts_with("multipart/form-data") => {
                // Try as multipart
                match TypedMultipart::<T>::from_request(req, state).await {
                    Ok(TypedMultipart(data)) => Ok(FlexibleForm(data)),
                    Err(multipart_err) => {
                        Err(FlexibleFormRejection::Multipart(multipart_err))
                    }
                }
            }
            &_ => Err(FlexibleFormRejection::UnsupportedContentType(
                UnsupportedContentTypeRejection {
                    error: "Unsupported content type".into(),
                },
            )),
        }
    }
}

// Implement IntoResponse for the rejection if needed (for error handling)
impl axum::response::IntoResponse for FlexibleFormRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            FlexibleFormRejection::Multipart(e) => e.into_response(),
            FlexibleFormRejection::Form(e) => e.into_response(),
            FlexibleFormRejection::UnsupportedContentType(e) => {
                use axum::http::StatusCode;
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, e.error).into_response()
            }
        }
    }
}
