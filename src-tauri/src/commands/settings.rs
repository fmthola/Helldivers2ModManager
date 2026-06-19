use std::path::{Path, PathBuf};

use anyhow_tauri::{IntoTAResult, TAResult};
use regex::Regex;
use tauri::State;

use crate::{AppState, models::settings::Settings};

const SETTINGS_FILE: &'static str = "settings.json";

/// Try to locate the Helldivers 2 install directory on Linux by scanning the
/// usual Steam library locations (native, Flatpak, and any extra libraries
/// registered in `libraryfolders.vdf`, e.g. a second drive or SD card).
fn do_detect_game_path() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var("HOME").ok()?);

    // Candidate Steam roots, including Flatpak Steam which is common on Bazzite.
    let steam_roots = [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(".steam/root"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ];

    let path_regex = Regex::new(r#""path"\s*"([^"]+)""#).ok()?;

    // Build the full set of library roots: each Steam root plus every library
    // path it lists in libraryfolders.vdf.
    let mut libraries: Vec<PathBuf> = Vec::new();
    for root in steam_roots.iter() {
        if !root.is_dir() {
            continue;
        }
        libraries.push(root.clone());

        let vdf = root.join("steamapps/libraryfolders.vdf");
        if let Ok(contents) = std::fs::read_to_string(&vdf) {
            for cap in path_regex.captures_iter(&contents) {
                libraries.push(PathBuf::from(cap[1].replace("\\\\", "/")));
            }
        }
    }

    // Return the first library that actually contains the game.
    for lib in libraries {
        let candidate = lib.join("steamapps/common/Helldivers 2");
        if candidate.join("data").is_dir() {
            log::info!("Auto-detected game path: {:?}", &candidate);
            return Some(candidate);
        }
    }

    log::info!("Could not auto-detect game path.");
    None
}

/// Returns the auto-detected Helldivers 2 install path, or `null` if not found.
#[tauri::command]
pub async fn detect_game_path() -> TAResult<Option<String>> {
    Ok(do_detect_game_path().and_then(|p| p.to_str().map(String::from)))
}

pub async fn do_load_settings(base_path: &Path) -> anyhow::Result<Settings> {
    log::info!("Loading settings...");
    let settings_file = base_path.join(SETTINGS_FILE);

    log::info!("Checking if {:?}", &settings_file);
    let settings = if tokio::fs::try_exists(&settings_file).await? {
        log::info!("Found.");

        let data = tokio::fs::read(&settings_file).await?;
        serde_json::from_slice(&data)?
    } else {
        log::info!("Using default.");

        Settings::V1 {
            game_path: PathBuf::new(),
            skip_list: vec![]
        }
    };

    log::info!("Settings loaded.");
    Ok(settings)
}

pub async fn do_check_settings(base_path: &Path) -> anyhow::Result<bool> {
    log::info!("Checking settings...");

    match do_load_settings(base_path).await {
        Ok(settings) => {
            match settings.validate().await {
                Ok(()) => {
                    log::info!("Settings vaid.");
                    Ok(true)
                }
                Err(e) => {
                    log::error!("Settings invalid: {}", e);       
                    Ok(false)
                }
            }
        }
        Err(e) => Err(e)
    }
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> TAResult<Settings> {
    do_load_settings(&state.base_path).await.into_ta_result()
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: Settings) -> TAResult<()> {
    let data = serde_json::to_vec_pretty(&settings).into_ta_result()?;
    tokio::fs::write(state.base_path.join(SETTINGS_FILE), data).await.into_ta_result()
}

#[tauri::command]
pub  async fn check_settings(state: State<'_, AppState>) -> TAResult<bool> {
    do_check_settings(&state.base_path).await.into_ta_result()
}