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

use axum::debug_handler;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use axum_typed_multipart::TryFromMultipart;
use maplit::btreemap;
use serde::{Deserialize, Serialize};

use crate::controllers::base::redirect_temporary;
use crate::flexible_form::FlexibleForm;
use crate::session_auth::AuthenticatedUser;
use crate::session_auth::Session;
use crate::utilities::feature;
use crate::utilities::password::Password;
use crate::utilities::*;
use crate::{
    AppState, RequestState, error_404, recoverable_error, respond_dialog,
};
use crate::{debug, error, info, json_value};
use axum::http::StatusCode;
use ctb_storage::user::{User, UserPublicInfo};
use pc_settings::{PcSettingStrKey, get_bool_setting, get_str_setting};

fn login_disabled(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let serve_public_web_site_only =
        get_bool_setting(pc_settings::PcSettingBoolKey::ServePublicWebSiteOnly);
    respond_dialog(
        &state,
        req,
        "login_disabled",
        &btreemap! { "serve_public_web_site_only".to_string() => serve_public_web_site_only },
    )
}

fn registration_disabled(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    let serve_public_web_site_only =
        get_bool_setting(pc_settings::PcSettingBoolKey::ServePublicWebSiteOnly);
    respond_dialog(
        &state,
        req,
        "registration_disabled",
        &btreemap! { "serve_public_web_site_only".to_string() => serve_public_web_site_only },
    )
}

pub async fn get_login(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    if !feature("login") {
        return login_disabled(State(state), req);
    }

    respond_dialog(&state, req, "login", &json_value!({}))
}

#[derive(TryFromMultipart, Serialize, Deserialize)]
#[try_from_multipart(strict)]
pub struct LoginForm {
    username: String,
}

pub async fn post_login(
    State(state): State<AppState>,
    req: RequestState,
    FlexibleForm(input): FlexibleForm<LoginForm>,
) -> Response {
    if !feature("login") {
        return login_disabled(State(state), req);
    }

    let username = input.username;

    let local_exists = local_account_exists(&username);
    let remote_exists = remote_account_exists(&username);

    if !local_exists && !remote_exists {
        // Registration view with suggested username
        return respond_dialog(
            &state,
            req,
            "register",
            &btreemap! { "username".to_string() => username.clone() },
        );
    } else if !local_exists && remote_exists {
        // Log in with remote account only: auto-create local account and do initial sync
        return recoverable_error(
            &state,
            req,
            "Logging in with remote-only account is not yet implemented.",
        );
    }

    // Either both exist, or only local exists. No problem, continue
    respond_dialog(
        &state,
        req,
        "login_password",
        &btreemap! { "username".to_string() => username.clone() },
    )
}

