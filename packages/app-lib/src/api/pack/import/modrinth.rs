use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::{
    State,
    install::{InstallPhaseDetails, InstallProgressReporter},
    prelude::ModLoader,
    state::{AppliedContentSetPatch, EditInstance, InstanceInstallStage},
    util::io,
};

use super::{finish_import, recache_icon};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthLegacyProfile {
    pub name: Option<String>,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
    pub icon: Option<String>,
}

pub async fn is_valid_modrinth(instance_folder: PathBuf) -> bool {
    if !tokio::fs::try_exists(&instance_folder).await.unwrap_or(false) {
        return false;
    }

    if tokio::fs::try_exists(instance_folder.join("profile.json")).await.unwrap_or(false) {
        return true;
    }
    if tokio::fs::try_exists(instance_folder.join("mods")).await.unwrap_or(false) {
        return true;
    }
    if tokio::fs::try_exists(instance_folder.join("config")).await.unwrap_or(false) {
        return true;
    }
    if tokio::fs::try_exists(instance_folder.join("options.txt")).await.unwrap_or(false) {
        return true;
    }
    if tokio::fs::try_exists(instance_folder.join("saves")).await.unwrap_or(false) {
        return true;
    }

    false
}

pub async fn import_modrinth(
    base_path: PathBuf,
    instance_folder: String,
    instance_id: &str,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    let instance_path = base_path.join("profiles").join(&instance_folder);
    let target_instance_path = if tokio::fs::try_exists(&instance_path).await.unwrap_or(false) {
        instance_path
    } else {
        base_path.join(&instance_folder)
    };

    let mut name = instance_folder.clone();
    let mut game_version: Option<String> = None;
    let mut mod_loader: ModLoader = ModLoader::Fabric;
    let mut loader_version: Option<String> = None;
    let mut icon_path: Option<PathBuf> = None;

    // 1. Try querying app.db if available
    let db_path = base_path.join("app.db");
    if tokio::fs::try_exists(&db_path).await.unwrap_or(false) {
        let db_url = format!("sqlite://{}", db_path.to_string_lossy());
        if let Ok(pool) = sqlx::sqlite::SqlitePoolOptions::new().connect(&db_url).await {
            #[derive(sqlx::FromRow)]
            struct InstanceDbRow {
                name: String,
                icon_path: Option<String>,
                game_version: Option<String>,
                loader: Option<String>,
                loader_version: Option<String>,
            }

            let row = sqlx::query_as::<sqlx::Sqlite, InstanceDbRow>(
                r#"
                SELECT i.name, i.icon_path, cs.game_version, cs.loader, cs.loader_version
                FROM instances i
                LEFT JOIN instance_content_sets cs ON i.applied_content_set_id = cs.id
                WHERE i.path = ? OR i.id = ? OR i.name = ?
                LIMIT 1
                "#,
            )
            .bind(&instance_folder)
            .bind(&instance_folder)
            .bind(&instance_folder)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

            if let Some(r) = row {
                name = r.name;
                if let Some(icon) = r.icon_path {
                    let p = PathBuf::from(&icon);
                    if p.is_absolute() && p.exists() {
                        icon_path = Some(p);
                    } else if base_path.join("icons").join(&p).exists() {
                        icon_path = Some(base_path.join("icons").join(&p));
                    }
                }
                if let Some(gv) = r.game_version {
                    game_version = Some(gv);
                }
                if let Some(l) = r.loader {
                    mod_loader = ModLoader::from_string(&l);
                }
                loader_version = r.loader_version;
            }
        }
    }

    // 2. Try profile.json if not found in db
    if game_version.is_none() {
        let profile_json = target_instance_path.join("profile.json");
        if let Ok((content, _)) = io::read_any_encoding_to_string(&profile_json).await {
            if let Ok(prof) = serde_json::from_str::<ModrinthLegacyProfile>(&content) {
                if let Some(n) = prof.name {
                    name = n;
                }
                if let Some(gv) = prof.game_version {
                    game_version = Some(gv);
                }
                if let Some(l) = prof.loader {
                    mod_loader = ModLoader::from_string(&l);
                }
                if let Some(lv) = prof.loader_version {
                    loader_version = Some(lv);
                }
                if let Some(ic) = prof.icon {
                    icon_path = Some(target_instance_path.join(ic));
                }
            }
        }
    }

    let game_version = game_version.unwrap_or_else(|| "1.21.1".to_string());

    let icon = if let Some(icon) = icon_path {
        recache_icon(icon).await.unwrap_or(None)
    } else {
        None
    };

    let resolved_loader_version = if mod_loader != ModLoader::Vanilla {
        crate::launcher::get_loader_version_from_profile(
            &game_version,
            mod_loader,
            loader_version.as_deref(),
        )
        .await
        .ok()
        .flatten()
    } else {
        None
    };

    crate::api::instance::edit(
        instance_id,
        EditInstance {
            install_stage: Some(InstanceInstallStage::PackInstalling),
            name: Some(name),
            icon_path: Some(icon.map(|x| x.to_string_lossy().to_string())),
            content_set_patch: Some(AppliedContentSetPatch {
                source_kind: None,
                game_version: Some(game_version),
                protocol_version: Some(None),
                loader: Some(mod_loader),
                loader_version: Some(resolved_loader_version.map(|x| x.id)),
            }),
            ..EditInstance::default()
        },
    )
    .await?;

    let state = State::get().await?;
    finish_import(
        instance_id,
        target_instance_path,
        &state.io_semaphore,
        reporter,
        details,
    )
    .await?;

    Ok(())
}
