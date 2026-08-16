//! Controller for general app UI pages.

use crate::flexible_form::FlexibleForm;
use crate::json::maybe_value::{
    MaybeOption, MaybeValue, bool_or_default, str_or_default, str_or_empty,
    u16_or_empty,
};
use crate::json::patch::{bool_to_patch, string_to_patch, u16_string_to_patch};
use axum::{extract::State, response::Response};
use axum_typed_multipart::TryFromMultipart;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::controllers::base::redirect_temporary;
use crate::error::{WebErr, WebError, WebResult};
use crate::extractors::request_state::RequestState;
use crate::utilities::build_info;
use crate::utilities::password::{Password, hash, verify};
use crate::utilities::*;
use crate::{AppState, error_400, error_403, respond_page};
use crate::{debug_fmt, json_value};
use ctb_formats_ipaddr::get_regex_ipv4_ipv6_exact;
use ctb_storage::user::UserPublicInfo;

/// The first-run setup and public PC settings page. For multi-user systems,
/// public servers, kiosks, etc. this should typically be locked down, but it
/// needs to be available for long enough after install for the administrator to
/// get the computer set up. FIXME: May not exactly be accurate? TBD.
pub async fn get_public_pc_settings(
    State(state): State<AppState>,
    req: RequestState,
) -> WebResult<Response> {
    #[derive(Serialize, Debug)]
    struct UserWithAdmin {
        admin: bool,
        local_id: u64,
        username: String,
        display_name: Option<Vec<u8>>,
        picture: Option<Vec<u8>>,
    }

    let build_info = build_info();
    // Reason for fallback: unreadable settings file defaults to default settings struct
    let current_settings = pc_settings::PcSettings::load().unwrap_or_default();
    let raw_settings = match pc_settings::PcSettings::load_raw_json() {
        Ok(v) => v,
        Err(_) => Value::Object(Map::new()),
    };
    let raw_obj = raw_settings.as_object();

    let is_default_key = |key: &str| match raw_obj.and_then(|o| o.get(key)) {
        None => true,
        Some(v) => v.is_null(),
    };

    let all_users = UserPublicInfo::list_all().web_err(&state, &req)?;

    debug_fmt!("All users: {all_users:#?}");

    let current_admin_users: Vec<u64> = match &current_settings.admin_users {
        MaybeOption::Value(v) => v.clone(),
        MaybeOption::Missing | MaybeOption::Null => Vec::new(),
    };

    let users: Vec<UserWithAdmin> = all_users
        .into_iter()
        .map(|u| UserWithAdmin {
            admin: current_admin_users.contains(&u.local_id()),
            local_id: u.local_id(),
            username: u.name().to_string(),
            display_name: u.display_name().map(<[u8]>::to_vec),
            picture: u.user_picture().map(<[u8]>::to_vec),
        })
        .collect();

    // debug_fmt!("Rendering PC settings page with users: {users:#?}");

    Ok(respond_page(
        &state,
        req,
        "settings.pc-settings",
        &json_value! ({
            "show_users" => bool_or_default(&current_settings.show_users, pc_settings::DEFAULT_SHOW_USERS),
            "show_users_default" => is_default_key("show_users"),
            "allow_local_account_creation" => bool_or_default(&current_settings.allow_local_account_creation, pc_settings::DEFAULT_ALLOW_LOCAL_ACCOUNT_CREATION),
            "allow_local_account_creation_default" => is_default_key("allow_local_account_creation"),
            "bind_to_ip" => str_or_default(&current_settings.bind_to_ip, pc_settings::DEFAULT_BIND_TO_IP),
            "bind_to_ip_default" => is_default_key("bind_to_ip"),
            "bind_to_ip_regex" => get_regex_ipv4_ipv6_exact(),
            "server_url" => str_or_default(&current_settings.server_url, pc_settings::DEFAULT_SERVER_URL),
            "server_url_default" => is_default_key("server_url"),
            "domain_name" => str_or_empty(&current_settings.domain_name),
            "domain_name_default" => is_default_key("domain_name"),
            // Reason for fallback: unconfigured fixed port option defaults string representation to empty
            "fixed_http_port" => u16_or_empty(&current_settings.fixed_http_port).map(|p| p.to_string()).unwrap_or_default(),
            "fixed_http_port_default" => is_default_key("fixed_http_port"),
            // Reason for fallback: unconfigured fixed port option defaults string representation to empty
            "fixed_https_port" => u16_or_empty(&current_settings.fixed_https_port).map(|p| p.to_string()).unwrap_or_default(),
            "fixed_https_port_default" => is_default_key("fixed_https_port"),
            "http_redirect" => bool_or_default(&current_settings.http_redirect, pc_settings::DEFAULT_HTTP_REDIRECT),
            "http_redirect_default" => is_default_key("http_redirect"),
            "redirect_www_to_non_www" => bool_or_default(&current_settings.redirect_www_to_non_www, pc_settings::DEFAULT_REDIRECT_WWW_TO_NON_WWW),
            "redirect_www_to_non_www_default" => is_default_key("redirect_www_to_non_www"),
            "tls_certificate" => str_or_empty(&current_settings.tls_certificate),
            "tls_certificate_default" => is_default_key("tls_certificate"),
            "tls_client_verification_cert" => {
                str_or_empty(&current_settings.tls_client_verification_cert)
            },
            "tls_client_verification_cert_default" => {
                is_default_key("tls_client_verification_cert")
            },
            "dev_signing_public_key" => str_or_empty(&current_settings.dev_signing_public_key),
            "dev_signing_public_key_default" => is_default_key("dev_signing_public_key"),
            "release_public_key" => str_or_empty(&current_settings.release_public_key),
            "release_public_key_default" => is_default_key("release_public_key"),
            "serve_public_web_site_only" => bool_or_default(&current_settings.serve_public_web_site_only, pc_settings::DEFAULT_SERVE_PUBLIC_WEB_SITE_ONLY),
            "serve_public_web_site_only_default" => is_default_key("serve_public_web_site_only"),
            "log_stack_file" => bool_or_default(&current_settings.log_stack_file, pc_settings::DEFAULT_LOG_STACK_FILE),
            "log_stack_file_default" => is_default_key("log_stack_file"),
            "access_log_mode_off" => current_settings.get_access_log_mode() == pc_settings::AccessLogMode::Off,
            "access_log_mode_errors" => current_settings.get_access_log_mode() == pc_settings::AccessLogMode::Errors,
            "access_log_mode_on" => current_settings.get_access_log_mode() == pc_settings::AccessLogMode::On,
            "access_log_mode_default" => is_default_key("access_log_mode"),

            // Feature flags
            "feature_login" => bool_or_default(&current_settings.feature_login, pc_settings::DEFAULT_FEATURE_LOGIN),
            "feature_registration" => bool_or_default(&current_settings.feature_registration, pc_settings::DEFAULT_FEATURE_REGISTRATION),
            "feature_login_default" => is_default_key("feature_login"),
            "feature_registration_default" => {
                is_default_key("feature_registration")
            },

            // Secrets not shown, just whether they are set
            "tls_private_key_set" => matches!(current_settings.tls_private_key, MaybeOption::Value(_)),
            "tls_private_key_default" => is_default_key("tls_private_key"),
            "dev_signing_private_key_set" => matches!(current_settings.dev_signing_private_key, MaybeOption::Value(_)),
            "dev_signing_private_key_default" => is_default_key("dev_signing_private_key"),
            "admin_password_set" => matches!(current_settings.admin_password_hash, MaybeOption::Value(_)),

            // All users, for building list
            "users" => users,

            // Provide build info for display
            "official_application_name" => crate::utilities::branding::official_application_name(),
            "crate_name" => build_info.name,
            "crate_version" => build_info.version,
            "build_date" => build_info.build_date,
            "commit" => build_info.commit
        }),
    ))
}

