use axum::extract::FromRequestParts;
use http::StatusCode;
use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub url: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct RequestState {
    pub route: String,
    pub method: String,
    pub accept: Option<String>,
    pub is_js_request: bool,
    pub is_embedded: bool,
    pub breadcrumbs: Option<Vec<BreadcrumbItem>>,
    pub logged_in_user: Option<String>,
    pub back_url: Option<String>,
}

fn query_has_embed_flag(query: Option<&str>) -> bool {
    query.is_some_and(|query| {
        query.split('&').any(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or_default();
            let value = parts.next().unwrap_or_default();
            key == "embed"
                && (value == "1" || value.eq_ignore_ascii_case("true"))
        })
    })
}

pub fn generate_breadcrumbs(route: &str) -> Option<Vec<BreadcrumbItem>> {
    if route.is_empty() || route == "/" {
        return None;
    }

    if route == "/home" {
        return Some(vec![BreadcrumbItem {
            label: "Home".to_string(),
            url: None,
        }]);
    }

    if route == "/nodes" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Nodes".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/nodes/create" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Nodes".to_string(),
                url: Some("/nodes".to_string()),
            },
            BreadcrumbItem {
                label: "Create".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/nodes/upload" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Nodes".to_string(),
                url: Some("/nodes".to_string()),
            },
            BreadcrumbItem {
                label: "Upload".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/nodes/view" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Nodes".to_string(),
                url: Some("/nodes".to_string()),
            },
            BreadcrumbItem {
                label: "View".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/search" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Search".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/debug/db-tables" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Database Tables".to_string(),
                url: None,
            },
        ]);
    }

    if let Some(table_name) = route.strip_prefix("/debug/db-tables/") {
        if !table_name.is_empty() {
            return Some(vec![
                BreadcrumbItem {
                    label: "Home".to_string(),
                    url: Some("/home".to_string()),
                },
                BreadcrumbItem {
                    label: "Database Tables".to_string(),
                    url: Some("/debug/db-tables".to_string()),
                },
                BreadcrumbItem {
                    label: table_name.to_string(),
                    url: None,
                },
            ]);
        }
    }

    if route == "/pc-settings" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Settings".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/home/subscribe" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Subscribe".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/privacy-policy" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Privacy Policy".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/login" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Login".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/registration" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Registration".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/newsletters" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Newsletters".to_string(),
                url: None,
            },
        ]);
    }

    if let Some(date) = route.strip_prefix("/newsletters/") {
        if !date.is_empty() {
            return Some(vec![
                BreadcrumbItem {
                    label: "Home".to_string(),
                    url: Some("/home".to_string()),
                },
                BreadcrumbItem {
                    label: "Newsletters".to_string(),
                    url: Some("/newsletters".to_string()),
                },
                BreadcrumbItem {
                    label: date.to_string(),
                    url: None,
                },
            ]);
        }
    }

    if route == "/docs/LICENSE.md" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "License".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/docs/TRADEMARKS.md" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Trademark Policy".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/docs/CHANGELOG.md" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Changelog".to_string(),
                url: None,
            },
        ]);
    }

    if route == "/docs" {
        return Some(vec![
            BreadcrumbItem {
                label: "Home".to_string(),
                url: Some("/home".to_string()),
            },
            BreadcrumbItem {
                label: "Documentation".to_string(),
                url: None,
            },
        ]);
    }

    if let Some(doc_path) = route.strip_prefix("/docs/") {
        if !doc_path.is_empty() {
            let mut items = vec![
                BreadcrumbItem {
                    label: "Home".to_string(),
                    url: Some("/home".to_string()),
                },
                BreadcrumbItem {
                    label: "Documentation".to_string(),
                    url: Some("/docs".to_string()),
                },
            ];
            let parts: Vec<&str> = doc_path.split('/').filter(|s| !s.is_empty()).collect();
            let mut current_subpath = "/docs".to_string();
            for part in &parts {
                let part_clean = if part.to_ascii_lowercase().ends_with(".md") {
                    part.get(..part.len().saturating_sub(3)).unwrap_or("")
                } else {
                    *part
                };

                if part_clean.eq_ignore_ascii_case("index") {
                    continue;
                }

                current_subpath.push('/');
                current_subpath.push_str(part);
                let label = format_segment_label(part_clean);
                items.push(BreadcrumbItem {
                    label,
                    url: Some(current_subpath.clone()),
                });
            }
            if let Some(last) = items.last_mut() {
                last.url = None;
            }
            return Some(items);
        }
    }

    None
}

fn format_segment_label(segment: &str) -> String {
    let mut label = String::new();
    let mut capitalize_next = true;
    for c in segment.chars() {
        if c == '-' || c == '_' {
            label.push(' ');
            capitalize_next = true;
        } else if capitalize_next {
            label.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            label.push(c);
        }
    }
    label
}

fn parse_relative_back_url(val: &str) -> Option<String> {
    let path_part = val.split('?').next().unwrap_or(val);
    let bytes = path_part.as_bytes();
    if path_part.starts_with('/')
        && bytes.get(1) != Some(&b'/')
        && bytes.get(1) != Some(&b'\\')
    {
        Some(val.to_string())
    } else {
        None
    }
}