#[derive(TryFromMultipart, Serialize, Deserialize)]
#[try_from_multipart(strict)]
pub struct LoginPasswordForm {
    username: String,
    password: String, // Hidden, just kept to maintain state
}
pub async fn post_login_password(
    State(state): State<AppState>,
    req: RequestState,
    jar: CookieJar,
    FlexibleForm(input): FlexibleForm<LoginPasswordForm>,
) -> impl IntoResponse {
    if !feature("login") {
        return (jar, login_disabled(State(state), req));
    }

    let username = input.username;
    let password = Password {
        password: input.password.as_bytes().to_vec(),
    };
    debug!(format!(
        "post_login_password: username={}",
        username.clone()
    ));
    let user_public_info = UserPublicInfo::get_by_name(&username);
    let Ok(Some(user_public_info)) = user_public_info else {
        return (
            jar,
            error_404(
                &state,
                &req,
                "The account seems to have disappeared or been removed? This may indicate a bug",
            ),
        );
    };

    // FIXME: Avoid cloning password string if possible.
    let session_token = match ctb_storage::user::login_user(
        &username,
        password.password.clone(),
        3600,
    ) {
        Ok(token) => token,
        Err(e) => {
            error!(format!("Failed to login user: {e}"));
            return (
                jar,
                recoverable_error(
                    &state,
                    req,
                    "The password is most likely incorrect, or perhaps there was an error looking up the user.",
                ),
            );
        }
    };

    let user_id = user_public_info.local_id();
    let remote_status = user_public_info.remote_status().to_string();

    let logged_in =
        User::from_public_info(user_public_info, Some(session_token.clone()));

    // Log in was ok; return redirect home with session cookie set
    let session =
        Session::new(&mut state.clone(), logged_in, &session_token).await;

    if remote_status == "Pending" {
        let server_url = get_str_setting(PcSettingStrKey::ServerUrl);
        if let Some(server_url) = server_url.as_deref() {
            let server_url = server_url.to_string();
            let test_name =
                crate::utilities::testing::try_get_current_test_name();
            let test_storage_dir =
                crate::utilities::testing::try_get_test_storage_dir();

            tokio::spawn(async move {
                let fut = async move {
                    match ctb_storage::models::sync::register_on_server(user_id, &server_url).await {
                        Ok(ctb_storage::models::sync::RemoteRegisterResult::Success) => {
                            let mut user_dto = ctb_storage::models::user_impl::UserDto {
                                id: user_id,
                                username: String::new(),
                                uuid: vec![],
                                auth: None,
                                display_name: None,
                                picture: None,
                                key_encryption_key_params: None,
                                wrapped_dek: None,
                                pubkey: None,
                                subscription_expiry: None,
                                token_quota: None,
                                remote_status: Some("Registered".to_string()),
                            };
                            if let Ok(Some(existing_dto)) = ipcb!(storage).get_user_by_id_b(user_id) {
                                user_dto.username = existing_dto.username;
                                user_dto.uuid = existing_dto.uuid;
                                user_dto.auth = existing_dto.auth;
                                user_dto.display_name = existing_dto.display_name;
                                user_dto.picture = existing_dto.picture;
                                user_dto.key_encryption_key_params = existing_dto.key_encryption_key_params;
                                user_dto.wrapped_dek = existing_dto.wrapped_dek;
                                user_dto.pubkey = existing_dto.pubkey;
                                user_dto.subscription_expiry = existing_dto.subscription_expiry;
                                user_dto.token_quota = existing_dto.token_quota;
                            }
                            let _ = ipcb!(storage).update_user_b(user_dto.into());
                        }
                        Ok(ctb_storage::models::sync::RemoteRegisterResult::Conflict) => {
                            let mut user_dto = ctb_storage::models::user_impl::UserDto {
                                id: user_id,
                                username: String::new(),
                                uuid: vec![],
                                auth: None,
                                display_name: None,
                                picture: None,
                                key_encryption_key_params: None,
                                wrapped_dek: None,
                                pubkey: None,
                                subscription_expiry: None,
                                token_quota: None,
                                remote_status: Some("Conflict".to_string()),
                            };
                            if let Ok(Some(existing_dto)) = ipcb!(storage).get_user_by_id_b(user_id) {
                                user_dto.username = existing_dto.username;
                                user_dto.uuid = existing_dto.uuid;
                                user_dto.auth = existing_dto.auth;
                                user_dto.display_name = existing_dto.display_name;
                                user_dto.picture = existing_dto.picture;
                                user_dto.key_encryption_key_params = existing_dto.key_encryption_key_params;
                                user_dto.wrapped_dek = existing_dto.wrapped_dek;
                                user_dto.pubkey = existing_dto.pubkey;
                                user_dto.subscription_expiry = existing_dto.subscription_expiry;
                                user_dto.token_quota = existing_dto.token_quota;
                            }
                            let _ = ipcb!(storage).update_user_b(user_dto.into());
                        }
                        _ => {}
                    }
                };

                let scope_fut = async move {
                    if let Some(dir) = test_storage_dir {
                        crate::utilities::testing::TEST_STORAGE_DIR
                            .scope(dir, fut)
                            .await;
                    } else {
                        fut.await;
                    }
                };

                if let Some(name) = test_name {
                    let _guard =
                        crate::utilities::testing::push_current_test_name(
                            Some(name),
                        );
                    scope_fut.await;
                } else {
                    scope_fut.await;
                }
            });
        }
    }

    let mut cookie = Cookie::new("session", session.id());
    cookie.set_path("/");
    let updated_jar = jar.add(cookie);

    (updated_jar, redirect_temporary(req.is_js_request, "/home"))
}

