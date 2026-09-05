use std::path::PathBuf;

use crate::{
    event::{
        CommandPayload,
        emit::{emit_command, emit_warning},
    },
    util::io,
};
use url::form_urlencoded;
use urlencoding::decode;

/// Handles external functions (such as through URL deep linkage)
/// Link is extracted value (link) in somewhat URL format, such as
/// subdomain1/subdomain2
/// (Does not include modrinth://)
pub async fn handle_url(sublink: &str) -> crate::Result<CommandPayload> {
    Ok(match sublink.split_once('/') {
        // /mod/{id}   -    Installs a mod of mod id
        Some(("mod", id)) => CommandPayload::InstallMod { id: id.to_string() },
        // /version/{id}   -    Installs a specific version of id
        Some(("version", id)) => {
            CommandPayload::InstallVersion { id: id.to_string() }
        }
        // /modpack/{id}   -    Installs a modpack of modpack id
        Some(("modpack", id)) => {
            CommandPayload::InstallModpack { id: id.to_string() }
        }
        // /server/{id}   -    Opens a server project page and triggers play flow
        Some(("server", id)) => {
            CommandPayload::InstallServer { id: id.to_string() }
        }
        // /share/{invite_id}
        Some(("share", raw)) => {
            let (raw, _) = raw.split_once('?').unwrap_or((raw, ""));

            match decode(raw) {
                Ok(decoded) => CommandPayload::InstallSharedInstanceInvite {
                    invite_id: decoded.to_string(),
                },
                Err(e) => {
                    emit_warning(&format!(
                        "Invalid UTF-8 in shared instance invite path: {e}"
                    ))
                    .await?;
                    return Err(crate::ErrorKind::InputError(format!(
                        "Invalid UTF-8 in shared instance invite path: {e}"
                    ))
                    .into());
                }
            }
        }
        // /launch/instance/{id}   -    Launches an instance
        Some(("launch", rest)) if rest.starts_with("instance/") => {
            let raw = rest.trim_start_matches("instance/");
            let (raw, query) = raw.split_once('?').unwrap_or((raw, ""));
            let mut server = None;
            let mut singleplayer_world = None;

            for (key, value) in form_urlencoded::parse(query.as_bytes()) {
                match &*key {
                    "server" => server = Some(value.into_owned()),
                    "singleplayer_world" => {
                        singleplayer_world = Some(value.into_owned());
                    }
                    _ => {}
                }
            }

            if server.is_some() && singleplayer_world.is_some() {
                emit_warning(
                    "Invalid command, cannot launch both a server and a singleplayer world",
                )
                .await?;
                return Err(crate::ErrorKind::InputError(
                    "Cannot launch both a server and a singleplayer world"
                        .to_string(),
                )
                .into());
            }

            match decode(raw) {
                Ok(decoded) => CommandPayload::LaunchInstance {
                    id: decoded.to_string(),
                    server,
                    singleplayer_world,
                },
                Err(e) => {
                    emit_warning(&format!(
                        "Invalid UTF-8 in instance path: {e}"
                    ))
                    .await?;
                    return Err(crate::ErrorKind::InputError(format!(
                        "Invalid UTF-8 in instance path: {e}"
                    ))
                    .into());
                }
            }
        }
        _ => {
            emit_warning(&format!(
                "Invalid command, unrecognized path: {sublink}"
            ))
            .await?;
            return Err(crate::ErrorKind::InputError(format!(
                "Invalid command, unrecognized path: {sublink}"
            ))
            .into());
        }
    })
}