impl<S> FromRequestParts<S> for RequestState
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let is_embedded = query_has_embed_flag(parts.uri.query());
        let route = parts.uri.path().to_string();
        let breadcrumbs = if is_embedded {
            None
        } else {
            generate_breadcrumbs(&route)
        };

        let session_key_bytes = crate::session_auth::session_key_from_headers(&parts.headers);
        let mut logged_in_user = None;
        if let Some(session_key_bytes) = session_key_bytes {
            use base64::Engine;
            let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&session_key_bytes);
            if let Ok(Some(user_id)) = ctb_storage::user::validate_session(&token) {
                if let Some(public_info) = ctb_storage::user::UserPublicInfo::get_by_id(user_id) {
                    logged_in_user = Some(public_info.name().to_string());
                }
            }
        }

        let back_url = parts
            .headers
            .get("X-Back-Url")
            .or_else(|| parts.headers.get(axum::http::header::REFERER))
            .and_then(|v| v.to_str().ok())
            .and_then(parse_relative_back_url);

        Ok(RequestState {
            route,
            method: parts.method.to_string(),
            accept: parts
                .headers
                .get(axum::http::header::ACCEPT)
                .and_then(|v| v.to_str().ok())
                .map(ToOwned::to_owned),
            is_js_request: parts
                .headers
                .get("X-CollectiveToolbox-IsJsRequest")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|s| s.eq_ignore_ascii_case("true")),
            is_embedded,
            breadcrumbs,
            logged_in_user,
            back_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_breadcrumbs_various_routes() {
        let home = generate_breadcrumbs("/home").unwrap();
        assert_eq!(home.len(), 1);
        assert_eq!(home[0].label, "Home");
        assert_eq!(home[0].url, None);

        let privacy = generate_breadcrumbs("/privacy-policy").unwrap();
        assert_eq!(privacy.len(), 2);
        assert_eq!(privacy[0].label, "Home");
        assert_eq!(privacy[0].url, Some("/home".to_string()));
        assert_eq!(privacy[1].label, "Privacy Policy");
        assert_eq!(privacy[1].url, None);

        // 2. License and Trademark Policy special cases
        let license = generate_breadcrumbs("/docs/LICENSE.md").unwrap();
        assert_eq!(license.len(), 2);
        assert_eq!(license[0].label, "Home");
        assert_eq!(license[1].label, "License");
        assert_eq!(license[1].url, None);

        let trademark = generate_breadcrumbs("/docs/TRADEMARKS.md").unwrap();
        assert_eq!(trademark.len(), 2);
        assert_eq!(trademark[0].label, "Home");
        assert_eq!(trademark[1].label, "Trademark Policy");
        assert_eq!(trademark[1].url, None);

        let changelog = generate_breadcrumbs("/docs/CHANGELOG.md").unwrap();
        assert_eq!(changelog.len(), 2);
        assert_eq!(changelog[0].label, "Home");
        assert_eq!(changelog[1].label, "Changelog");
        assert_eq!(changelog[1].url, None);

        // 3. Regular documentation index
        let docs = generate_breadcrumbs("/docs").unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].label, "Home");
        assert_eq!(docs[1].label, "Documentation");
        assert_eq!(docs[1].url, None);

        // 4. index.md omission
        let index_md = generate_breadcrumbs("/docs/index.md").unwrap();
        assert_eq!(index_md.len(), 2);
        assert_eq!(index_md[0].label, "Home");
        assert_eq!(index_md[1].label, "Documentation");
        assert_eq!(index_md[1].url, None);

        // 5. Nested paths with and without .md
        let nested_file = generate_breadcrumbs("/docs/some-dir/some-file.md").unwrap();
        assert_eq!(nested_file.len(), 4);
        assert_eq!(nested_file[0].label, "Home");
        assert_eq!(nested_file[1].label, "Documentation");
        assert_eq!(nested_file[2].label, "Some Dir");
        assert_eq!(nested_file[2].url, Some("/docs/some-dir".to_string()));
        assert_eq!(nested_file[3].label, "Some File");
        assert_eq!(nested_file[3].url, None);

        let nested_index = generate_breadcrumbs("/docs/some-dir/index.md").unwrap();
        assert_eq!(nested_index.len(), 3);
        assert_eq!(nested_index[0].label, "Home");
        assert_eq!(nested_index[1].label, "Documentation");
        assert_eq!(nested_index[2].label, "Some Dir");
        assert_eq!(nested_index[2].url, None);
    }

    #[crate::ctb_test]
    fn test_parse_relative_back_url() {
        assert_eq!(parse_relative_back_url("/search?q=test"), Some("/search?q=test".to_string()));
        assert_eq!(parse_relative_back_url("/search?q=a:b"), Some("/search?q=a:b".to_string()));
        assert_eq!(parse_relative_back_url("/a:b"), Some("/a:b".to_string()));
        assert_eq!(parse_relative_back_url("a:b"), None);
        assert_eq!(parse_relative_back_url("//foo.com/"), None);
        assert_eq!(parse_relative_back_url("/\\foo.com/"), None);
        assert_eq!(parse_relative_back_url("http://localhost/nodes"), None);
    }
}