#[derive(TryFromMultipart, Serialize, Deserialize)]
#[try_from_multipart(strict)]
pub struct RegistrationForm {
    username: String,
    password: String,
    password_confirm: String,
}
/// Handles registration form submission.
/// Logs detailed errors if multipart parsing fails or if any step fails.
#[debug_handler]
pub async fn post_registration(
    State(state): State<AppState>,
    req: RequestState,
    jar: CookieJar,
    FlexibleForm(input): FlexibleForm<RegistrationForm>,
) -> impl IntoResponse {
    if !feature("registration") {
        return registration_disabled(State(state), req);
    }

    debug!("post_registration: received registration request");
    info!("post_registration: received registration request");

    let username = input.username.clone();
    let password = Password {
        password: input.password.as_bytes().to_vec(),
    };
    let password_confirm = Password {
        password: input.password_confirm.as_bytes().to_vec(),
    };

    if password != password_confirm {
        error!(
            "post_registration: passwords did not match for user '{}'",
            username
        );
        return recoverable_error(
            &state,
            req,
            "Passwords did not match".to_string(),
        );
    }

    error!("post_registration: creating user '{}'", &username);
    // FIXME: Avoid cloning password string if possible.
    let session_token = match ctb_storage::user::create_user_and_session(
        &username,
        password.password.clone(),
        3600,
    ) {
        Ok(token) => token,
        Err(e) => {
            error!(
                "post_registration: failed to create user '{}': {:?}",
                username, &e
            );
            return recoverable_error(
                &state,
                req,
                format!("Failed to create user: {e:?}"),
            );
        }
    };

    let Ok(Some(user_info)) = UserPublicInfo::get_by_name(&username) else {
        error!(
            "post_registration: failed to get user info for '{}'",
            &username
        );
        return recoverable_error(
            &state,
            req,
            format!("Failed to get user info for '{username}'"),
        )
        .into_response();
    };
    if user_info.name() != username {
        error!(
            "post_registration: user info name mismatch: '{}' != '{}'",
            user_info.name(),
            &username
        );
        return recoverable_error(
            &state,
            req,
            format!(
                "User info name mismatch: '{}' != '{}'",
                user_info.name(),
                &username
            ),
        )
        .into_response();
    }
    info!("post_registration: created user '{}'", &username);

    let mut final_status = "Pending".to_string();
    let server_url = get_str_setting(PcSettingStrKey::ServerUrl);
    if let Some(server_url) = server_url.as_deref() {
        match ctb_storage::models::sync::register_on_server(
            user_info.local_id(),
            server_url,
        )
        .await
        {
            Ok(ctb_storage::models::sync::RemoteRegisterResult::Success) => {
                final_status = "Registered".to_string();
            }
            Ok(ctb_storage::models::sync::RemoteRegisterResult::Conflict) => {
                ctb_storage::user::User::delete_by_name(&username).ok();
                error!(format!(
                    "post_registration: username '{}' already taken on server",
                    username
                ));
                return recoverable_error(
                    &state,
                    req,
                    format!("The username '{username}' is already taken on the remote server. Please choose a different username."),
                ).into_response();
            }
            _ => {
                // Keep as Pending
            }
        }
    }

    if final_status == "Registered" {
        let mut user_dto = ctb_storage::models::user_impl::UserDto {
            id: user_info.local_id(),
            username: user_info.name().to_string(),
            uuid: user_info.uuid().clone(),
            auth: None,
            display_name: user_info.display_name().map(<[u8]>::to_vec),
            picture: user_info.user_picture().map(<[u8]>::to_vec),
            key_encryption_key_params: None,
            wrapped_dek: None,
            pubkey: None,
            subscription_expiry: None,
            token_quota: None,
            remote_status: Some("Registered".to_string()),
        };
        if let Ok(Some(existing_dto)) =
            ipcb!(storage).get_user_by_id_b(user_info.local_id())
        {
            user_dto.auth = existing_dto.auth;
            user_dto.key_encryption_key_params =
                existing_dto.key_encryption_key_params;
            user_dto.wrapped_dek = existing_dto.wrapped_dek;
            user_dto.pubkey = existing_dto.pubkey;
            user_dto.subscription_expiry = existing_dto.subscription_expiry;
            user_dto.token_quota = existing_dto.token_quota;
        }
        let _ = ipcb!(storage).update_user_b(user_dto.into());
    }

    let Ok(Some(updated_user_info)) = UserPublicInfo::get_by_name(&username)
    else {
        return recoverable_error(
            &state,
            req,
            format!("Failed to retrieve updated user info for '{username}'"),
        )
        .into_response();
    };

    let logged_in =
        User::from_public_info(updated_user_info, Some(session_token.clone()));
    let session =
        Session::new(&mut state.clone(), logged_in, &session_token).await;
    let mut cookie = Cookie::new("session", session.id());
    cookie.set_path("/");
    let updated_jar = jar.add(cookie);

    (updated_jar, redirect_temporary(req.is_js_request, "/home"))
        .into_response()
}

