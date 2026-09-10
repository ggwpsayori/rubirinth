use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::models::astralrinth::authentication::{
    ExternalAuthLibraryCatalogEntry, ExternalAuthLibraryState, ExternalAuthProviderMetadata,
    external_auth_library_catalog,
    ExternalOAuthPoll, begin_external_authentication,
    external_auth_library_states, external_auth_providers,
    install_external_auth_library_version, poll_external_authentication,
    select_external_auth_library_version,
};
use theseus::prelude::*;
use url::Url;

const EXTERNAL_OAUTH_WINDOW_LABEL: &str = "rubirinth-external-signin";
const OAUTH_SLOW_DOWN_SECONDS: u64 = 5;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("auth")
        .invoke_handler(tauri::generate_handler![
            offline_login,
            get_external_auth_providers,
            get_external_auth_library_catalog,
            get_external_auth_library_states,
            install_external_auth_library,
            select_external_auth_library,
            authenticate_external_provider,
            elyby_upload_and_wear_skin,
            check_reachable,
            login,
            remove_user,
            get_default_user,
            set_default_user,
            get_users,
        ])
        .build()
}

/// Create new offline user
#[tauri::command]
pub async fn offline_login(name: &str) -> Result<Credentials> {
    let credentials = minecraft_auth::offline_auth(name).await?;
    Ok(credentials)
}

/// Returns provider metadata used to build the account-selection interface.
#[tauri::command]
pub fn get_external_auth_providers() -> Vec<ExternalAuthProviderMetadata> {
    external_auth_providers()
        .iter()
        .copied()
        .map(|provider| provider.metadata())
        .collect()
}

/// Returns the remote/fallback catalog for all external authentication libraries.
#[tauri::command]
pub async fn get_external_auth_library_catalog() -> Result<Vec<ExternalAuthLibraryCatalogEntry>> {
    Ok(external_auth_library_catalog().await?)
}

/// Returns persisted selections and locally available provider libraries.
#[tauri::command]
pub async fn get_external_auth_library_states(
) -> Result<Vec<ExternalAuthLibraryState>> {
    Ok(external_auth_library_states().await?)
}

/// Installs and selects an exact provider-library asset.
#[tauri::command]
pub async fn install_external_auth_library(
    provider: &str,
    asset_name: &str,
) -> Result<()> {
    Ok(install_external_auth_library_version(provider, asset_name).await?)
}

/// Selects an already-downloaded provider-library asset.
#[tauri::command]
pub async fn select_external_auth_library(
    provider: &str,
    asset_name: &str,
) -> Result<bool> {
    Ok(select_external_auth_library_version(provider, asset_name).await?)
}

