use crate::state::instances::{
    ContentEntry, InstanceFile,
    adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
    CacheBehaviour, CachedEntry, ProjectType, ReleaseChannel, State, Version,
};
use std::collections::{HashMap, HashSet};

use super::sync_content_files::{
    project_type_for_file, sync_instance_content_files,
};

#[derive(Clone, Debug)]
pub(crate) struct ContentUpdate {
    pub relative_path: String,
    pub current_version_id: String,
    pub update_version_id: String,
}

#[derive(Clone, Debug)]
struct UpdateCandidate {
    entry: Option<ContentEntry>,
    file: InstanceFile,
    project_type: ProjectType,
    project_id: String,
    current_version_id: String,
}

pub(crate) async fn check_content_updates(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<Vec<ContentUpdate>> {
    check_content_updates_with_cache_behaviours(
        instance_id,
        cache_behaviour,
        cache_behaviour,
        state,
    )
    .await
}

pub(crate) async fn refresh_content_updates(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    check_content_updates_with_cache_behaviours(
        instance_id,
        None,
        Some(CacheBehaviour::Bypass),
        state,
    )
    .await?;

    Ok(())
}

async fn check_content_updates_with_cache_behaviours(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
    update_cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<Vec<ContentUpdate>> {
    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let content_set =
        content_rows::get_applied_content_set(&instance.id, &state.pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Instance {} has no applied content set",
                    instance.id
                ))
            })?;
    let entries =
        content_rows::get_content_entries(&content_set.id, &state.pool).await?;
    let entries_by_file_id = entries
        .iter()
        .filter_map(|entry| {
            entry.file_id.as_deref().map(|file_id| (file_id, entry))
        })
        .collect::<HashMap<_, _>>();
    let files = sync_instance_content_files(&instance, state).await?;
    let hashes = files
        .iter()
        .map(|file| file.sha1.as_str())
        .collect::<Vec<_>>();
    let file_info = CachedEntry::get_file_many(
        &hashes,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let file_info_by_hash = file_info
        .into_iter()
        .map(|file| (file.hash.clone(), file))
        .collect::<HashMap<_, _>>();
    let candidates = files
        .into_iter()
        .filter_map(|file| {
            let project_type = project_type_for_file(&file)?;
            let metadata = file_info_by_hash.get(&file.sha1)?;
            Some(UpdateCandidate {
                entry: entries_by_file_id
                    .get(file.id.as_str())
                    .copied()
                    .cloned(),
                file,
                project_type,
                project_id: metadata.project_id.clone(),
                current_version_id: metadata.version_id.clone(),
            })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // Separate candidates into Modrinth vs CurseForge so they NEVER cross platforms
    let (mr_candidates, cf_candidates): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .partition(|candidate| {
            !candidate.current_version_id.starts_with("cf:")
                && !candidate.project_id.starts_with("cf:")
        });

    let mut updates_by_hash: HashMap<String, Vec<String>> = HashMap::new();

    // 1. Modrinth candidates: checked ONLY via Modrinth file update API
    if !mr_candidates.is_empty() {
        let installed_channels =
            installed_update_channels(&mr_candidates, cache_behaviour, state).await?;
        let update_keys = mr_candidates
            .iter()
            .map(|candidate| {
                update_cache_key(
                    &candidate.file,
                    candidate.project_type,
                    effective_update_channel(
                        instance.update_channel,
                        installed_channels.get(&candidate.file.sha1).copied(),
                    ),
                    &content_set.game_version,
                    content_set.loader.as_str(),
                )
            })
            .collect::<Vec<_>>();
        let update_key_refs = update_keys
            .iter()
            .map(|key| key.as_str())
            .collect::<Vec<_>>();
        let updates = CachedEntry::get_file_update_many(
            &update_key_refs,
            update_cache_behaviour,
            &state.pool,
            &state.api_semaphore,
        )
        .await?;
        for update in updates {
            updates_by_hash
                .entry(update.hash)
                .or_default()
                .push(update.update_version_id);
        }
    }

    // 2. CurseForge candidates: checked ONLY via CurseForge project versions
    if !cf_candidates.is_empty() {
        let mut cf_project_ids = HashSet::new();
        for c in &cf_candidates {
            if c.project_id.starts_with("cf:") {
                cf_project_ids.insert(c.project_id.clone());
            }
        }

        let project_id_list: Vec<String> = cf_project_ids.into_iter().collect();
        let versions_results = if project_id_list.len() <= 15 {
            let version_futures = project_id_list.iter().map(|pid| {
                CachedEntry::get_project_versions(
                    pid,
                    update_cache_behaviour,
                    &state.pool,
                    &state.api_semaphore,
                )
            });
            futures::future::join_all(version_futures).await
        } else {
            Vec::new()
        };

        let mut cf_versions_by_project: HashMap<String, Vec<Version>> = HashMap::new();
        for (pid, res) in project_id_list.into_iter().zip(versions_results) {
            if let Ok(Some(versions)) = res {
                cf_versions_by_project.insert(pid, versions);
            }
        }

        for candidate in &cf_candidates {
            if let Some(versions) = cf_versions_by_project.get(&candidate.project_id) {
                if let Some(update_version_id) = check_file_version_update(
                    &candidate.current_version_id,
                    versions,
                    &content_set.game_version,
                    content_set.loader.as_str(),
                    instance.update_channel,
                ) {
                    updates_by_hash
                        .insert(candidate.file.sha1.clone(), vec![update_version_id]);
                }
            }
        }
    }

    let all_candidates: Vec<UpdateCandidate> =
        mr_candidates.into_iter().chain(cf_candidates).collect();

    let mut output = Vec::new();
    for candidate in all_candidates {
        let update_version_id = updates_by_hash
            .remove(&candidate.file.sha1)
            .unwrap_or_default()
            .into_iter()
            .find(|update_version_id| {
                update_version_id != &candidate.current_version_id
            });

        if let Some(entry) = &candidate.entry {
            content_rows::upsert_content_update_check(
                &entry.id,
                instance.update_channel,
                update_version_id.as_deref(),
                &state.pool,
            )
            .await?;
        }

        if let Some(update_version_id) = update_version_id {
            output.push(ContentUpdate {
                relative_path: candidate.file.relative_path,
                current_version_id: candidate.current_version_id,
                update_version_id,
            });
        }
    }

    Ok(output)
}

pub(crate) fn check_file_version_update(
    current_version_id: &str,
    all_versions: &[Version],
    game_version: &str,
    loader: &str,
    preferred_update_channel: ReleaseChannel,
) -> Option<String> {
    let current_version = all_versions.iter().find(|v| v.id == current_version_id);
    let installed_channel = current_version
        .map(|v| ReleaseChannel::from_version_type(&v.version_type))
        .unwrap_or(preferred_update_channel);
    let current_date = current_version
        .map(|v| v.date_published)
        .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::MIN_UTC);
    let effective_channel = preferred_update_channel.least_stable(installed_channel);

    for version_types in effective_channel.version_type_fallbacks() {
        if !all_versions
            .iter()
            .any(|v| version_types.contains(&v.version_type.as_str()))
        {
            continue;
        }

        let mut newer_versions = all_versions
            .iter()
            .filter(|version| {
                version.id != current_version_id
                    && version.date_published > current_date
                    && version_types.contains(&version.version_type.as_str())
                    && (version.game_versions.is_empty()
                        || version.game_versions.iter().any(|gv| gv == game_version))
                    && (version.loaders.is_empty()
                        || loader.is_empty()
                        || loader.eq_ignore_ascii_case("vanilla")
                        || version.loaders.iter().any(|l| l.eq_ignore_ascii_case(loader)))
            })
            .collect::<Vec<_>>();

        newer_versions.sort_by_key(|version| std::cmp::Reverse(version.date_published));

        if let Some(newest) = newer_versions.first() {
            return Some(newest.id.clone());
        }
    }

    None
}

async fn installed_update_channels(
    candidates: &[UpdateCandidate],
    cache_behaviour: Option<CacheBehaviour>,
    state: &State,
) -> crate::Result<HashMap<String, ReleaseChannel>> {
    let version_ids = candidates
        .iter()
        .map(|candidate| candidate.current_version_id.as_str())
        .collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        &version_ids,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let channels_by_version_id = versions
        .into_iter()
        .map(|version| {
            (
                version.id,
                ReleaseChannel::from_version_type(&version.version_type),
            )
        })
        .collect::<HashMap<_, _>>();

    Ok(candidates
        .iter()
        .filter_map(|candidate| {
            channels_by_version_id
                .get(&candidate.current_version_id)
                .copied()
                .map(|channel| (candidate.file.sha1.clone(), channel))
        })
        .collect())
}

fn effective_update_channel(
    preferred: ReleaseChannel,
    installed: Option<ReleaseChannel>,
) -> ReleaseChannel {
    installed.map_or(preferred, |channel| preferred.least_stable(channel))
}

fn update_cache_key(
    file: &InstanceFile,
    project_type: ProjectType,
    channel: ReleaseChannel,
    game_version: &str,
    loader: &str,
) -> String {
    format!(
        "{}-{}-{}-{}",
        file.sha1,
        if project_type == ProjectType::Mod {
            loader.to_string()
        } else {
            project_type.get_loaders().join("+")
        },
        channel.key(),
        game_version
    )
}