pub async fn get_logout(
    State(mut state): State<AppState>,
    req: RequestState,
    jar: CookieJar,
) -> impl IntoResponse {
    let mut updated_jar = jar.clone();
    if let Some(cookie) = jar.get("session") {
        let session_val = cookie.value();
        use base64::Engine;
        if let Ok(key) =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(session_val)
        {
            Session::invalidate(&mut state, &key).await;
        }
        let mut remove_cookie = Cookie::new("session", "");
        remove_cookie.set_path("/");
        updated_jar = updated_jar.remove(remove_cookie);
    }
    (updated_jar, redirect_temporary(req.is_js_request, "/"))
}

#[debug_handler]
pub async fn post_api_user_register(
    State(_state): State<AppState>,
    axum::Json(payload): axum::Json<
        ctb_storage::models::sync::ApiRegisterRequest,
    >,
) -> impl IntoResponse {
    use crate::utilities::environment::is_public_website;

    if !is_public_website() {
        return (
            StatusCode::FORBIDDEN,
            "Registration API only available on public server website",
        )
            .into_response();
    }

    let username = payload.username.clone();
    let exists = ctb_storage::user::user_exists(&username);
    if exists {
        return (
            StatusCode::CONFLICT,
            axum::Json(serde_json::json!({ "error": "username_taken" })),
        )
            .into_response();
    }

    let server_user_id =
        match ctb_storage::user::User::increment_and_get_user_id() {
            Ok(id) => id,
            Err(e) => {
                error!(format!(
                    "Failed to increment and get user ID on server: {e}"
                ));
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Server database error",
                )
                    .into_response();
            }
        };

    let user_dto = ctb_storage::models::user_impl::UserDto {
        id: server_user_id,
        username,
        uuid: payload.uuid,
        auth: payload.auth,
        display_name: payload.display_name,
        picture: None,
        key_encryption_key_params: payload.key_encryption_key_params,
        wrapped_dek: payload.wrapped_dek,
        pubkey: None,
        subscription_expiry: None,
        token_quota: None,
        remote_status: Some("Registered".to_string()),
    };

    if let Err(e) = ipcb!(storage).create_user_b(user_dto.into()) {
        error!(format!("Failed to create user on server: {e}"));
        return (StatusCode::INTERNAL_SERVER_ERROR, "Server database error")
            .into_response();
    }

    (
        StatusCode::CREATED,
        axum::Json(serde_json::json!({ "success": true })),
    )
        .into_response()
}

pub async fn get_rename(
    State(state): State<AppState>,
    req: RequestState,
    user: AuthenticatedUser,
) -> Response {
    let u = user.user.lock().await;
    respond_dialog(
        &state,
        req,
        "rename",
        &json_value!({
            "username" => u.name(),
        }),
    )
}

#[derive(TryFromMultipart, Serialize, Deserialize)]
#[try_from_multipart(strict)]
pub struct RenameForm {
    new_username: String,
}

