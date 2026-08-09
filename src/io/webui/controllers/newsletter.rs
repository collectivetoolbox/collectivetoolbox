//! Controller for the newsletters pages.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, header},
    response::{Html, IntoResponse, Response},
};
use chrono::NaiveDate;
use ctb_formats_html::text_clipper::{ClipHtmlOptions, clip, html_options};
use ctb_formats_syndication::{Entry, Feed, FeedFormat, Image};
use ctb_storage::get_asset;
use pc_settings::PcSettingStrKey;
use std::fmt::Write;

use crate::{
    AppState, RequestState, error_400, error_404, render_page,
    respond_page_literal,
};

pub async fn get_newsletters_intro(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let page = get_asset("views/pages/newsletters-intro.html");
    let Some(page) = page else {
        return error_404(
            &state,
            &req,
            "Newsletter introduction page not found".to_string(),
        );
    };

    let mut page = String::from_utf8_lossy(&page).to_string();
    match render_newsletter_post_index_hatom() {
        Ok(index_html) => {
            page.push('\n');
            page.push_str(index_html.as_str());
        }
        Err(e) => return error_400(&state, &req, e),
    }

    match render_page(
        &state.hbs,
        Some("page"),
        "layouts.literal".to_string(),
        &req,
        &crate::json_value!({
            "page" => page,
        }),
    ) {
        Ok(html) => Html(html).into_response(),
        Err(e) => error_400(&state, &req, e),
    }
}

pub async fn get_newsletter(
    State(state): State<AppState>,
    req: RequestState,
    Path(date): Path<String>,
) -> Response {
    // Only serve newsletters that have a real date. Drafts like `2026-01-dd.md`
    // should not be routable.
    if NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").is_err() {
        return error_404(
            &state,
            &req,
            format!("Newsletter not found (invalid date): {date}"),
        );
    }

    let md = ctb_storage::get_asset(
        format!("views/pages/newsletters/{date}.md").as_str(),
    );

    let Some(md) = md else {
        return error_404(
            &state,
            &req,
            format!("Newsletter not found: {date}"),
        );
    };

    let heading = get_title_for_date(&date);
    let md = format!("# {heading}\n\n{}", String::from_utf8_lossy(&md));

    // Content embedded as assets should be safe
    let page = match ctb_formats_markdown::markdown2html_unsafe(md.into()) {
        Ok(page) => page,
        Err(err) => {
            return error_400(
                &state,
                &req,
                format!("Failed to render newsletter markdown: {err}"),
            );
        }
    };
    let page = String::from_utf8_lossy(&page);

    respond_page_literal(&state, req, page.as_ref())
}

pub async fn get_newsletters_syndication(
    state: State<AppState>,
    req: RequestState,
    Path(format): Path<String>,
) -> Response {
    match format.as_str() {
        "rss" => get_newsletters_rss(state, req),
        "atom" => get_newsletters_atom(state, req),
        "json" => get_newsletters_json(state, req),
        "rss-1.xml" => get_newsletters_rss_1(state, req),
        "rss-092.xml" => get_newsletters_rss_092(state, req),
        "rss-091ul.xml" => get_newsletters_rss_091ul(state, req),
        "rss-091ns.xml" => get_newsletters_rss_091ns(state, req),
        "rss-09.rdf" => get_newsletters_rss_09(state, req),
        "sn-02.xml" => get_newsletters_scripting_news_02(state, req),
        "sn-01.xml" => get_newsletters_scripting_news_01(state, req),
        _ => error_404(
            &state,
            &req,
            format!("Unsupported feed format requested: {format}"),
        ),
    }
}

