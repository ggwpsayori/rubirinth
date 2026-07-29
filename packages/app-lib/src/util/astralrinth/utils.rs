///
/// This file is modified by AstralRinth
///
use crate::api::astralrinth::update;
use crate::event::emit::emit_info;
use crate::models::astralrinth::authentication::ExternalAuthLibrary;
use crate::{Result, State};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process;
use std::time::SystemTime;
use tokio::{fs, io};

const PACKAGE_JSON_CONTENT: &str =
    // include_str!("../../../../../apps/app-frontend/package.json");
    include_str!("../../../../../apps/app/tauri.conf.json");

/// Deserialize the content of package.json into a Launcher struct
pub fn read_package_json() -> io::Result<Launcher> {
    let launcher: Launcher = serde_json::from_str(PACKAGE_JSON_CONTENT)?;
    Ok(launcher)
}

#[derive(Serialize, Deserialize)]
pub struct Launcher {
    pub version: String,
}

/// Fetches the provider's current AuthLib Injector library or reuses its local copy.
pub async fn get_authlib_injector_library(
    library: ExternalAuthLibrary,
) -> Result<PathBuf> {
    tracing::info!("[AR] • Initializing AuthLib Injector...");
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();

    validate_library_dir(&libraries_dir, library.cache_directory).await?;
    let injector_dir = libraries_dir.join(format!(
        "astralrinth/{}/",
        library.cache_directory
    ));
    fs::create_dir_all(&injector_dir).await?;

    let mut local_injectors = Vec::new();
    if let Ok(mut entries) = fs::read_dir(&injector_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let (Some(name), Ok(meta)) = (
                path.file_name().and_then(|s| s.to_str()),
                entry.metadata().await,
            ) {
                if name.starts_with(library.asset_name_prefix) {
                    local_injectors.push((
                        path,
                        meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    ));
                }
            }
        }
    }
    local_injectors.sort_by(|a, b| b.1.cmp(&a.1));

    if !local_injectors.is_empty() {
        tracing::info!("[AR] • Local versions:");
        for (path, mtime) in &local_injectors {
            tracing::info!("  • {:?} ({:?})", path.file_name().unwrap(), mtime);
        }
    }

    let latest_local = local_injectors.first().cloned();
    let (remote_name, remote_url) =
        match extract_library_metadata(library).await {
            Ok(data) => {
                tracing::info!("[AR] • Remote: {} ({})", data.0, data.1);
                data
            }
            Err(error) => {
                tracing::warn!("[AR] • Remote failed: {}, using local", error);
                ("".to_string(), "".to_string())
            }
        };

    let remote_path = if !remote_name.is_empty() {
        Some(injector_dir.join(&remote_name))
    } else {
        None
    };

    if let Some(local_path) = &latest_local {
        let local_name = local_path.0.file_name().unwrap().to_string_lossy();
        if let Some(remote_path) = &remote_path {
            let remote_name =
                remote_path.file_name().unwrap().to_string_lossy();
            if local_name == remote_name {
                tracing::info!("[AR] • Versions match: {}", local_name);
                return Ok(local_path.0.clone());
            }
        } else {
            tracing::info!(
                "[AR] • No remote info, using local: {}",
                local_name
            );
            let _ = emit_info(&format!(
                "[AR] No remote info, using local: {}",
                local_name
            ))
            .await;
            return Ok(local_path.0.clone());
        }
    }

    let Some(remote_path) = remote_path else {
        return Err(crate::ErrorKind::NetworkErrorOccurred {
            error: "No local injector and remote unavailable".to_string(),
        }
        .as_error());
    };

    let file_name = remote_path.file_name().unwrap().to_string_lossy();
    tracing::info!("[AR] • Downloading: {}", file_name);
    let _ = emit_info(&format!("[AR] Downloading: {}", file_name)).await;

    let bytes = fetch_bytes_from_url(&remote_url).await?;
    let relative_path = remote_path
        .strip_prefix(&libraries_dir)?
        .to_string_lossy()
        .into_owned();
    write_file_to_libraries(&relative_path, &bytes).await?;

    tracing::info!("[AR] • Saved: {}", remote_path.display());
    let _ = emit_info(&format!("[AR] Saved: {}", remote_path.display())).await;
    Ok(remote_path)
}

