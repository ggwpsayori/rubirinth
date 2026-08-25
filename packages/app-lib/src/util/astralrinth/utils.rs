///
/// This file is modified by AstralRinth / Rubirinth
///
use crate::models::astralrinth::authentication::ExternalAuthLibrary;
use crate::{Result, State};

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::{fs, io};

const PACKAGE_JSON_CONTENT: &str =
    include_str!("../../../../../apps/app-frontend/package.json");

/// Deserialize the content of package.json into a Launcher struct
pub fn read_package_json() -> io::Result<Launcher> {
    let launcher: Launcher = serde_json::from_str(PACKAGE_JSON_CONTENT)?;
    Ok(launcher)
}

#[derive(Serialize, Deserialize)]
pub struct Launcher {
    pub version: String,
}

#[derive(Deserialize)]
struct ExternalAuthLibraryRelease {
    assets: Vec<ExternalAuthLibraryAsset>,
}

#[derive(Deserialize)]
struct ExternalAuthLibraryAsset {
    name: String,
    browser_download_url: String,
}

/// Resolves the selected library path for the given provider.
pub async fn get_authlib_injector_library(
    provider_id: &str,
    library: ExternalAuthLibrary,
) -> Result<PathBuf> {
    let asset_name = load_authlib_injector_library_selection(provider_id)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "External auth library selection not found for provider {provider_id}"
            ))
            .as_error()
        })?;
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();
    let path = authlib_injector_path(&libraries_dir, library, &asset_name);
    if !path.is_file() {
        return Err(crate::ErrorKind::OtherError(format!(
            "External auth library file not found: {}",
            path.display()
        ))
        .as_error());
    }

    Ok(path)
}

/// Returns the file names of all libraries found in the local library directory.
pub async fn local_authlib_injector_libraries(
    library: ExternalAuthLibrary,
) -> Result<Vec<String>> {
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();
    let mut entries = match fs::read_dir(authlib_injector_dir(
        &libraries_dir,
        library,
    ))
    .await
    {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };

    let mut libraries = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        if let Ok(name) = entry.file_name().into_string() {
            if validate_library_asset_name(&name).is_ok() {
                libraries.push(name);
            }
        }
    }

    libraries.sort_by(|left, right| {
        let left_version = library_version(left);
        let right_version = library_version(right);
        match (left_version, right_version) {
            (Some(left), Some(right)) => right.cmp(&left),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => right.cmp(left),
        }
    });

    Ok(libraries)
}

/// Downloads and selects the specified remote library asset.
pub async fn install_authlib_injector_library(
    provider_id: &str,
    library: ExternalAuthLibrary,
    asset_name: &str,
) -> Result<()> {
    validate_library_asset_name(asset_name)?;
    let asset = fetch_external_auth_library_release(library)
        .await?
        .assets
        .into_iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Library asset not found: {asset_name}"
            ))
            .as_error()
        })?;
    install_authlib_injector_asset(provider_id, library, asset).await?;

    Ok(())
}

/// Downloads and selects the newest provider library available remotely.
pub async fn install_latest_authlib_injector_library(
    provider_id: &str,
    library: ExternalAuthLibrary,
) -> Result<PathBuf> {
    let asset = fetch_external_auth_library_release(library)
        .await?
        .assets
        .into_iter()
        .filter_map(|asset| {
            library_version(&asset.name).map(|version| (asset, version))
        })
        .max_by(|(left_asset, left_version), (right_asset, right_version)| {
            left_version
                .cmp(right_version)
                .then_with(|| left_asset.name.cmp(&right_asset.name))
        })
        .map(|(asset, _)| asset)
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(
                "No compatible external authentication library was found"
                    .to_string(),
            )
            .as_error()
        })?;

    install_authlib_injector_asset(provider_id, library, asset).await
}

async fn install_authlib_injector_asset(
    provider_id: &str,
    library: ExternalAuthLibrary,
    asset: ExternalAuthLibraryAsset,
) -> Result<PathBuf> {
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();
    let directory = authlib_injector_dir(&libraries_dir, library);
    fs::create_dir_all(&directory).await?;

    let path = authlib_injector_path(&libraries_dir, library, &asset.name);
    if !path.is_file() {
        tracing::info!(
            url = asset.browser_download_url,
            "[AR] Auth library download started"
        );
        let bytes = fetch_bytes_from_url(&asset.browser_download_url).await?;
        let relative_path = path
            .strip_prefix(&libraries_dir)
            .map_err(|_| {
                crate::ErrorKind::OtherError(
                    "Invalid library path".to_string(),
                )
                .as_error()
            })?
            .to_str()
            .ok_or_else(|| {
                crate::ErrorKind::OtherError(
                    "Invalid library path string".to_string(),
                )
                .as_error()
            })?;
        write_file_to_libraries(relative_path, &bytes).await?;
    }

    save_authlib_injector_library_selection(provider_id, &asset.name).await?;

    Ok(path)
}