fn newsletters_feed(
    format: FeedFormat,
    feed_path: &str,
    include_body_html: bool,
    force_http_urls: bool,
) -> Result<String> {
    let server_url = server_url();
    let server_url = if force_http_urls {
        server_url_http_only(&server_url)
    } else {
        server_url
    };

    let home_page_url = format!("{server_url}/newsletters");
    let feed_url = format!("{server_url}{feed_path}");
    let channel_image =
        Image::new().with_url(format!("{server_url}/channel.gif"));

    let mut feed = Feed::empty("Collective Toolbox Newsletters")
        .with_home_page_url_opt(Some(home_page_url))
        .with_feed_url_opt(Some(feed_url))
        .with_image(channel_image);

    for (_date, date_str) in list_newsletter_posts()? {
        let url = format!("{server_url}/newsletters/{date_str}");
        let title = format!("Collective Toolbox Newsletter for {date_str}");

        let body_html = if include_body_html {
            let md = ctb_storage::get_asset(
                format!("views/pages/newsletters/{date_str}.md").as_str(),
            );
            if let Some(md) = md {
                // Content embedded as assets should be safe
                let page = ctb_formats_markdown::markdown2html_unsafe(md)?;
                String::from_utf8_lossy(&page).to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let published = format!("{date_str}T00:00:00Z");
        let entry = Entry::new(url.clone(), title, body_html, published)
            .with_url_opt(Some(url))
            .with_author("Collective Toolbox Developers".to_string());
        feed.add_entry(entry);
    }

    feed.to(format).context("Failed to render newsletters feed")
}

fn server_url() -> String {
    // Reason for fallback: unconfigured server URL setting defaults to default_url constant
    pc_settings::get_str_setting(PcSettingStrKey::ServerUrl)
        .unwrap_or_else(default_url)
}

fn server_url_http_only(server_url: &str) -> String {
    if server_url.starts_with("http://") {
        return server_url.to_string();
    }
    if let Some(rest) = server_url.strip_prefix("https://") {
        return format!("http://{rest}");
    }
    // RSS 0.9/0.91 renderers have strict URL requirements; fall back to a
    // known-good default if configuration provides an unsupported scheme.
    if let Some(rest) = default_url().strip_prefix("https://") {
        return format!("http://{rest}");
    }
    format!("http://{}", default_domain())
}

fn list_newsletter_posts() -> Result<Vec<(NaiveDate, String)>> {
    let mut posts: Vec<(NaiveDate, String)> = Vec::new();

    for path in ctb_storage::find_assets("views/pages/newsletters/*.md")? {
        let Some(file_name) = path.rsplit('/').next() else {
            continue;
        };
        let Some(date_str) = file_name.strip_suffix(".md") else {
            continue;
        };
        let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
            continue;
        };
        posts.push((date, date_str.to_string()));
    }

    posts.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(posts)
}

fn response_with_content_type(
    body: String,
    content_type: &'static str,
) -> Response {
    let mut resp = Response::new(Body::from(body));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    resp
}

fn newsletters_feed_response(
    state: &AppState,
    req: &RequestState,
    format: FeedFormat,
    feed_path: &str,
    content_type: &'static str,
    include_body_html: bool,
    force_http_urls: bool,
) -> Response {
    match newsletters_feed(
        format,
        feed_path,
        include_body_html,
        force_http_urls,
    ) {
        Ok(body) => response_with_content_type(body, content_type),
        Err(e) => error_400(state, req, e),
    }
}

fn get_title_for_date(date_str: &str) -> String {
    format!("Collective Toolbox Newsletter for {date_str}")
}

fn render_newsletter_post_index_hatom() -> Result<String> {
    let mut feed = Feed::empty("Collective Toolbox Newsletters")
        .with_home_page_url_opt(Some("/newsletters".to_string()));

    for (_date, date_str) in list_newsletter_posts()? {
        let url =
            format!("https://{}/newsletters/{date_str}", default_domain());

        // Render markdown to HTML, get an owned String, append "Read more",
        // then pass that String to the clip() function.
        let md_bytes = bail_if_none!(ctb_storage::get_asset(&format!(
            "views/pages/newsletters/{date_str}.md"
        )));
        // Content embedded as assets should be safe
        let page_html = ctb_formats_markdown::markdown2html_unsafe(md_bytes)?;
        let mut combined = String::from_utf8_lossy(&page_html).to_string();
        write!(combined, r#"<a href="{url}">Read more</a>"#)?;

        let clipped = clip(
            &combined,
            200,
            Some(&html_options(ClipHtmlOptions::default())),
        );

        let entry = Entry::new(
            url.clone(),
            get_title_for_date(&date_str),
            clipped,
            date_str,
        )
        .with_url_opt(Some(url))
        .with_author("Collective Toolbox Developers".to_string());
        feed.add_entry(entry);
    }

    feed.to_hatom()
        .context("Failed to render newsletter index as hAtom")
}

fn get_newsletters_rss(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::Rss,
        "/newsletters/newsletters.rss",
        "application/rss+xml; charset=utf-8",
        true,
        false,
    )
}

fn get_newsletters_atom(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::Atom,
        "/newsletters/newsletters.atom",
        "application/atom+xml; charset=utf-8",
        true,
        false,
    )
}

fn get_newsletters_json(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::JsonFeed,
        "/newsletters/newsletters.json",
        "application/feed+json; charset=utf-8",
        true,
        false,
    )
}

fn get_newsletters_rss_1(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::Rss1,
        "/newsletters/newsletters-rss-1.xml",
        "application/rss+xml; charset=utf-8",
        false,
        true,
    )
}

fn get_newsletters_rss_092(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::Rss092,
        "/newsletters/newsletters-rss-092.xml",
        "application/rss+xml; charset=utf-8",
        false,
        true,
    )
}

fn get_newsletters_rss_091ul(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::Rss091UserLand,
        "/newsletters/newsletters-rss-091ul.xml",
        "application/rss+xml; charset=utf-8",
        false,
        true,
    )
}

fn get_newsletters_rss_091ns(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::Rss091Netscape,
        "/newsletters/newsletters-rss-091ns.xml",
        "application/rss+xml; charset=utf-8",
        false,
        true,
    )
}