/// Reads the provider release metadata and selects its matching injector asset.
async fn extract_library_metadata(
    library: ExternalAuthLibrary,
) -> Result<(String, String)> {
    let response = reqwest::get(library.release_url).await.map_err(|e| {
        tracing::error!(
            "[AR] • Failed to fetch provider library release JSON: {:?}",
            e
        );
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!(
                "Failed to fetch provider library release JSON: {}",
                e
            ),
        }
        .as_error()
    })?;

    let json: serde_json::Value = response.json().await.map_err(|e| {
        tracing::error!(
            "[AR] • Failed to parse provider library release JSON: {:?}",
            e
        );
        crate::ErrorKind::ParseError {
            reason: format!(
                "Failed to parse provider library release JSON: {}",
                e
            ),
        }
        .as_error()
    })?;

    let assets =
        json.get("assets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::ErrorKind::ParseError {
                    reason: "Missing 'assets' array".into(),
                }
                .as_error()
            })?;

    let asset = assets
        .iter()
        .find(|a| {
            a.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.starts_with(library.asset_name_prefix))
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: format!(
					"No matching asset starting with {} in provider release response.",
					library.asset_name_prefix
                ),
            }
            .as_error()
        })?;

    let download_url = asset
        .get("browser_download_url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: "Missing 'browser_download_url'".into(),
            }
            .as_error()
        })?
        .to_string();

    let asset_name = asset
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            crate::ErrorKind::ParseError {
                reason: "Missing 'name'".into(),
            }
            .as_error()
        })?
        .to_string();

    Ok((asset_name, download_url))
}

/// Initialize the update launcher.
pub async fn init_update_launcher(
    download_url: &str,
    local_filename: &str,
    os_type: &str,
) -> Result<()> {
    tracing::info!("[AR] • Initialize downloading from • {:?}", download_url);
    tracing::info!("[AR] • Save local file name • {:?}", local_filename);
    tracing::info!("[AR] • OS type • {}", os_type);

    if let Err(e) = update::get_resource(
        download_url,
        local_filename,
        os_type,
    )
    .await
    {
        eprintln!(
            "[AR] • An error occurred! Failed to download the file: {}",
            e
        );
    } else {
        println!("[AR] • Code finishes without errors.");
        process::exit(0)
    }
    Ok(())
}

/// Validating the `astralrinth/{target_directory}/` directory exists inside the libraries/astralrinth directory.
async fn validate_library_dir(
    libraries_dir: &PathBuf,
    validation_directory: &str,
) -> Result<()> {
    let astralrinth_path =
        libraries_dir.join(format!("astralrinth/{}", validation_directory));
    if !astralrinth_path.exists() {
        tokio::fs::create_dir_all(&astralrinth_path)
            .await
            .map_err(|e| {
                tracing::error!(
                    "[AR] • Failed to create {} directory: {:?}",
                    astralrinth_path.display(),
                    e
                );
                crate::ErrorKind::IOErrorOccurred {
                    error: format!(
                        "Failed to create {} directory: {}",
                        astralrinth_path.display(),
                        e
                    ),
                }
                .as_error()
            })?;
        tracing::info!(
            "[AR] • Created missing {} directory",
            astralrinth_path.display()
        );
    }
    Ok(())
}

/// Saves the downloaded bytes to the `libraries` directory using the given relative path.
async fn write_file_to_libraries(
    relative_path: &str,
    bytes: &bytes::Bytes,
) -> Result<()> {
    let state = State::get().await?;
    let output_path = state.directories.libraries_dir().join(relative_path);

    fs::write(&output_path, bytes).await.map_err(|e| {
        tracing::error!("[AR] • Failed to save file: {:?}", e);
        crate::ErrorKind::IOErrorOccurred {
            error: format!("Failed to save file: {e}"),
        }
        .as_error()
    })
}

/// Downloads bytes from the provided URL with a 15 second timeout.
async fn fetch_bytes_from_url(url: &str) -> Result<bytes::Bytes> {
    // Create client instance with request timeout.
    let client = reqwest::Client::new();
    const TIMEOUT_SECONDS: u64 = 5;

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(TIMEOUT_SECONDS),
        client.get(url).send(),
    )
    .await
    .map_err(|_| {
        tracing::error!(
            "[AR] • Download timed out after {} seconds",
            TIMEOUT_SECONDS
        );
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!(
                "Download timed out after {TIMEOUT_SECONDS} seconds"
            )
            .to_string(),
        }
        .as_error()
    })?
    .map_err(|e| {
        tracing::error!("[AR] • Request error: {:?}", e);
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Request error: {e}"),
        }
        .as_error()
    })?;

    if !response.status().is_success() {
        let status = response.status().to_string();
        tracing::error!("[AR] • Failed to download file: HTTP {}", status);
        return Err(crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to download file: HTTP {status}"),
        }
        .as_error());
    }

    response.bytes().await.map_err(|e| {
        tracing::error!("[AR] • Failed to read response bytes: {:?}", e);
        crate::ErrorKind::NetworkErrorOccurred {
            error: format!("Failed to read response bytes: {e}"),
        }
        .as_error()
    })
}