pub async fn post_rename(
    State(state): State<AppState>,
    req: RequestState,
    user: AuthenticatedUser,
    FlexibleForm(input): FlexibleForm<RenameForm>,
) -> Response {
    let u: tokio::sync::MutexGuard<'_, User> = user.user.lock().await;
    let user_id = u.local_id();
    let old_username = u.name();
    let new_username = input.new_username.clone();

    if new_username == old_username {
        return recoverable_error(
            &state,
            req,
            "The new username must be different from your current username."
                .to_string(),
        );
    }

    match ipcb!(storage).rename_user_b(user_id, &new_username) {
        Ok(()) => {}
        Err(e) => {
            error!(format!("Failed to rename user locally: {e}"));
            return recoverable_error(
                &state,
                req,
                format!("Failed to rename user: {e}"),
            );
        }
    }

    {
        let mut users_cache = state.users.lock().await;
        users_cache.remove(&user_id);
    }

    let server_url_opt = get_str_setting(PcSettingStrKey::ServerUrl);
    let mut final_status = "Pending".to_string();
    if let Some(url) = server_url_opt.as_deref() {
        match ctb_storage::models::sync::register_on_server(user_id, url).await
        {
            Ok(ctb_storage::models::sync::RemoteRegisterResult::Success) => {
                final_status = "Registered".to_string();
            }
            Ok(ctb_storage::models::sync::RemoteRegisterResult::Conflict) => {
                final_status = "Conflict".to_string();
            }
            _ => {
                // Keep as Pending
            }
        }
    }

    let mut user_dto = ctb_storage::models::user_impl::UserDto {
        id: user_id,
        username: String::new(),
        uuid: vec![],
        auth: None,
        display_name: None,
        picture: None,
        key_encryption_key_params: None,
        wrapped_dek: None,
        pubkey: None,
        subscription_expiry: None,
        token_quota: None,
        remote_status: Some(final_status.clone()),
    };
    if let Ok(Some(existing_dto)) = ipcb!(storage).get_user_by_id_b(user_id) {
        user_dto.username = existing_dto.username;
        user_dto.uuid = existing_dto.uuid;
        user_dto.auth = existing_dto.auth;
        user_dto.display_name = existing_dto.display_name;
        user_dto.picture = existing_dto.picture;
        user_dto.key_encryption_key_params =
            existing_dto.key_encryption_key_params;
        user_dto.wrapped_dek = existing_dto.wrapped_dek;
        user_dto.pubkey = existing_dto.pubkey;
        user_dto.subscription_expiry = existing_dto.subscription_expiry;
        user_dto.token_quota = existing_dto.token_quota;
    }
    let _ = ipcb!(storage).update_user_b(user_dto.into());

    if final_status == "Conflict" {
        return recoverable_error(
            &state,
            req,
            format!(
                "The username '{new_username}' is already taken on the remote server. Please choose a different username."
            ),
        );
    }

    redirect_temporary(req.is_js_request, "/home")
}

// ================ Auth and user helpers (stubs) ================

fn local_account_exists(username: &str) -> bool {
    if let Ok(Some(_)) = UserPublicInfo::get_by_name(username) {
        return true;
    }
    false
}

fn remote_account_password_matches(
    _username: &String,
    _password: &String,
) -> bool {
    true
}