#[derive(TryFromMultipart, Serialize, Deserialize)]
#[expect(
    clippy::similar_names,
    reason = "settings form fields have similar names by design"
)]
#[try_from_multipart(strict)]
pub struct PcSettingsForm {
    show_users: Option<bool>,
    show_users_default: Option<bool>,
    allow_local_account_creation: Option<bool>,
    allow_local_account_creation_default: Option<bool>,
    bind_to_ip: Option<String>,
    bind_to_ip_default: Option<bool>,
    server_url: Option<String>,
    server_url_default: Option<bool>,
    domain_name: Option<String>,
    domain_name_default: Option<bool>,
    fixed_http_port: Option<String>,
    fixed_http_port_default: Option<bool>,
    fixed_https_port: Option<String>,
    fixed_https_port_default: Option<bool>,
    http_redirect: Option<bool>,
    http_redirect_default: Option<bool>,
    redirect_www_to_non_www: Option<bool>,
    redirect_www_to_non_www_default: Option<bool>,
    tls_certificate: Option<String>,
    tls_certificate_default: Option<bool>,
    tls_private_key: Option<String>,
    tls_private_key_default: Option<bool>,
    tls_client_verification_cert: Option<String>,
    tls_client_verification_cert_default: Option<bool>,
    dev_signing_public_key: Option<String>,
    dev_signing_public_key_default: Option<bool>,
    dev_signing_private_key: Option<String>,
    dev_signing_private_key_default: Option<bool>,
    release_public_key: Option<String>,
    release_public_key_default: Option<bool>,
    #[serde(default)]
    // I guess since it's a Vec, "empty" vs "omitted" can't really be distinguished with the way it gets serialized in the request
    admin_users: Vec<u64>,
    admin_password: Option<String>,
    serve_public_web_site_only: Option<bool>,
    serve_public_web_site_only_default: Option<bool>,
    log_stack_file: Option<bool>,
    log_stack_file_default: Option<bool>,
    feature_login: Option<bool>,
    feature_login_default: Option<bool>,
    feature_registration: Option<bool>,
    feature_registration_default: Option<bool>,
    access_log_mode: Option<String>,
    access_log_mode_default: Option<bool>,
}