/// Selects an already downloaded library file.
pub async fn select_local_authlib_injector_library(
    provider_id: &str,
    library: ExternalAuthLibrary,
    asset_name: &str,
) -> Result<()> {
    validate_library_asset_name(asset_name)?;
    let state = State::get().await?;
    let libraries_dir = state.directories.libraries_dir();
    let path = authlib_injector_path(
        &libraries_dir,
        library,
        asset_name,
    );
    if !path.is_file() {
        return Err(crate::ErrorKind::OtherError(format!(
            "Local library file not found: {asset_name}"
        ))
        .as_error());
    }

    save_authlib_injector_library_selection(provider_id, asset_name).await?;

    Ok(())
}

async fn save_authlib_injector_library_selection(
    provider_id: &str,
    asset_name: &str,
) -> Result<()> {
    let state = State::get().await?;
    sqlx::query(
        r#"
		INSERT INTO external_auth_libraries (provider_id, asset_name)
		VALUES (?, ?)
		ON CONFLICT(provider_id) DO UPDATE SET asset_name = excluded.asset_name
		"#,
    )
    .bind(provider_id)
    .bind(asset_name)
    .execute(&state.pool)
    .await?;

    Ok(())
}

pub async fn load_authlib_injector_library_selection(
    provider_id: &str,
) -> Result<Option<String>> {
    let state = State::get().await?;
    let row: Option<(String,)> = sqlx::query_as(
        r#"
		SELECT asset_name
		FROM external_auth_libraries
		WHERE provider_id = ?
		"#,
    )
    .bind(provider_id)
    .fetch_optional(&state.pool)
    .await?;

    Ok(row.map(|(asset_name,)| asset_name))
}

fn authlib_injector_dir(
    libraries_dir: &Path,
    library: ExternalAuthLibrary,
) -> PathBuf {
    let directory = library
        .cache_directory
        .split(':')
        .fold(libraries_dir.to_path_buf(), |path, component| {
            path.join(component)
        });

    directory
}

fn authlib_injector_path(
    libraries_dir: &Path,
    library: ExternalAuthLibrary,
    asset_name: &str,
) -> PathBuf {
    authlib_injector_dir(libraries_dir, library).join(asset_name)
}

async fn fetch_external_auth_library_release(
    library: ExternalAuthLibrary,
) -> Result<ExternalAuthLibraryRelease> {
    Ok(reqwest::Client::new()
        .get(library.release_url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?
        .error_for_status()?
        .json::<ExternalAuthLibraryRelease>()
        .await?)
}

fn library_version(asset_name: &str) -> Option<Vec<u64>> {
    if validate_library_asset_name(asset_name).is_err()
        || !asset_name.contains("authlib-injector")
    {
        return None;
    }

    asset_name
        .split(|character: char| {
            !character.is_ascii_digit() && character != '.'
        })
        .find_map(|candidate| {
            let version = candidate.trim_matches('.');
            if !version.contains('.') {
                return None;
            }

            version
                .split('.')
                .map(str::parse)
                .collect::<std::result::Result<Vec<u64>, _>>()
                .ok()
        })
}

fn validate_library_asset_name(asset_name: &str) -> Result<()> {
    let is_file_name = !asset_name.is_empty()
        && !asset_name.contains('/')
        && !asset_name.contains('\\')
        && !asset_name.contains("..");

    if !is_file_name || !asset_name.ends_with(".jar") {
        return Err(crate::ErrorKind::OtherError(format!(
            "Invalid library file name: {asset_name}"
        ))
        .as_error());
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

    fs::write(&output_path, bytes).await.map_err(|error| {
        tracing::error!(error = %error, "[AR] Library save failed");
        crate::ErrorKind::OtherError(format!("Failed to save file: {error}"))
            .as_error()
    })
}

/// Downloads bytes from the provided URL with a 15 second timeout.
async fn fetch_bytes_from_url(url: &str) -> Result<bytes::Bytes> {
    let client = reqwest::Client::new();
    const TIMEOUT_SECONDS: u64 = 15;

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(TIMEOUT_SECONDS),
        client.get(url).send(),
    )
    .await
    .map_err(|_| {
        tracing::error!(
            timeout_seconds = TIMEOUT_SECONDS,
            "[AR] Download timed out"
        );
        crate::ErrorKind::OtherError(format!(
            "Download timed out after {TIMEOUT_SECONDS} seconds"
        ))
        .as_error()
    })?
    .map_err(|error| {
        tracing::error!(error = %error, "[AR] Download request failed");
        crate::ErrorKind::OtherError(format!("Request error: {error}"))
            .as_error()
    })?;

    if !response.status().is_success() {
        let status = response.status().to_string();
        tracing::error!(%status, "[AR] Download failed");
        return Err(crate::ErrorKind::OtherError(format!(
            "Failed to download file: HTTP {status}"
        ))
        .as_error());
    }

    response.bytes().await.map_err(|error| {
        tracing::error!(error = %error, "[AR] Download read failed");
        crate::ErrorKind::OtherError(format!("Failed to read response bytes: {error}"))
            .as_error()
    })
}