fn remote_account_exists(_username: &String) -> bool {
    false
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
mod auth_controller_tests {
    use super::*;
    use crate::test_helpers::{
        assert_eq_or_print_body, assert_or_print_body,
        assert_successful_and_return_resp, test_app, test_get_no_login,
        test_post_no_login, test_request, test_request_get_response,
    };
    use ::http::Method;
    use axum::http::StatusCode;
    use ctb_storage::user::{get_test_user, lock_by_name};

    // In the interest of actually testing this flow, these few tests should NOT
    // use the default test password (which bypasses the hash check for
    // performance)

    const THIS_TEST_USER_PASS: &str = "test_password_auth_controller";

    #[ctb_test("tokio")]
    async fn test_get_login_route() {
        let (status, body) = test_get_no_login("/login").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("login"));
    }

    #[ctb_test("tokio")]
    async fn test_registration_and_login_flow() -> Result<()> {
        let name = function_name!();
        let _ = lock_by_name(name)?;
        User::delete_by_name(name).ok();
        let (_state, app) = test_app();
        #[derive(serde::Serialize)]
        struct RegistrationForm<'a> {
            username: &'a str,
            password: &'a str,
            password_confirm: &'a str,
        }
        let reg_form = RegistrationForm {
            username: name,
            password: THIS_TEST_USER_PASS,
            password_confirm: THIS_TEST_USER_PASS,
        };
        let resp = test_request_get_response(
            &app,
            axum::http::Method::POST,
            "/registration",
            None,
            None,
            None,
            None,
            Some(&reg_form),
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        let cookie = resp.headers().get("Set-Cookie");
        assert!(cookie.is_some(), "No Set-Cookie header found");
        let user_info = UserPublicInfo::get_by_name(name)?
            .expect("Failed to get user info");
        assert!(user_info.name() == name);

        #[derive(serde::Serialize)]
        struct LoginForm<'a> {
            username: &'a str,
        }
        let login_form = LoginForm { username: name };
        let (status, body) =
            test_post_no_login("/login", None, None, Some(&login_form)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("login-password"));

        #[derive(serde::Serialize)]
        struct LoginPasswordForm<'a> {
            username: &'a str,
            password: &'a str,
        }
        let login_pw_form = LoginPasswordForm {
            username: name,
            password: THIS_TEST_USER_PASS,
        };
        let resp = test_request_get_response(
            &app,
            axum::http::Method::POST,
            "/login-password",
            None,
            None,
            None,
            None,
            Some(&login_pw_form),
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        let cookie = resp.headers().get("Set-Cookie");
        assert!(cookie.is_some(), "No Set-Cookie header found");
        let cookie = cookie
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("Could not get cookie"));
        assert!(cookie.is_ok(), "Could not get cookie");
        let cookie = cookie.expect("checked").to_string();

        let (status, body) = test_request::<()>(
            &app,
            Method::GET,
            "/search",
            None,
            Some(&cookie),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert_or_print_body(body.contains("name=\"search-text\""), &body);

        // Test logging out
        let resp = test_request_get_response::<()>(
            &app,
            Method::GET,
            "/logout",
            None,
            Some(&cookie),
            None,
            None,
            None,
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);

        let logout_cookie = resp.headers().get("Set-Cookie");
        assert!(
            logout_cookie.is_some(),
            "No Set-Cookie header found on logout"
        );
        let logout_cookie_str = logout_cookie.unwrap().to_str().unwrap();
        assert!(
            logout_cookie_str.contains("Max-Age=0")
                || logout_cookie_str.contains("expires=")
        );

        // After logging out, requesting search with the old cookie should fail
        let (status, _body) = test_request::<()>(
            &app,
            Method::GET,
            "/search",
            None,
            Some(&cookie),
            None,
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        Ok(())
    }

    #[ctb_test("tokio")]
    async fn test_post_login_password_route_wrong() -> Result<()> {
        let name = function_name!();
        let _ = lock_by_name(name)?;
        let (_state, app) = test_app();
        #[derive(serde::Serialize)]
        struct LoginPasswordForm<'a> {
            username: &'a str,
            password: &'a str,
        }
        // Make sure the user exists
        get_test_user(name);
        let login_pw_form = LoginPasswordForm {
            username: name,
            password: "WRONG_test_password",
        };
        let resp = test_request_get_response(
            &app,
            axum::http::Method::POST,
            "/login-password",
            None,
            None,
            None,
            None,
            Some(&login_pw_form),
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, false).await;
        let cookie = resp.headers().get("Set-Cookie");
        assert!(cookie.is_none(), "No Set-Cookie header found");
        Ok(())
    }

    #[ctb_test("tokio")]
    async fn test_remote_registration_and_rename_collision() -> Result<()> {
        use crate::json::maybe_value::MaybeValue;
        use crate::pc_settings::PcSettings;
        use ctb_storage::models::sync::{
            RemoteRegisterResult, set_mock_register_result,
        };

        let name = function_name!();
        let _ = lock_by_name(name)?;
        User::delete_by_name(name).ok();

        // 1. Success on initial register
        set_mock_register_result(Some(RemoteRegisterResult::Success));

        let mut settings = PcSettings::load().unwrap_or_default();
        settings.server_url =
            MaybeValue::Value("http://mock-server.test".to_string());
        settings.save().unwrap();

        let (_state, app) = test_app();

        #[derive(serde::Serialize)]
        struct RegistrationForm<'a> {
            username: &'a str,
            password: &'a str,
            password_confirm: &'a str,
        }
        let reg_form = RegistrationForm {
            username: name,
            password: THIS_TEST_USER_PASS,
            password_confirm: THIS_TEST_USER_PASS,
        };

        let resp = test_request_get_response(
            &app,
            axum::http::Method::POST,
            "/registration",
            None,
            None,
            None,
            None,
            Some(&reg_form),
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        let cookie = resp.headers().get("Set-Cookie").cloned();
        assert!(cookie.is_some(), "No Set-Cookie header found");

        let user_info = UserPublicInfo::get_by_name(name)?
            .expect("Failed to get user info");
        assert_eq!(user_info.remote_status(), "Registered");

        User::delete_by_name(name).ok();

        // 2. Offline on initial register (status should be Pending)
        set_mock_register_result(Some(RemoteRegisterResult::Offline));

        let resp = test_request_get_response(
            &app,
            axum::http::Method::POST,
            "/registration",
            None,
            None,
            None,
            None,
            Some(&reg_form),
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        let _cookie_str = resp
            .headers()
            .get("Set-Cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let user_info = UserPublicInfo::get_by_name(name)?
            .expect("Failed to get user info");
        assert_eq!(user_info.remote_status(), "Pending");

        // 3. Set mock to Conflict for background sync during login
        set_mock_register_result(Some(RemoteRegisterResult::Conflict));

        #[derive(serde::Serialize)]
        struct LoginPasswordForm<'a> {
            username: &'a str,
            password: &'a str,
        }
        let login_pw_form = LoginPasswordForm {
            username: name,
            password: THIS_TEST_USER_PASS,
        };

        let resp = test_request_get_response(
            &app,
            axum::http::Method::POST,
            "/login-password",
            None,
            None,
            None,
            None,
            Some(&login_pw_form),
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        let login_cookie = resp
            .headers()
            .get("Set-Cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Wait for background task to execute
        let mut user_info = None;
        for _ in 0..100 {
            if let Ok(Some(info)) = UserPublicInfo::get_by_name(name) {
                if info.remote_status() == "Conflict" {
                    user_info = Some(info);
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        let user_info = user_info
            .or_else(|| UserPublicInfo::get_by_name(name).ok().flatten())
            .expect("Failed to get user info");
        assert_eq!(user_info.remote_status(), "Conflict");

        let resp = test_request_get_response::<()>(
            &app,
            Method::GET,
            "/home",
            None,
            Some(&login_cookie),
            None,
            None,
            None,
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            resp.headers().get("Location").unwrap().to_str().unwrap(),
            "/rename"
        );

        let (status, body) = test_request::<()>(
            &app,
            Method::GET,
            "/rename",
            None,
            Some(&login_cookie),
            None,
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("rename"));

        // 4. Set mock to Success for the rename attempt
        set_mock_register_result(Some(RemoteRegisterResult::Success));

        #[derive(serde::Serialize)]
        struct RenameForm<'a> {
            new_username: &'a str,
        }
        let rename_form = RenameForm {
            new_username: &format!("{name}_new"),
        };
        let resp = test_request_get_response(
            &app,
            Method::POST,
            "/rename",
            None,
            Some(&login_cookie),
            None,
            None,
            Some(&rename_form),
        )
        .await;
        let resp = assert_successful_and_return_resp(resp, true).await;
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            resp.headers().get("Location").unwrap().to_str().unwrap(),
            "/home"
        );

        let new_user_info =
            UserPublicInfo::get_by_name(&format!("{name}_new"))?
                .expect("Failed to get new user info");
        assert_eq!(new_user_info.remote_status(), "Registered");

        User::delete_by_name(&format!("{name}_new")).ok();
        set_mock_register_result(None);

        Ok(())
    }
}
