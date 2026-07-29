use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::models::astralrinth::authentication::{
    ExternalAuthProviderMetadata, ExternalOAuthPoll,
    begin_external_authentication, external_auth_providers,
    poll_external_authentication,
};
use theseus::prelude::Credentials;
use url::Url;

const EXTERNAL_OAUTH_WINDOW_LABEL: &str = "astralrinth-external-signin";
const OAUTH_SLOW_DOWN_SECONDS: u64 = 5;

/// Returns provider metadata used to build the account-selection interface.
#[tauri::command]
pub fn get_external_auth_providers() -> Vec<ExternalAuthProviderMetadata> {
    external_auth_providers()
        .iter()
        .copied()
        .map(|provider| provider.metadata())
        .collect()
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
    .title("Sign into AstralRinth")
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

/// Converts provider-supplied verification URLs into Tauri external URLs.
fn parse_oauth_url(url: &str) -> Result<Url> {
    Url::parse(url)
        .map_err(|error| other_error(format!("Invalid OAuth URL: {error}")))
}

/// Adapts an OAuth-flow failure to the serializable Tauri API error type.
fn other_error(
    message: impl Into<String>,
) -> crate::api::TheseusSerializableError {
    theseus::ErrorKind::OtherError(message.into())
        .as_error()
        .into()
}
