use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::pack::install_from::{PackDependency, PackFile, PackFileHash, PackFormat};
use crate::state::{
    Dependency, DependencyType, GalleryItem, License, Project, SideType, Version, VersionFile,
};
use path_util::SafeRelativeUtf8UnixPathBuf;

pub const CURSEFORGE_API_KEY: &str =
    "$2a$10$X70Sj4q5B9i03tElmqpvfezFNT/AQGwIa0yy5qG8Q7yhowAnLmxY.";
pub const CURSEFORGE_API_BASE: &str = "https://api.curseforge.com/v1";
pub const MINECRAFT_GAME_ID: u32 = 432;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeManifest {
    pub minecraft: CurseforgeManifestMinecraft,
    pub manifest_type: String,
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub files: Vec<CurseforgeManifestFile>,
    pub overrides: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeManifestMinecraft {
    pub version: String,
    pub mod_loaders: Vec<CurseforgeManifestModLoader>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeManifestModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    pub required: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeMod {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub links: Option<CurseforgeModLinks>,
    pub summary: Option<String>,
    pub status: u32,
    pub download_count: f64,
    pub is_featured: bool,
    pub primary_category_id: u32,
    #[serde(default)]
    pub categories: Vec<CurseforgeCategory>,
    pub class_id: Option<u32>,
    #[serde(default)]
    pub authors: Vec<CurseforgeAuthor>,
    pub logo: Option<CurseforgeLogo>,
    #[serde(default)]
    pub screenshots: Vec<CurseforgeScreenshot>,
    pub main_file_id: u32,
    #[serde(default)]
    pub latest_files: Vec<CurseforgeFile>,
    #[serde(default)]
    pub latest_files_indexes: Vec<CurseforgeFileIndex>,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub date_released: Option<DateTime<Utc>>,
    pub allow_mod_distribution: Option<bool>,
    pub game_popularity_rank: Option<u32>,
    pub is_available: bool,
    pub thumbs_up_count: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeModLinks {
    pub website_url: Option<String>,
    pub wiki_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeCategory {
    pub id: u32,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub icon_url: String,
    pub date_modified: DateTime<Utc>,
    pub is_class: Option<bool>,
    pub class_id: Option<u32>,
    pub parent_category_id: Option<u32>,
    pub display_index: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeAuthor {
    pub id: u32,
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeLogo {
    pub id: u32,
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeScreenshot {
    pub id: u32,
    pub mod_id: u32,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeFile {
    pub id: u32,
    pub game_id: u32,
    pub mod_id: u32,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String,
    pub release_type: u32,
    pub file_status: u32,
    #[serde(default)]
    pub hashes: Vec<CurseforgeFileHash>,
    pub file_date: DateTime<Utc>,
    pub file_length: u64,
    pub download_count: f64,
    pub download_url: Option<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<CurseforgeFileDependency>,
    #[serde(default)]
    pub file_fingerprint: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeFileHash {
    pub value: String,
    pub algo: u32, // 1 = Sha1, 2 = Md5
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeFileDependency {
    pub mod_id: u32,
    pub relation_type: u32, // 1 = EmbeddedLibrary, 2 = OptionalDependency, 3 = RequiredDependency, 4 = Tool, 5 = Incompatible, 6 = Include
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeFileIndex {
    pub game_version: String,
    pub file_id: u32,
    pub filename: String,
    pub release_type: u32,
    pub game_version_type_id: Option<u32>,
    pub mod_loader: Option<u32>, // 1 = Forge, 4 = Fabric, 5 = Quilt, 6 = NeoForge
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurseforgeDataResponse<T> {
    pub data: T,
}

pub fn make_curseforge_client() -> crate::Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(CURSEFORGE_API_KEY)
            .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?,
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_static("Rubirinth-App"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())
}

pub async fn get_curseforge_mod(mod_id: u32) -> crate::Result<CurseforgeMod> {
    let client = make_curseforge_client()?;
    let url = format!("{CURSEFORGE_API_BASE}/mods/{mod_id}");
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

    if !res.status().is_success() {
        return Err(crate::ErrorKind::OtherError(format!(
            "CurseForge API returned status {}: {url}",
            res.status()
        ))
        .as_error());
    }

    let data: CurseforgeDataResponse<CurseforgeMod> = res
        .json()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;
    Ok(data.data)
}

pub async fn get_curseforge_mod_description(mod_id: u32) -> crate::Result<String> {
    let client = make_curseforge_client()?;
    let url = format!("{CURSEFORGE_API_BASE}/mods/{mod_id}/description");
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

    if !res.status().is_success() {
        return Ok(String::new());
    }

    let data: CurseforgeDataResponse<String> = res
        .json()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;
    Ok(data.data)
}

pub async fn get_curseforge_file(
    mod_id: u32,
    file_id: u32,
) -> crate::Result<CurseforgeFile> {
    let client = make_curseforge_client()?;
    let url = format!("{CURSEFORGE_API_BASE}/mods/{mod_id}/files/{file_id}");
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

    if !res.status().is_success() {
        return Err(crate::ErrorKind::OtherError(format!(
            "CurseForge API returned status {}: {url}",
            res.status()
        ))
        .as_error());
    }

    let data: CurseforgeDataResponse<CurseforgeFile> = res
        .json()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;
    Ok(data.data)
}


pub async fn get_curseforge_files(
    mod_id: u32,
    page_size: u32,
) -> crate::Result<Vec<CurseforgeFile>> {
    let client = make_curseforge_client()?;
    let url = format!("{CURSEFORGE_API_BASE}/mods/{mod_id}/files?pageSize={page_size}");
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

    if !res.status().is_success() {
        return Err(crate::ErrorKind::OtherError(format!(
            "CurseForge API error (status {}): {url}",
            res.status()
        ))
        .as_error());
    }

    let val: CurseforgeDataResponse<Vec<CurseforgeFile>> = res
        .json()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

    Ok(val.data)
}

pub async fn get_curseforge_files_batch(
    file_ids: &[u32],
) -> crate::Result<Vec<CurseforgeFile>> {
    if file_ids.is_empty() {
        return Ok(Vec::new());
    }

    let client = make_curseforge_client()?;
    let url = format!("{CURSEFORGE_API_BASE}/mods/files");

    let mut all_files = Vec::new();
    for chunk in file_ids.chunks(100) {
        let body = serde_json::json!({
            "fileIds": chunk
        });

        let res = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

        if res.status().is_success() {
            let data: CurseforgeDataResponse<Vec<CurseforgeFile>> = res
                .json()
                .await
                .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;
            all_files.extend(data.data);
        }
    }

    Ok(all_files)
}

pub fn compute_curseforge_fingerprint(bytes: &[u8]) -> u32 {
    let filtered: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|&b| b != 9 && b != 10 && b != 13 && b != 32)
        .collect();
    let m: u32 = 0x5bd1e995;
    let r: u32 = 24;
    let len = filtered.len() as u32;
    let mut h: u32 = 1 ^ len;
    let mut chunks = filtered.chunks_exact(4);
    for chunk in &mut chunks {
        let mut k = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);
        h = h.wrapping_mul(m);
        h ^= k;
    }
    let remainder = chunks.remainder();
    if remainder.len() == 3 {
        h ^= (remainder[2] as u32) << 16;
        h ^= (remainder[1] as u32) << 8;
        h ^= remainder[0] as u32;
        h = h.wrapping_mul(m);
    } else if remainder.len() == 2 {
        h ^= (remainder[1] as u32) << 8;
        h ^= remainder[0] as u32;
        h = h.wrapping_mul(m);
    } else if remainder.len() == 1 {
        h ^= remainder[0] as u32;
        h = h.wrapping_mul(m);
    }
    h ^= h >> 13;
    h = h.wrapping_mul(m);
    h ^= h >> 15;
    h
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeFingerprintMatch {
    pub id: u32,
    pub file: CurseforgeFile,
    #[serde(default)]
    pub latest_files: Vec<CurseforgeFile>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeFingerprintsResponse {
    #[serde(default)]
    pub exact_matches: Vec<CurseforgeFingerprintMatch>,
}

pub async fn get_curseforge_fingerprint_matches(
    fingerprints: &[u32],
) -> crate::Result<Vec<CurseforgeFingerprintMatch>> {
    if fingerprints.is_empty() {
        return Ok(Vec::new());
    }
    let client = make_curseforge_client()?;
    let url = format!("{CURSEFORGE_API_BASE}/fingerprints");

    let mut all_matches = Vec::new();
    for chunk in fingerprints.chunks(1000) {
        let res = client
            .post(&url)
            .json(&serde_json::json!({
                "fingerprints": chunk,
            }))
            .send()
            .await
            .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

        if res.status().is_success() {
            let data: CurseforgeDataResponse<CurseforgeFingerprintsResponse> = res
                .json()
                .await
                .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;
            all_matches.extend(data.data.exact_matches);
        }
    }
    Ok(all_matches)
}

pub async fn get_curseforge_mods_batch(
    mod_ids: &[u32],
) -> crate::Result<Vec<CurseforgeMod>> {
    if mod_ids.is_empty() {
        return Ok(Vec::new());
    }
    let client = make_curseforge_client()?;
    let url = format!("{CURSEFORGE_API_BASE}/mods");

    let mut all_mods = Vec::new();
    for chunk in mod_ids.chunks(100) {
        let res = client
            .post(&url)
            .json(&serde_json::json!({
                "modIds": chunk,
            }))
            .send()
            .await
            .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

        if res.status().is_success() {
            let data: CurseforgeDataResponse<Vec<CurseforgeMod>> = res
                .json()
                .await
                .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;
            all_mods.extend(data.data);
        }
    }
    Ok(all_mods)
}

pub async fn curseforge_manifest_to_pack_format(
    manifest: &CurseforgeManifest,
) -> crate::Result<PackFormat> {
    let file_ids: Vec<u32> = manifest.files.iter().map(|f| f.file_id).collect();
    let cf_files = get_curseforge_files_batch(&file_ids).await?;

    if let Ok(state) = crate::State::get().await {
        for file in &cf_files {
            if let Some(sha1) = file.hashes.iter().find(|h| h.algo == 1).map(|h| &h.value) {
                let cf_file = crate::state::CachedFile {
                    hash: sha1.clone(),
                    project_id: format!("cf:{}", file.mod_id),
                    version_id: format!("cf:{}", file.id),
                };
                let entry = crate::state::CacheValue::File(cf_file).get_entry();
                let _ = crate::state::CachedEntry::insert_cache_entry(&entry, &state.pool).await;

                let ver = map_curseforge_file_to_version(file);
                let ver_entry = crate::state::CacheValue::Version(ver).get_entry();
                let _ = crate::state::CachedEntry::insert_cache_entry(&ver_entry, &state.pool).await;
            }
        }

        let mut mod_ids: Vec<u32> = cf_files.iter().map(|f| f.mod_id).collect();
        mod_ids.sort();
        mod_ids.dedup();
        if let Ok(mods) = get_curseforge_mods_batch(&mod_ids).await {
            for m in mods {
                let p = map_curseforge_mod_to_project(&m, None);
                let p_entry = crate::state::CacheValue::Project(p).get_entry();
                let _ = crate::state::CachedEntry::insert_cache_entry(&p_entry, &state.pool).await;
            }
        }
    }

    let files_by_id: HashMap<u32, CurseforgeFile> =
        cf_files.into_iter().map(|f| (f.id, f)).collect();

    let mut pack_files = Vec::new();
    for mf in &manifest.files {
        if let Some(file) = files_by_id.get(&mf.file_id) {
            let download_url = if let Some(ref u) = file.download_url {
                u.clone()
            } else {
                format!(
                    "https://edge.forgecdn.net/files/{}/{}/{}",
                    file.id / 1000,
                    file.id % 1000,
                    file.file_name
                )
            };

            let mut hashes = HashMap::new();
            for h in &file.hashes {
                if h.algo == 1 {
                    hashes.insert(PackFileHash::Sha1, h.value.clone());
                } else if h.algo == 2 {
                    hashes.insert(PackFileHash::Sha512, h.value.clone());
                }
            }

            let rel_path = format!("mods/{}", file.file_name);
            if let Ok(safe_path) = SafeRelativeUtf8UnixPathBuf::try_from(rel_path) {
                pack_files.push(PackFile {
                    path: safe_path,
                    hashes,
                    env: None,
                    downloads: vec![download_url],
                    file_size: file.file_length as u32,
                });
            }
        }
    }

    let mut dependencies = HashMap::new();
    dependencies.insert(
        PackDependency::Minecraft,
        manifest.minecraft.version.clone(),
    );

    for loader in &manifest.minecraft.mod_loaders {
        let id = loader.id.to_lowercase();
        if id.starts_with("forge-") {
            dependencies.insert(
                PackDependency::Forge,
                id.strip_prefix("forge-").unwrap_or(&id).to_string(),
            );
        } else if id.starts_with("fabric-") {
            dependencies.insert(
                PackDependency::FabricLoader,
                id.strip_prefix("fabric-").unwrap_or(&id).to_string(),
            );
        } else if id.starts_with("quilt-") {
            dependencies.insert(
                PackDependency::QuiltLoader,
                id.strip_prefix("quilt-").unwrap_or(&id).to_string(),
            );
        } else if id.starts_with("neoforge-") {
            dependencies.insert(
                PackDependency::NeoForge,
                id.strip_prefix("neoforge-").unwrap_or(&id).to_string(),
            );
        }
    }

    Ok(PackFormat {
        format_version: 1,
        game: "minecraft".to_string(),
        version_id: format!("cf:{}", manifest.version),
        name: manifest.name.clone(),
        summary: manifest.description.clone(),
        files: pack_files,
        dependencies,
    })
}

pub fn map_curseforge_file_to_version(file: &CurseforgeFile) -> Version {
    let version_type = match file.release_type {
        2 => "beta".to_string(),
        3 => "alpha".to_string(),
        _ => "release".to_string(),
    };

    let mut hashes = HashMap::new();
    for h in &file.hashes {
        if h.algo == 1 {
            hashes.insert("sha1".to_string(), h.value.clone());
        } else if h.algo == 2 {
            hashes.insert("md5".to_string(), h.value.clone());
        }
    }

    let download_url = file.download_url.clone().unwrap_or_else(|| {
        format!(
            "https://edge.forgecdn.net/files/{}/{}/{}",
            file.id / 1000,
            file.id % 1000,
            file.file_name
        )
    });

    let game_versions: Vec<String> = file
        .game_versions
        .iter()
        .filter(|v| v.chars().next().map_or(false, |c| c.is_ascii_digit()))
        .cloned()
        .collect();

    let loaders: Vec<String> = file
        .game_versions
        .iter()
        .filter_map(|v| {
            let lower = v.to_lowercase();
            match lower.as_str() {
                "forge" | "fabric" | "quilt" | "neoforge" => Some(lower),
                _ => None,
            }
        })
        .collect();

    let dependencies: Vec<Dependency> = file
        .dependencies
        .iter()
        .map(|d| {
            let dependency_type = match d.relation_type {
                3 => DependencyType::Required,
                5 => DependencyType::Incompatible,
                1 | 6 => DependencyType::Embedded,
                _ => DependencyType::Optional,
            };

            Dependency {
                version_id: None,
                project_id: Some(format!("cf:{}", d.mod_id)),
                file_name: None,
                dependency_type,
            }
        })
        .collect();

    Version {
        id: format!("cf:{}", file.id),
        project_id: format!("cf:{}", file.mod_id),
        author_id: "curseforge".to_string(),
        featured: false,
        name: file.display_name.clone(),
        version_number: if file.display_name.is_empty() {
            file.file_name.clone()
        } else {
            file.display_name.clone()
        },
        changelog: None,
        changelog_url: None,
        date_published: file.file_date,
        downloads: file.download_count as u32,
        version_type,
        files: vec![VersionFile {
            hashes,
            url: download_url,
            filename: file.file_name.clone(),
            primary: true,
            size: file.file_length as u32,
            file_type: None,
        }],
        dependencies,
        game_versions,
        loaders,
    }
}

pub fn map_curseforge_mod_to_project(
    mod_: &CurseforgeMod,
    body: Option<String>,
) -> Project {
    let project_type = match mod_.class_id {
        Some(4471) => "modpack",
        Some(12) => "resourcepack",
        Some(6552) => "shader",
        _ => "mod",
    }
    .to_string();

    let categories = mod_
        .categories
        .iter()
        .map(|c| c.slug.clone())
        .collect::<Vec<_>>();

    let mut game_versions = Vec::new();
    let mut loaders = Vec::new();
    for idx in &mod_.latest_files_indexes {
        if !game_versions.contains(&idx.game_version) {
            game_versions.push(idx.game_version.clone());
        }
        if let Some(loader_id) = idx.mod_loader {
            let loader_name = match loader_id {
                1 => "forge",
                4 => "fabric",
                5 => "quilt",
                6 => "neoforge",
                _ => "",
            };
            if !loader_name.is_empty() && !loaders.contains(&loader_name.to_string()) {
                loaders.push(loader_name.to_string());
            }
        }
    }

    let gallery = mod_
        .screenshots
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            GalleryItem {
                url: s.url.clone(),
                raw_url: s.url.clone(),
                featured: idx == 0,
                title: Some(s.title.clone()),
                description: s.description.clone(),
                created: mod_.date_created,
                ordering: idx as i64,
            }
        })
        .collect();

    Project {
        id: format!("cf:{}", mod_.id),
        slug: Some(mod_.slug.clone()),
        project_type,
        team: format!("cf-team-{}", mod_.id),
        organization: None,
        title: mod_.name.clone(),
        description: mod_.summary.clone().unwrap_or_default(),
        body: body.unwrap_or_else(|| mod_.summary.clone().unwrap_or_default()),
        published: mod_.date_created,
        updated: mod_.date_modified,
        approved: Some(mod_.date_created),
        status: "approved".to_string(),
        license: License {
            id: "custom".to_string(),
            name: "CurseForge".to_string(),
            url: mod_.links.as_ref().and_then(|l| l.website_url.clone()),
        },
        client_side: SideType::Required,
        server_side: SideType::Optional,
        downloads: mod_.download_count as u32,
        followers: mod_.thumbs_up_count.unwrap_or(0),
        categories: categories.clone(),
        additional_categories: Vec::new(),
        game_versions,
        loaders,
        versions: {
            let mut version_ids = mod_
                .latest_files
                .iter()
                .map(|f| format!("cf:{}", f.id))
                .collect::<Vec<_>>();
            for idx in &mod_.latest_files_indexes {
                let vid = format!("cf:{}", idx.file_id);
                if !version_ids.contains(&vid) {
                    version_ids.push(vid);
                }
            }
            version_ids
        },
        icon_url: mod_
            .logo
            .as_ref()
            .and_then(|l| l.thumbnail_url.clone().or_else(|| l.url.clone())),
        issues_url: mod_.links.as_ref().and_then(|l| l.issues_url.clone()),
        source_url: mod_.links.as_ref().and_then(|l| l.source_url.clone()),
        wiki_url: mod_.links.as_ref().and_then(|l| l.wiki_url.clone()),
        discord_url: None,
        donation_urls: None,
        gallery,
        color: None,
    }
}

pub async fn curseforge_raw_request(
    path: &str,
    method: Option<&str>,
    body: Option<serde_json::Value>,
) -> crate::Result<serde_json::Value> {
    let client = make_curseforge_client()?;
    let path_formatted = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let url = format!("{CURSEFORGE_API_BASE}{path_formatted}");
    let method_str = method.unwrap_or("GET").to_uppercase();

    let mut req = match method_str.as_str() {
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => client.get(&url),
    };

    if let Some(b) = body {
        req = req.json(&b);
    }

    let res = req
        .send()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

    if !res.status().is_success() {
        return Err(crate::ErrorKind::OtherError(format!(
            "CurseForge API error (status {}): {url}",
            res.status()
        ))
        .as_error());
    }

    let val: serde_json::Value = res
        .json()
        .await
        .map_err(|e| crate::ErrorKind::OtherError(e.to_string()).as_error())?;

    Ok(val)
}
