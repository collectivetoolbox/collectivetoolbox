use axum::extract::FromRequestParts;
use http::StatusCode;

/// Extractor for the request URI as a string.
pub struct RequestUri(pub String);

impl<S> FromRequestParts<S> for RequestUri
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(RequestUri(parts.uri.to_string()))
    }
}