/// Handles external CurseForge functions (such as through URL deep linkage)
/// e.g. curseforge://install?addonId=303541&source=cf_website
pub async fn handle_curseforge_url(sublink: &str) -> crate::Result<CommandPayload> {
    let sublink_clean = sublink.trim_start_matches('/');
    let (path, query) = sublink_clean.split_once('?').unwrap_or((sublink_clean, ""));

    let mut addon_id = None;
    let mut file_id = None;

    if !query.is_empty() {
        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            match key.to_ascii_lowercase().as_str() {
                "addonid" | "addon_id" | "projectid" | "project_id" | "id" | "modid" | "mod_id" => {
                    addon_id = Some(value.into_owned());
                }
                "fileid" | "file_id" | "versionid" | "version_id" => {
                    file_id = Some(value.into_owned());
                }
                _ => {}
            }
        }
    }

    let path_trimmed = path.trim_matches('/');
    let path_segments: Vec<&str> = path_trimmed.split('/').filter(|s| !s.is_empty()).collect();

    if addon_id.is_none() && file_id.is_none() {
        match path_segments.as_slice() {
            ["install", aid, fid] | ["mod", aid, fid] | ["project", aid, fid] => {
                addon_id = Some(aid.to_string());
                file_id = Some(fid.to_string());
            }
            ["install", id] | ["mod", id] | ["modpack", id] | ["project", id] | ["addon", id] => {
                addon_id = Some(id.to_string());
            }
            ["file", fid] | ["version", fid] => {
                file_id = Some(fid.to_string());
            }
            [id] if id.parse::<u32>().is_ok() => {
                addon_id = Some(id.to_string());
            }
            _ => {}
        }
    }

    if let Some(fid) = file_id {
        let raw_fid = fid.strip_prefix("cf:").unwrap_or(&fid);
        return Ok(CommandPayload::InstallVersion {
            id: format!("cf:{raw_fid}"),
        });
    }

    if let Some(aid) = addon_id {
        let raw_aid = aid.strip_prefix("cf:").unwrap_or(&aid);
        return Ok(CommandPayload::InstallMod {
            id: format!("cf:{raw_aid}"),
        });
    }

    emit_warning(&format!(
        "Invalid CurseForge command, unrecognized path or missing addonId: {sublink}"
    ))
    .await?;
    Err(crate::ErrorKind::InputError(format!(
        "Invalid CurseForge command, unrecognized path or missing addonId: {sublink}"
    ))
    .into())
}

pub async fn parse_command(
    command_string: &str,
) -> crate::Result<CommandPayload> {
    tracing::debug!("Parsing external command");

    let clean = if let Ok(mut list) = serde_json::from_str::<Vec<String>>(command_string) {
        if !list.is_empty() {
            list.remove(0)
        } else {
            command_string.to_string()
        }
    } else if let Ok(single) = serde_json::from_str::<String>(command_string) {
        single
    } else {
        command_string.trim_matches('"').to_string()
    };

    // modrinth://some-command or curseforge://some-command
    // This occurs when following a web redirect link
    if let Some(sublink) = clean.strip_prefix("modrinth://") {
        Ok(handle_url(sublink).await?)
    } else if let Some(sublink) = clean.strip_prefix("curseforge://") {
        Ok(handle_curseforge_url(sublink).await?)
    } else if let Some(sublink) = clean.strip_prefix("curseforge:") {
        Ok(handle_curseforge_url(sublink.trim_start_matches('/')).await?)
    } else {
        // We assume anything else is a filepath to an .mrpack file
        let path = PathBuf::from(&clean);
        let path = io::canonicalize(path)?;
        if let Some(ext) = path.extension()
            && ext == "mrpack"
        {
            return Ok(CommandPayload::RunMRPack {
                path: path.to_string_lossy().into_owned(),
            });
        }
        emit_warning(&format!(
            "Invalid command, unrecognized filetype: {}",
            path.display()
        ))
        .await?;
        Err(crate::ErrorKind::InputError(format!(
            "Invalid command, unrecognized filetype: {}",
            path.display()
        ))
        .into())
    }
}

pub async fn parse_and_emit_command(command_string: &str) -> crate::Result<()> {
    let command = parse_command(command_string).await?;
    emit_command(command).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_curseforge_install_addon_id() {
        let cmd = parse_command("curseforge://install?addonId=303541&source=cf_website").await.unwrap();
        match cmd {
            CommandPayload::InstallMod { id } => assert_eq!(id, "cf:303541"),
            _ => panic!("Expected InstallMod"),
        }
    }

    #[tokio::test]
    async fn test_parse_curseforge_install_with_file_id() {
        let cmd = parse_command("curseforge://install?addonId=303541&fileId=123456").await.unwrap();
        match cmd {
            CommandPayload::InstallVersion { id } => assert_eq!(id, "cf:123456"),
            _ => panic!("Expected InstallVersion"),
        }
    }

    #[tokio::test]
    async fn test_parse_curseforge_json_array() {
        let cmd = parse_command(r#"["curseforge://install?addonId=303541"]"#).await.unwrap();
        match cmd {
            CommandPayload::InstallMod { id } => assert_eq!(id, "cf:303541"),
            _ => panic!("Expected InstallMod"),
        }
    }

    #[tokio::test]
    async fn test_parse_curseforge_path_format() {
        let cmd = parse_command("curseforge://install/303541").await.unwrap();
        match cmd {
            CommandPayload::InstallMod { id } => assert_eq!(id, "cf:303541"),
            _ => panic!("Expected InstallMod"),
        }
    }
}
