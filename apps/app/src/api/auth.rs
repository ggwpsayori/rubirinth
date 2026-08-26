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