/// Runs an external OAuth device flow and returns credentials after approval.
#[tauri::command]
pub async fn authenticate_external_provider<R: Runtime>(
    app: tauri::AppHandle<R>,
    provider: &str,
) -> Result<Option<Credentials>> {
    let flow = begin_external_authentication(provider).await?;
    let verification_url = parse_oauth_url(&flow.verification_url)?;
    let start = Utc::now();
    let mut poll_interval = flow.interval;

    if let Some(window) = app.get_webview_window(EXTERNAL_OAUTH_WINDOW_LABEL) {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        EXTERNAL_OAUTH_WINDOW_LABEL,
        tauri::WebviewUrl::External(verification_url),
    )
    .title("Sign into Rubirinth")
    .always_on_top(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::seconds(flow.expires_in) {
        if window.title().is_err() {
            return Ok(None);
        }

        tokio::time::sleep(std::time::Duration::from_secs(poll_interval)).await;
        let poll = match poll_external_authentication(&flow).await {
            Ok(poll) => poll,
            Err(error) => {
                window.close()?;
                return Err(error.into());
            }
        };
        match poll {
            ExternalOAuthPoll::Pending => {}
            ExternalOAuthPoll::SlowDown => {
                poll_interval =
                    poll_interval.saturating_add(OAUTH_SLOW_DOWN_SECONDS);
            }
            ExternalOAuthPoll::Authorized(credentials) => {
                window.close()?;
                return Ok(Some(credentials));
            }
            ExternalOAuthPoll::Denied | ExternalOAuthPoll::Expired => {
                window.close()?;
                return Ok(None);
            }
        }
    }

    window.close()?;
    Ok(None)
}

fn parse_oauth_url(url: &str) -> Result<Url> {
    Url::parse(url)
        .map_err(|error| {
            theseus::ErrorKind::OtherError(format!("Invalid OAuth URL: {error}"))
                .as_error()
                .into()
        })
}

/// Checks if the authentication servers are reachable.
#[tauri::command]
pub async fn check_reachable() -> Result<()> {
    minecraft_auth::check_reachable().await?;
    Ok(())
}

/// Authenticate a user with Microsoft OAuth
#[tauri::command]
pub async fn login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
    let flow = minecraft_auth::begin_login().await?;
    let start = Utc::now();

    if let Some(window) = app.get_webview_window("signin") {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "signin",
        tauri::WebviewUrl::External(flow.auth_request_uri.parse().map_err(
            |_| {
                theseus::ErrorKind::OtherError(
                    "Error parsing auth redirect URL".to_string(),
                )
                .as_error()
            },
        )?),
    )
    .title("Sign into Rubirinth")
    .always_on_top(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(None);
        }

        if window
            .url()?
            .as_str()
            .starts_with("https://login.live.com/oauth20_desktop.srf")
            && let Some((_, code)) =
                window.url()?.query_pairs().find(|x| x.0 == "code")
        {
            window.close()?;
            let val = minecraft_auth::finish_login(&code.clone(), flow).await?;
            return Ok(Some(val));
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}

#[tauri::command]
pub async fn remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
#[tauri::command]
pub async fn get_users() -> Result<Vec<Credentials>> {
    Ok(minecraft_auth::users().await?)
}
/// Uploads a skin to Ely.by and wears it, seamlessly authenticating if needed.
#[tauri::command]
pub async fn elyby_upload_and_wear_skin<R: Runtime>(
    app: tauri::AppHandle<R>,
    skin_base64: String,
) -> Result<u64> {
    let label = format!("elyby-skin-bridge-{}", uuid::Uuid::new_v4());

    let auth_url: Url = "https://ely.by/authorization/login"
        .parse()
        .map_err(|error| {
            theseus::ErrorKind::OtherError(format!("Invalid URL: {error}"))
                .as_error()
        })?;

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::External(auth_url),
    )
    .title("Вход на Ely.by для установки скина")
    .inner_size(520.0, 660.0)
    .center()
    .always_on_top(true)
    .build()?;

    let _ = window.request_user_attention(Some(UserAttentionType::Critical));

    let start = Utc::now();
    let mut last_eval = Utc::now() - Duration::seconds(10);

    let script_template = r#"(function() {
    if (window.__elyby_uploading || window.__elyby_done) return;
    if (!document.body) return;
    window.__elyby_uploading = true;

    function isError(res) {
        if (!res) return false;
        if (typeof res.error === 'string') {
            return !res.error.includes('success');
        }
        return Boolean(res.error);
    }

    (async () => {
        try {
            const b64 = "__SKIN_BASE64__";
            const bin = atob(b64);
            const bytes = new Uint8Array(bin.length);
            for (let i = 0; i < bin.length; i++) {
                bytes[i] = bin.charCodeAt(i);
            }
            const blob = new Blob([bytes], { type: 'image/png' });

            const upForm = new FormData();
            upForm.append('file', blob, 'skin.png');

            const upRes = await fetch('/skins/upload', {
                method: 'POST',
                body: upForm,
                headers: { 'X-Requested-With': 'XMLHttpRequest' }
            });

            const upData = await upRes.json();
            if (isError(upData)) {
                const msg = upData.text || upData.error;
                window.__elyby_done = true;
                window.location.href = 'https://ely.by/skin-applied-error?msg=' + encodeURIComponent(msg);
                return;
            }

            let skinId = upData?.id || upData?.skin?.id || upData?.skinId || upData?.data?.id || 0;
            if (!skinId && upData?.url) {
                const digits = String(upData.url).replace(/[^0-9]/g, '');
                if (digits) skinId = parseInt(digits, 10);
            }

            if (!skinId) {
                window.__elyby_done = true;
                window.location.href = 'https://ely.by/skin-applied-error?msg=' + encodeURIComponent('ID скина не найден в ответе Ely.by: ' + JSON.stringify(upData));
                return;
            }

            let wearData;
            if (window.$ && window.$.ajax) {
                wearData = await new Promise((resolve, reject) => {
                    window.$.ajax({
                        url: '/skins/wear',
                        type: 'POST',
                        data: { skinId: skinId },
                        success: resolve,
                        error: (xhr) => resolve({ error: xhr.statusText || 'network_error' })
                    });
                });
            } else {
                const wearForm = new URLSearchParams({ skinId: String(skinId) });
                const res = await fetch('/skins/wear', {
                    method: 'POST',
                    body: wearForm,
                    headers: {
                        'X-Requested-With': 'XMLHttpRequest',
                        'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8'
                    }
                });
                wearData = await res.json().catch(() => ({}));
            }

            if (isError(wearData)) {
                const msg = wearData?.text || wearData?.error || ('Не удалось надеть скин: ' + JSON.stringify(wearData));
                window.__elyby_done = true;
                window.location.href = 'https://ely.by/skin-applied-error?msg=' + encodeURIComponent(msg);
                return;
            }

            window.__elyby_done = true;
            window.location.href = 'https://ely.by/skin-applied-result?skin_id=' + skinId;
        } catch (e) {
            window.__elyby_done = true;
            const msg = e.message || String(e);
            window.location.href = 'https://ely.by/skin-applied-error?msg=' + encodeURIComponent(msg);
        } finally {
            window.__elyby_uploading = false;
        }
    })();
})()"#;

    let script = script_template.replace("__SKIN_BASE64__", &skin_base64);

    while (Utc::now() - start) < Duration::seconds(120) {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        if (Utc::now() - start) > Duration::seconds(2) && window.title().is_err() {
            return Err(theseus::ErrorKind::OtherError(
                "Окно авторизации было закрыто".to_string(),
            )
            .as_error()
            .into());
        }

        let url = match window.url() {
            Ok(u) => u,
            Err(_) => continue,
        };

        let url_str = url.as_str();

        if url_str.contains("/skin-applied-result") {
            let skin_id = url
                .query_pairs()
                .find(|(k, _)| k == "skin_id")
                .and_then(|(_, v)| v.parse::<u64>().ok())
                .unwrap_or(0);
            let _ = window.close();
            return Ok(skin_id);
        }

        if url_str.contains("/skin-applied-error") {
            let err_msg = url
                .query_pairs()
                .find(|(k, _)| k == "msg")
                .map(|(_, v)| v.to_string())
                .unwrap_or_else(|| "Ошибка при смене скина на Ely.by".into());
            let _ = window.close();
            return Err(theseus::ErrorKind::OtherError(err_msg).as_error().into());
        }

        if url_str.starts_with("https://ely.by") && !url_str.contains("/authorization") {
            if (Utc::now() - last_eval) > Duration::milliseconds(500) {
                last_eval = Utc::now();
                let _ = window.eval(&script);
            }
        }
    }

    let _ = window.close();
    Err(theseus::ErrorKind::OtherError(
        "Время ожидания авторизации на Ely.by истекло".to_string(),
    )
    .as_error()
    .into())
}