fn get_newsletters_rss_09(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::Rss09,
        "/newsletters/newsletters-rss-09.rdf",
        "application/rdf+xml; charset=utf-8",
        false,
        false,
    )
}

fn get_newsletters_scripting_news_02(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::ScriptingNews20,
        "/newsletters/newsletters-sn-02.xml",
        "application/xml; charset=utf-8",
        false,
        false,
    )
}

fn get_newsletters_scripting_news_01(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    newsletters_feed_response(
        &state,
        &req,
        FeedFormat::ScriptingNews10,
        "/newsletters/newsletters-sn-01.xml",
        "application/xml; charset=utf-8",
        false,
        false,
    )
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use crate::test_helpers::test_get_no_login;

    use super::*;

    #[crate::ctb_test]
    fn server_url_http_only_keeps_http() {
        assert_eq!(
            server_url_http_only("http://example.com"),
            "http://example.com"
        );
    }

    #[crate::ctb_test]
    fn server_url_http_only_converts_https_to_http() {
        assert_eq!(
            server_url_http_only("https://example.com"),
            "http://example.com"
        );
    }

    #[crate::ctb_test]
    fn server_url_http_only_falls_back_for_unknown_scheme() {
        let normalized = server_url_http_only("ftp://example.com");
        assert!(normalized.starts_with("http://"), "got: {normalized}");
    }

    #[crate::ctb_test]
    fn list_newsletter_posts_is_sorted_and_excludes_drafts() -> Result<()> {
        let posts = list_newsletter_posts()?;
        assert!(!posts.is_empty(), "Expected at least one newsletter post");

        for (_date, date_str) in &posts {
            assert_ne!(date_str, "2026-07-dd");
        }

        for window in posts.windows(2) {
            let Some(first) = window.first() else {
                continue;
            };
            let Some(second) = window.get(1) else {
                continue;
            };
            assert!(first.0 >= second.0, "Posts not sorted descending");
        }

        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn can_get_newsletter_pages_and_feeds() {
        let (status, body) = test_get_no_login("/newsletters").await;
        assert_eq!(status, 200);
        assert!(body.contains("Newsletter for 2026-06-08"), "{body}");
        assert!(body.contains("Collective Toolbox Newsletters"), "{body}");

        let (status, body) = test_get_no_login("/newsletters/2026-06-08").await;
        assert_eq!(status, 200);
        assert!(body.contains("Newsletter for 2026-06-08"), "{body}");
        assert!(body.contains("Welcome to the first ever edition"), "{body}");
        assert!(!body.contains("Collective Toolbox Newsletters"), "{body}");
        assert!(!body.contains("/newsletters/2026-06-08\n›"), "{body}");

        // FIXME: Don't have a test helper to easily check MIME type.
        let (status, body) =
            test_get_no_login("/newsletters/newsletters.rss").await;
        assert_eq!(status, 200);
        assert!(body.contains("rss version=\"2.0\""), "{body}");

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.atom").await;
        assert_eq!(status, 200);
        assert!(
            body.contains("feed xmlns=\"http://www.w3.org/2005/Atom\""),
            "{body}"
        );

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.json").await;
        assert_eq!(status, 200);
        assert!(
            body.contains("\"version\": \"https://jsonfeed.org/version/1.1\""),
            "{body}"
        );

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.rss-1.xml").await;
        assert_eq!(status, 200);
        assert!(
            body.contains("xmlns=\"http://purl.org/rss/1.0/\""),
            "{body}"
        );

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.rss-092.xml").await;
        assert_eq!(status, 200);
        assert!(body.contains("<rss version=\"0.92\">"), "{body}");

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.rss-091ul.xml").await;
        assert_eq!(status, 200);
        assert!(body.contains("<rss version=\"0.91\">"), "{body}");
        assert!(!body.contains("<!DOCTYPE rss SYSTEM"), "{body}");

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.rss-091ns.xml").await;
        assert_eq!(status, 200);
        assert!(body.contains("<rss version=\"0.91\">"), "{body}");
        assert!(body.contains("<!DOCTYPE rss SYSTEM"), "{body}");

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.rss-09.rdf").await;
        assert_eq!(status, 200);
        assert!(
            body.contains(
                "xmlns=\"http://channel.netscape.com/rdf/simple/0.9/\""
            ),
            "{body}"
        );

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.sn-02.xml").await;
        assert_eq!(status, 200);
        assert!(
            body.contains("<scriptingNewsVersion>2.0b1</scriptingNewsVersion>"),
            "{body}"
        );

        let (status, body) =
            test_get_no_login("/newsletters/newsletters.sn-01.xml").await;
        assert_eq!(status, 200);
        assert!(
            body.contains("<scriptingNewsVersion>1.0a2</scriptingNewsVersion>"),
            "{body}"
        );
    }
}
