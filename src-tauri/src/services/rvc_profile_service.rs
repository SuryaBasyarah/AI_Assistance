use crate::{
    database::repositories::voice_profile_repository,
    error::{AppError, AppResult},
    models::{ProfileConfig, RvcProfile, VoiceList},
    services::AppState,
};
use std::path::{Path, PathBuf};

fn stable_id(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn display_from_folder(name: &str) -> String {
    name.split(['-', '_', ' '])
        .filter(|x| !x.is_empty())
        .map(|x| {
            let mut chars = x.chars();
            chars
                .next()
                .map(|f| f.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn safe_child(root: &Path, folder: &Path, name: &str) -> Option<PathBuf> {
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return None;
    }
    let path = folder.join(name);
    let parent = path.parent()?.canonicalize().ok()?;
    let root = root.canonicalize().ok()?;
    if parent == root || parent.starts_with(&root) {
        Some(path)
    } else {
        None
    }
}

fn scan_folder(root: &Path, folder: &Path, selected: Option<&str>) -> RvcProfile {
    let folder_name = folder
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("unknown");
    let id = stable_id(folder_name);
    let config_path = folder.join("profile.json");
    let config: ProfileConfig = if config_path.is_file() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default()
    } else {
        ProfileConfig::default()
    };
    let files = std::fs::read_dir(folder)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .collect::<Vec<_>>();
    let pths = files
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|v| v.to_str())
                .map(|v| v.eq_ignore_ascii_case("pth"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let indexes = files
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|v| v.to_str())
                .map(|v| v.eq_ignore_ascii_case("index"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let configured_model = config
        .model
        .as_ref()
        .and_then(|n| safe_child(root, folder, n))
        .filter(|p| p.is_file());
    let model = configured_model.or_else(|| {
        if pths.len() == 1 {
            Some(pths[0].path())
        } else {
            None
        }
    });
    let configured_index = config
        .index
        .as_ref()
        .and_then(|n| safe_child(root, folder, n))
        .filter(|p| p.is_file());
    let index = configured_index.or_else(|| {
        if indexes.len() == 1 {
            Some(indexes[0].path())
        } else {
            None
        }
    });
    let error = if !config.enabled {
        Some("Disabled in profile.json".into())
    } else if pths.is_empty() {
        Some("No .pth checkpoint found".into())
    } else if pths.len() > 1 && config.model.is_none() {
        Some("Multiple .pth files; set model in profile.json".into())
    } else if config.model.is_some() && model.is_none() {
        Some("Configured model file was not found or is unsafe".into())
    } else if indexes.len() > 1 && config.index.is_none() {
        Some("Multiple .index files; set index in profile.json".into())
    } else {
        None
    };
    let warning = if error.is_none() && index.is_none() {
        Some("No .index file; index-less conversion must be supported by the RVC engine".into())
    } else {
        None
    };
    RvcProfile {
        id: id.clone(),
        display_name: config
            .display_name
            .unwrap_or_else(|| display_from_folder(folder_name)),
        model_file: model
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|v| v.to_str())
            .map(str::to_owned),
        index_file: index
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|v| v.to_str())
            .map(str::to_owned),
        valid: error.is_none(),
        loaded: selected == Some(id.as_str()),
        warning,
        error,
        language_hint: config.language_hint.unwrap_or_else(|| "ja".into()),
        pitch: config.pitch,
        f0_method: config.f0_method,
        index_rate: config.index_rate,
        protect: config.protect,
        enabled: config.enabled,
    }
}

pub fn discover(state: &AppState) -> AppResult<VoiceList> {
    std::fs::create_dir_all(&state.voices_root)?;
    let selected = {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        crate::database::repositories::settings_repository::get(&conn, "selected_rvc_profile")?
    };
    let mut all = std::fs::read_dir(&state.voices_root)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| scan_folder(&state.voices_root, &e.path(), selected.as_deref()))
        .collect::<Vec<_>>();
    all.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    {
        let conn = state
            .db
            .0
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))?;
        voice_profile_repository::sync(&conn, &all)?;
        for p in &mut all {
            voice_profile_repository::apply_overrides(&conn, p)?;
        }
    }
    let voices = all
        .iter()
        .filter(|p| p.valid && p.enabled)
        .cloned()
        .collect();
    let invalid = all
        .iter()
        .filter(|p| !p.valid || !p.enabled)
        .cloned()
        .collect();
    Ok(VoiceList {
        voices,
        invalid,
        selected_profile: selected,
        voices_root: state.voices_root.to_string_lossy().into_owned(),
    })
}
