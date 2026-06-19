pub mod commands;
pub mod models;
pub mod archive;
pub mod utils;

use std::path::PathBuf;

use log::LevelFilter;
use tauri_plugin_log::{Target, TargetKind};
use tokio::sync::Mutex;

use crate::models::Mod;

pub struct AppState {
    base_path: PathBuf,
    mods: Mutex<Option<Vec<Mod>>>,
}

impl AppState {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            mods: Mutex::default(),
        }
    }
}

/// Per-user, writable directory for logs, settings, mods, and profiles.
///
/// Must not be the executable's directory: a packaged app (AppImage mount,
/// /usr/bin install) lives on a read-only filesystem, so writing there panics
/// at startup. Follows the XDG base directory spec.
fn data_dir() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            home.join(".local").join("share")
        });
    base.join("hd2mm")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WebKitGTK renders a black window on some GPU/compositor setups (notably
    // NVIDIA, and after suspend/resume) when its DMABUF renderer is active.
    // Disable it before the webview initializes. Respect an explicit override.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let data_dir = data_dir();
    // Create it up front so the log plugin and settings/mods writes succeed.
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("failed to create data dir {:?}: {}", data_dir, e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    LevelFilter::Debug
                } else {
                    LevelFilter::Info
                })
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    //#[cfg(not(debug_assertions))]
                    Target::new(TargetKind::Folder {
                        path: data_dir.clone(),
                        file_name: None
                    })
                ])
                .build()
        )
        .manage(AppState::new(data_dir))
        .invoke_handler(tauri::generate_handler![
            commands::mods::get_mods,
            commands::mods::delete_mod,
            commands::mods::add_mod,
            commands::mods::add_mods,
            commands::profiles::load_profiles,
            commands::profiles::save_profiles,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::settings::check_settings,
            commands::settings::detect_game_path,
            commands::settings::validate_game_path,
            commands::purge,
            commands::deploy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}