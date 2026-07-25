use serde::Deserialize;

#[derive(Deserialize)]
/// The `PageQuery` struct is meant for extracting query parameters from the request URL, specifically a ?page=N parameter. For example, /nodes?page=2.
pub struct PageQuery {
    page: Option<String>,
}