#[expect(
    clippy::too_many_lines,
    clippy::similar_names,
    reason = "form processing is naturally long and references similar names"
)]
pub async fn post_public_pc_settings(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    req: RequestState,
    form: FlexibleForm<PcSettingsForm>,
) -> WebResult<Response> {
    let PcSettingsForm {
        show_users,
        show_users_default,
        allow_local_account_creation,
        allow_local_account_creation_default,
        bind_to_ip,
        bind_to_ip_default,
        server_url,
        server_url_default,
        domain_name,
        domain_name_default,
        fixed_http_port: fixed_http_port_str,
        fixed_http_port_default,
        fixed_https_port: fixed_https_port_str,
        fixed_https_port_default,
        http_redirect,
        http_redirect_default,
        redirect_www_to_non_www,
        redirect_www_to_non_www_default,
        tls_certificate,
        tls_certificate_default,
        tls_private_key,
        tls_private_key_default,
        tls_client_verification_cert,
        tls_client_verification_cert_default,
        dev_signing_public_key,
        dev_signing_public_key_default,
        dev_signing_private_key,
        dev_signing_private_key_default,
        release_public_key,
        release_public_key_default,
        admin_users,
        admin_password,
        serve_public_web_site_only,
        serve_public_web_site_only_default,
        log_stack_file,
        log_stack_file_default,
        feature_login,
        feature_login_default,
        feature_registration,
        feature_registration_default,
        access_log_mode,
        access_log_mode_default,
    } = form.0;

    let admin_password: Option<Password> = admin_password
        .map(|admin_password| Password::from_string(&admin_password));

    // Reason for fallback: unreadable settings file defaults to default settings struct
    let current_settings = pc_settings::PcSettings::load().unwrap_or_default();

    let current_admin_users: Vec<u64> = match &current_settings.admin_users {
        MaybeOption::Value(v) => v.clone(),
        MaybeOption::Missing | MaybeOption::Null => Vec::new(),
    };

    if !current_admin_users.is_empty() {
        let session_key_bytes =
            crate::session_auth::session_key_from_headers(&headers);
        let mut user_id = None;
        if let Some(key) = session_key_bytes {
            if let Some(user_mutex) =
                crate::session_auth::Session::get_user_by_key(
                    &mut state.clone(),
                    &key,
                )
                .await
            {
                user_id = Some(user_mutex.lock().await.local_id());
            }
        }
        let is_admin =
            user_id.is_some_and(|id| current_admin_users.contains(&id));
        if !is_admin {
            return Ok(error_403(
                &state,
                &req,
                "Must be logged in as an admin user to change settings"
                    .to_string(),
            ));
        }
    }

    let current_pass_hash = match &current_settings.admin_password_hash {
        MaybeOption::Value(s) => Some(s.as_str()),
        MaybeOption::Missing | MaybeOption::Null => None,
    };

    // If an admin password is set, require it
    if let Some(current_pass_hash) = current_pass_hash {
        let Some(provided) = admin_password.as_ref() else {
            return Ok(error_403(
                &state,
                &req,
                "Invalid admin password".to_string(),
            ));
        };

        let ok = verify(provided, current_pass_hash)
            .map_err(|e| WebError::new(e, state.clone(), req.clone()))?;

        if !ok {
            return Ok(error_403(
                &state,
                &req,
                "Invalid admin password".to_string(),
            ));
        }
    }

    // If no admin password is set and one is provided, hash it and save it.
    // If a password is already set, do not modify it here.
    let admin_password_hash =
        if current_pass_hash.is_none() {
            if let Some(provided) = admin_password {
                Some(hash(&provided).map_err(|e| {
                    WebError::new(e, state.clone(), req.clone())
                })?)
            } else {
                None
            }
        } else {
            None
        };

    // Reason for fallback: unselected checkbox form input defaults boolean state to false
    let checkbox_is_checked = |v: Option<bool>| v.unwrap_or(false);

    let show_users =
        bool_to_patch(checkbox_is_checked(show_users_default), show_users);

    let allow_local_account_creation = bool_to_patch(
        checkbox_is_checked(allow_local_account_creation_default),
        allow_local_account_creation,
    );

    let http_redirect = bool_to_patch(
        checkbox_is_checked(http_redirect_default),
        http_redirect,
    );

    let redirect_www_to_non_www = bool_to_patch(
        checkbox_is_checked(redirect_www_to_non_www_default),
        redirect_www_to_non_www,
    );

    let serve_public_web_site_only = bool_to_patch(
        checkbox_is_checked(serve_public_web_site_only_default),
        serve_public_web_site_only,
    );

    let log_stack_file = bool_to_patch(
        checkbox_is_checked(log_stack_file_default),
        log_stack_file,
    );

    let feature_login = bool_to_patch(
        checkbox_is_checked(feature_login_default),
        feature_login,
    );

    let feature_registration = bool_to_patch(
        checkbox_is_checked(feature_registration_default),
        feature_registration,
    );

    let access_log_mode = if checkbox_is_checked(access_log_mode_default) {
        MaybeValue::Missing
    } else {
        match access_log_mode.as_deref() {
            Some("off") => MaybeValue::Value(pc_settings::AccessLogMode::Off),
            Some("errors") => {
                MaybeValue::Value(pc_settings::AccessLogMode::Errors)
            }
            Some("on") => MaybeValue::Value(pc_settings::AccessLogMode::On),
            _ => MaybeValue::Missing,
        }
    };

    let bind_to_ip =
        string_to_patch(checkbox_is_checked(bind_to_ip_default), bind_to_ip);

    let target_bind_to_ip = match &bind_to_ip {
        MaybeValue::Value(s) => s.as_str(),
        MaybeValue::Missing => pc_settings::DEFAULT_BIND_TO_IP,
    };
    // Reason for fallback: invalid IP address string format evaluates bind address check as specified
    let is_target_zero =
        ctb_formats_ipaddr::is_unspecified(target_bind_to_ip).unwrap_or(false);
    if is_target_zero {
        let has_pass =
            current_pass_hash.is_some() || admin_password_hash.is_some();
        if !has_pass {
            return Ok(error_400(
                &state,
                &req,
                "Cannot bind to unspecified address (such as 0.0.0.0) without an admin password",
            ));
        }
    }

    let server_url =
        string_to_patch(checkbox_is_checked(server_url_default), server_url);
    let domain_name: Option<String> = if let Some(domain) = domain_name {
        pc_settings::normalize_ctb_domain_name(&domain).ok()
    } else {
        None
    };

    let domain_name =
        string_to_patch(checkbox_is_checked(domain_name_default), domain_name);

    let Ok(fixed_http_port_patch) = u16_string_to_patch(
        checkbox_is_checked(fixed_http_port_default),
        fixed_http_port_str,
    ) else {
        return Ok(error_400(&state, &req, "Invalid fixed HTTP port"));
    };

    let Ok(fixed_https_port_patch) = u16_string_to_patch(
        checkbox_is_checked(fixed_https_port_default),
        fixed_https_port_str,
    ) else {
        return Ok(error_400(&state, &req, "Invalid fixed HTTPS port"));
    };

    // TLS: empty textarea clears for certs, but does not clear private key.
    let tls_certificate = if checkbox_is_checked(tls_certificate_default) {
        MaybeOption::Missing
    } else if let Some(v) = tls_certificate {
        if v.trim().is_empty() {
            MaybeOption::Null
        } else {
            MaybeOption::Value(v)
        }
    } else {
        MaybeOption::Missing
    };

    let tls_private_key = if checkbox_is_checked(tls_private_key_default) {
        MaybeOption::Missing
    } else if let Some(v) = tls_private_key {
        if v.trim().is_empty() {
            MaybeOption::Missing
        } else {
            MaybeOption::Value(v)
        }
    } else {
        MaybeOption::Missing
    };

    let tls_client_verification_cert =
        if checkbox_is_checked(tls_client_verification_cert_default) {
            MaybeOption::Missing
        } else if let Some(v) = tls_client_verification_cert {
            if v.trim().is_empty() {
                MaybeOption::Null
            } else {
                MaybeOption::Value(v)
            }
        } else {
            MaybeOption::Missing
        };

    let dev_signing_public_key =
        if checkbox_is_checked(dev_signing_public_key_default) {
            MaybeOption::Missing
        } else if let Some(v) = dev_signing_public_key {
            if v.trim().is_empty() {
                MaybeOption::Null
            } else {
                MaybeOption::Value(v)
            }
        } else {
            MaybeOption::Missing
        };

    let dev_signing_private_key =
        if checkbox_is_checked(dev_signing_private_key_default) {
            MaybeOption::Missing
        } else if let Some(v) = dev_signing_private_key {
            if v.trim().is_empty() {
                MaybeOption::Missing
            } else {
                MaybeOption::Value(v)
            }
        } else {
            MaybeOption::Missing
        };

    let release_public_key = if checkbox_is_checked(release_public_key_default)
    {
        MaybeOption::Missing
    } else if let Some(v) = release_public_key {
        if v.trim().is_empty() {
            MaybeOption::Null
        } else {
            MaybeOption::Value(v)
        }
    } else {
        MaybeOption::Missing
    };

    let admin_users = if admin_users.is_empty() {
        let current_admin_users: Vec<u64> = match &current_settings.admin_users
        {
            MaybeOption::Value(v) => v.clone(),
            MaybeOption::Missing | MaybeOption::Null => Vec::new(),
        };
        if current_admin_users.is_empty() {
            MaybeOption::Missing
        } else {
            MaybeOption::Null
        }
    } else {
        MaybeOption::Value(admin_users)
    };

    let admin_password_hash = match admin_password_hash {
        Some(hash) => MaybeOption::Value(hash),
        None => MaybeOption::Missing,
    };

    let patch = pc_settings::PcSettings {
        show_users,
        bind_to_ip,
        server_url,
        domain_name,
        fixed_http_port: fixed_http_port_patch,
        fixed_https_port: fixed_https_port_patch,
        http_redirect,
        redirect_www_to_non_www,
        tls_certificate,
        tls_private_key,
        tls_client_verification_cert,
        dev_signing_public_key,
        dev_signing_private_key,
        release_public_key,
        admin_users,
        admin_password_hash,
        serve_public_web_site_only,
        log_stack_file,
        allow_local_account_creation,
        feature_login,
        feature_registration,
        access_log_mode,
        ..Default::default()
    };

    pc_settings::PcSettings::apply_patch(patch).web_err(&state, &req)?;
    Ok(redirect_temporary(req.is_js_request, "/pc-settings"))
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

    use crate::pc_settings::PcSettings;
    use crate::test_helpers::{
        assert_body_contains, assert_body_not_contains,
        assert_eq_or_print_body, test_get_no_login, test_post_no_login,
    };
    use ctb_storage::user::{UserPublicInfo, get_test_user};
    use ctb_utilities::json::maybe_value::{MaybeOption, MaybeValue};

    #[crate::ctb_test("tokio")]
    async fn can_get_pc_settings() {
        let test_user_1 =
            get_test_user(format!("{}_1", function_name!()).as_str());
        let test_user_2 =
            get_test_user(format!("{}_2", function_name!()).as_str());
        let test_user_3 =
            get_test_user(format!("{}_3", function_name!()).as_str());

        let old_settings = PcSettings::load().unwrap();

        // Set known settings
        let settings = PcSettings {
            show_users: MaybeValue::Value(true),
            bind_to_ip: MaybeValue::Value("123.456.789".to_string()),
            domain_name: MaybeOption::Value("foo.com".to_string()),
            fixed_http_port: MaybeOption::Value(1234),
            fixed_https_port: MaybeOption::Value(4567),
            admin_users: MaybeOption::Value(vec![
                test_user_1.local_id(),
                test_user_3.local_id(),
            ]),
            ..Default::default()
        };
        settings.save().unwrap();

        let (status, body) = test_get_no_login("/pc-settings").await;
        assert_eq_or_print_body(status, 200, &body);
        assert_body_contains(
            "These options apply to all users on this computer",
            &body,
        );
        // Check that the form contains the current values
        assert_body_contains("value=\"foo.com\"", &body);
        assert_body_contains("value=\"1234\"", &body);
        assert_body_contains(
            &format!("value=\"{}\"  checked", test_user_1.local_id()),
            &body,
        );
        assert_body_not_contains(
            &format!("value=\"{}\"  checked", test_user_2.local_id()),
            &body,
        );
        assert_body_contains(
            &format!("value=\"{}\"  checked", test_user_3.local_id()),
            &body,
        );
        // assert_body_contains("checked", &body); // show_users checked

        old_settings.save().unwrap();
    }

    #[crate::ctb_test("tokio")]
    async fn can_save_settings_and_read_back() {
        struct SettingsRestoreGuard(PcSettings);
        impl Drop for SettingsRestoreGuard {
            fn drop(&mut self) {
                self.0.save().unwrap();
            }
        }

        let old_settings = PcSettings::load().unwrap();
        let _guard = SettingsRestoreGuard(old_settings.clone());

        // Ensure no admin users are set first
        let clear_settings = PcSettings {
            admin_users: MaybeOption::Missing,
            ..old_settings.clone()
        };
        clear_settings.save().unwrap();

        // Prepare form data
        #[derive(serde::Serialize)]
        struct Form {
            show_users: bool,
            bind_to_ip: Option<String>,
            domain_name: Option<String>,
            fixed_http_port: Option<String>,
            fixed_https_port: Option<String>,
            admin_users: Vec<u64>,
            feature_login: bool,
            access_log_mode: Option<String>,
        }
        let form = Form {
            show_users: false,
            bind_to_ip: Some("127.0.0.1".to_string()),
            domain_name: Some("example.com".to_string()),
            fixed_http_port: Some("5678".to_string()),
            fixed_https_port: Some("5679".to_string()),
            admin_users: vec![18, 19],
            feature_login: true,
            access_log_mode: Some("on".to_string()),
        };

        let (status, body) =
            test_post_no_login("/pc-settings", None, Some(&form), None).await;
        assert_eq_or_print_body(status, 303, &body); // Should redirect

        // Now load settings and check values
        let loaded = PcSettings::load().unwrap();
        assert_eq!(loaded.show_users, MaybeValue::Value(false));
        assert_eq!(
            loaded.bind_to_ip,
            MaybeValue::Value("127.0.0.1".to_string())
        );
        assert_eq!(
            loaded.domain_name,
            MaybeOption::Value("example.com".to_string())
        );
        assert_eq!(loaded.fixed_http_port, MaybeOption::Value(5678));
        assert_eq!(loaded.fixed_https_port, MaybeOption::Value(5679));
        assert_eq!(loaded.admin_users, MaybeOption::Value(vec![18, 19]));
        assert_eq!(loaded.feature_login, MaybeValue::Value(true));
        assert_eq!(
            loaded.access_log_mode,
            MaybeValue::Value(crate::pc_settings::AccessLogMode::On)
        );

        // Clear admin_users before the second POST so it doesn't fail due to lack of login
        let mut reset_settings = PcSettings::load().unwrap();
        reset_settings.admin_users = MaybeOption::Missing;
        reset_settings.save().unwrap();

        // Test same values with multipart upload (just make sure it returns 303)
        let (status, body) =
            test_post_no_login("/pc-settings", None, None, Some(&form)).await;
        assert_eq_or_print_body(status, 303, &body); // Should redirect
    }

    #[crate::ctb_test("tokio")]
    async fn pc_settings_roundtrip_does_not_write_bogus_nulls() {
        struct SettingsRestoreGuard(PcSettings);
        impl Drop for SettingsRestoreGuard {
            fn drop(&mut self) {
                self.0.save().unwrap();
            }
        }

        let old_settings = PcSettings::load().unwrap();
        let _guard = SettingsRestoreGuard(old_settings.clone());

        // Ensure no admin users are set first
        let clear_settings = PcSettings {
            admin_users: MaybeOption::Missing,
            ..old_settings.clone()
        };
        clear_settings.save().unwrap();

        // Simulate a browser submission where many "use default" checkboxes
        // are checked because the keys are absent in the raw JSON.
        #[derive(serde::Serialize)]
        #[expect(
            clippy::struct_excessive_bools,
            reason = "test form data mock structure"
        )]
        struct Form {
            // Intentionally set
            tls_certificate: Option<String>,
            admin_password: Option<String>,
            feature_registration: bool,

            // Defaults checked (historically caused JSON `null` entries)
            domain_name_default: bool,
            fixed_http_port_default: bool,
            fixed_https_port_default: bool,
            tls_private_key_default: bool,
            tls_client_verification_cert_default: bool,

            // This is not editable right now (IPC disabled), but the form type
            // includes it and it used to get persisted as `[]`.
            admin_users: Vec<u64>,
        }

        let form = Form {
            tls_certificate: Some("test".to_string()),
            admin_password: Some("password123".to_string()),
            feature_registration: false,

            domain_name_default: true,
            fixed_http_port_default: true,
            fixed_https_port_default: true,
            tls_private_key_default: true,
            tls_client_verification_cert_default: true,

            admin_users: Vec::new(),
        };

        let (status, body) =
            test_post_no_login("/pc-settings", None, Some(&form), None).await;
        assert_eq_or_print_body(status, 303, &body);

        let raw = PcSettings::load_raw_json().unwrap();
        let obj = raw.as_object().unwrap();

        // Should not create keys for defaulted/untouched fields.
        assert!(!obj.contains_key("domain_name"));
        assert!(!obj.contains_key("fixed_http_port"));
        assert!(!obj.contains_key("fixed_https_port"));
        assert!(!obj.contains_key("tls_private_key"));
        assert!(!obj.contains_key("tls_client_verification_cert"));
        assert!(!obj.contains_key("admin_users"));

        // Should persist the intentionally set fields.
        assert_eq!(
            obj.get("tls_certificate"),
            Some(&serde_json::Value::String("test".to_string()))
        );
        assert_eq!(
            obj.get("feature_registration"),
            Some(&serde_json::Value::Bool(false))
        );
        assert!(
            obj.get("admin_password_hash")
                .and_then(|v| v.as_str())
                .is_some()
        );

        // Test same values with multipart upload (just make sure it returns 303)
        let (status, body) =
            test_post_no_login("/pc-settings", None, None, Some(&form)).await;
        assert_eq_or_print_body(status, 303, &body);
    }

    #[crate::ctb_test("tokio")]
    async fn post_refuses_saving_zero_ip_without_password() {
        let old_settings = PcSettings::load().unwrap();

        // Clear password first
        let clear_settings = PcSettings {
            admin_password_hash: MaybeOption::Null,
            ..old_settings.clone()
        };
        clear_settings.save().unwrap();

        #[derive(serde::Serialize)]
        struct Form {
            bind_to_ip: Option<String>,
            admin_password: Option<String>,
        }

        let form = Form {
            bind_to_ip: Some("0.0.0.0".to_string()),
            admin_password: None,
        };

        // Submit form. Since we have bind_to_ip = "0.0.0.0" and no password, it should fail with 400.
        let (status, body) =
            test_post_no_login("/pc-settings", None, Some(&form), None).await;
        assert_eq_or_print_body(status, 400, &body);
        assert_body_contains(
            "Cannot bind to unspecified address (such as 0.0.0.0) without an admin password",
            &body,
        );

        old_settings.save().unwrap();
    }

    #[crate::ctb_test("tokio")]
    async fn admin_users_restrictions() {
        use crate::test_helpers::TestApp;
        use http::Method;

        struct SettingsRestoreGuard(PcSettings);
        impl Drop for SettingsRestoreGuard {
            fn drop(&mut self) {
                self.0.save().unwrap();
            }
        }

        let old_settings = PcSettings::load().unwrap();
        let _guard = SettingsRestoreGuard(old_settings.clone());

        // Ensure no admin users are set first
        let clear_settings = PcSettings {
            admin_users: MaybeOption::Null,
            ..old_settings.clone()
        };
        clear_settings.save().unwrap();

        let test_app = TestApp::new();

        // 1. Register and login admin_user first to get their actual local ID
        let admin_username = format!("{}_admin", function_name!());
        let (admin_cookie, _admin_lock) =
            test_app.register_and_login(&admin_username).await.unwrap();

        // Look up the logged-in user's ID
        let admin_user_info = UserPublicInfo::get_by_name(&admin_username)
            .unwrap()
            .unwrap();
        let admin_user_id = admin_user_info.local_id();

        // Register and login non_admin_user
        let non_admin_username = format!("{}_non_admin", function_name!());
        let (non_admin_cookie, _non_admin_lock) = test_app
            .register_and_login(&non_admin_username)
            .await
            .unwrap();

        // Prepare a form to submit
        #[derive(serde::Serialize)]
        struct Form {
            show_users: bool,
            admin_users: Vec<u64>,
        }
        let form_to_submit = Form {
            show_users: true,
            admin_users: vec![admin_user_id],
        };

        // Without admin users set, posting without login should succeed (redirects to /pc-settings)
        let (status, body) = test_post_no_login(
            "/pc-settings",
            None,
            Some(&form_to_submit),
            None,
        )
        .await;
        assert_eq_or_print_body(status, 303, &body);

        // Now admin_user is in settings.admin_users
        let loaded = PcSettings::load().unwrap();
        assert_eq!(loaded.admin_users, MaybeOption::Value(vec![admin_user_id]));

        // With admin users set:
        // A. POST without login should fail with 403
        let (status, body) = test_post_no_login(
            "/pc-settings",
            None,
            Some(&form_to_submit),
            None,
        )
        .await;
        assert_eq_or_print_body(status, 403, &body);
        assert_body_contains(
            "Must be logged in as an admin user to change settings",
            &body,
        );

        // B. POST with non_admin_user logged in should fail with 403
        let (status, body) = test_app
            .request(
                Method::POST,
                "/pc-settings",
                None,
                Some(&non_admin_cookie),
                None,
                Some(&form_to_submit),
            )
            .await;
        assert_eq_or_print_body(status, 403, &body);
        assert_body_contains(
            "Must be logged in as an admin user to change settings",
            &body,
        );

        // C. POST with admin_user logged in should succeed with 303
        let form_to_clear = Form {
            show_users: true,
            admin_users: Vec::new(),
        };

        let (status, body) = test_app
            .request(
                Method::POST,
                "/pc-settings",
                None,
                Some(&admin_cookie),
                None,
                Some(&form_to_clear),
            )
            .await;
        assert_eq_or_print_body(status, 303, &body);

        // Check that settings are updated and admin_users is cleared/missing
        let loaded = PcSettings::load().unwrap();
        assert!(matches!(
            loaded.admin_users,
            MaybeOption::Missing | MaybeOption::Null
        ));
    }
}
