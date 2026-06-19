use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}, sync::OnceLock};

use anyhow_tauri::{IntoTAResult, TAResult};
use regex::Regex;
use tauri::State;

use crate::{AppState, commands::settings::{do_load_settings, load_settings}, models::{Mod, manifest::{Manifest, v1}, profile::Config, settings::Settings}};

pub mod mods;
pub mod profiles;
pub mod settings;

static PATCH_REGEX: OnceLock<Regex> = OnceLock::new();
static INDEX_REGEX: OnceLock<Regex> = OnceLock::new();

/// Pattern for the patch-file triplet names the game loads from its `data` dir.
const PATCH_PATTERN: &str = r"^[0-9a-f]{16}\.patch_\d+(?:\.gpu_resources|\.stream)?$";

struct PatchFileTriplet {
    patch: Option<PathBuf>,
    gpu_resources: Option<PathBuf>,
    stream: Option<PathBuf>,
}

async fn get_patch_files_from_dir(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let patch_regex = PATCH_REGEX.get_or_init(|| Regex::new(PATCH_PATTERN).unwrap());

    let mut entries = Vec::new();
    let mut dir_reader = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = dir_reader.next_entry().await? {
        if !entry.file_type()
            .await
            .map(|t| t.is_file())
            .unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if !path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| patch_regex.is_match(n))
            .unwrap_or(false) {
            continue;
        }
        entries.push(path);
    }

    Ok(entries)
}

async fn add_files_from_dir(dir: &Path, groups: &mut HashMap<String, Vec<PatchFileTriplet>>) -> anyhow::Result<()> {
    let index_regex = INDEX_REGEX.get_or_init(|| Regex::new(r"\.patch_(\d+)").unwrap());

    let entries = get_patch_files_from_dir(dir).await?;

    let names: HashSet<String> = entries
        .iter()
        .filter_map(|p| p.file_name()?.to_str().map(|s| s[..16].to_string()))
        .collect();

    for name in names {
        let indices: HashSet<u32> = entries
            .iter()
            .filter_map(|p| {
                let fname = p.file_name()?.to_str()?;
                if !fname.starts_with(&*name) {
                    return None;
                }
                let caps = index_regex.captures(fname)?;
                caps[1].parse().ok()
            })
            .collect();

        for index in indices {
            let patch = entries.iter().find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == format!("{}.patch_{}", name, index))
                    .unwrap_or(false)
            }).cloned();

            let gpu_resources = entries.iter().find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == format!("{}.patch_{}.gpu_resources", name, index))
                    .unwrap_or(false)
            }).cloned();

            let stream = entries.iter().find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == format!("{}.patch_{}.stream", name, index))
                    .unwrap_or(false)
            }).cloned();

            groups
                .entry(name.clone())
                .or_default()
                .push(PatchFileTriplet { patch, gpu_resources, stream });
        }
    }

    Ok(())
}

async fn do_purge(data_dir: &Path) -> anyhow::Result<()> {
    let patch_files = get_patch_files_from_dir(data_dir).await?;

    futures::future::try_join_all(patch_files.iter().map(|f| tokio::fs::remove_file(f))).await?;

    Ok(())
}

/// Copy `src` to `dest`, or create an empty file when there is no source.
async fn deploy_file(src: &Option<PathBuf>, dest: PathBuf) -> anyhow::Result<()> {
    match src {
        Some(s) => { tokio::fs::copy(s, &dest).await?; }
        None => { tokio::fs::File::create(&dest).await?; }
    }
    Ok(())
}

/// Write each grouped patch triplet into the game `data` dir, applying the
/// skip-list index offset per asset name.
async fn write_groups(
    data_dir: &Path,
    settings: &Settings,
    groups: &HashMap<String, Vec<PatchFileTriplet>>,
) -> anyhow::Result<()> {
    for (name, triplets) in groups {
        let offset = if settings.has_skip_entry(name) { 1 } else { 0 };

        for (i, triplet) in triplets.iter().enumerate() {
            let index = i + offset;
            deploy_file(&triplet.patch, data_dir.join(format!("{name}.patch_{index}"))).await?;
            deploy_file(&triplet.gpu_resources, data_dir.join(format!("{name}.patch_{index}.gpu_resources"))).await?;
            deploy_file(&triplet.stream, data_dir.join(format!("{name}.patch_{index}.stream"))).await?;
        }
    }

    Ok(())
}

/// Collect the files a V1 mod contributes for its toggled options and the
/// selected sub-option of each.
async fn collect_v1_files(
    base: &Path,
    manifest: &v1::Manifest,
    toggled: &[bool],
    selected: &[usize],
    groups: &mut HashMap<String, Vec<PatchFileTriplet>>,
) -> anyhow::Result<()> {
    let Some(options) = manifest.options.as_ref() else {
        return add_files_from_dir(base, groups).await;
    };

    for (i, opt) in options.iter().enumerate() {
        if !toggled.get(i).copied().unwrap_or(false) {
            continue;
        }

        for inc in opt.include.iter().flatten() {
            add_files_from_dir(&base.join(inc), groups).await?;
        }

        let sub = opt.sub_options.as_ref()
            .and_then(|subs| selected.get(i).copied().and_then(|idx| subs.get(idx)));
        if let Some(sub) = sub {
            for inc in &sub.include {
                add_files_from_dir(&base.join(inc), groups).await?;
            }
        }
    }

    Ok(())
}

/// Collect the files an enabled mod contributes, based on its manifest + config.
async fn collect_mod_files(
    r#mod: &Mod,
    config: &Config,
    groups: &mut HashMap<String, Vec<PatchFileTriplet>>,
) -> anyhow::Result<()> {
    let base = &r#mod.directory;

    match (&r#mod.manifest, config) {
        (Manifest::Legacy(manifest), Config::Legacy { selected, .. }) => {
            match manifest.options.as_ref() {
                Some(options) => {
                    if let Some(opt) = options.get(*selected) {
                        add_files_from_dir(&base.join(opt), groups).await?;
                    }
                }
                None => add_files_from_dir(base, groups).await?,
            }
        }
        (Manifest::V1(manifest), Config::V1 { toggled, selected, .. }) => {
            collect_v1_files(base, manifest, toggled, selected, groups).await?;
        }
        (Manifest::V2(_), Config::V2 { .. }) => {
            anyhow::bail!("V2 manifest mods not supported yet");
        }
        _ => unreachable!("manifest and config version should always match"),
    }

    Ok(())
}

#[tauri::command]
pub async fn deploy(state: State<'_, AppState>, configs: Vec<Config>) -> TAResult<()> {
    let mods = state.inner().mods.lock().await;
    if mods.is_none() {
        return anyhow::anyhow!("mods not read").into_ta_result();
    }
    let mods = mods.as_ref().unwrap();

    let settings = do_load_settings(&state.base_path).await?;
    if let Err(e) = settings.validate().await {
        return anyhow::anyhow!("invalid settings: {}", e).into_ta_result();
    }

    let by_guid = mods.iter().map(|m| (m.guid(), m)).collect::<HashMap<_, _>>();
    let selected = configs.iter()
        .filter_map(|c| by_guid.get(c.uuid()).map(|m| (*m, c)))
        .collect::<Vec<_>>();

    let data_dir = settings.game_path().join("data");
    do_purge(&data_dir).await?;

    if selected.is_empty() {
        return Ok(());
    }

    let mut groups: HashMap<String, Vec<PatchFileTriplet>> = HashMap::new();
    for (r#mod, config) in selected {
        if config.enabled() {
            collect_mod_files(r#mod, config, &mut groups).await?;
        }
    }

    write_groups(&data_dir, &settings, &groups).await.into_ta_result()
}

#[tauri::command]
pub async fn purge(state: State<'_, AppState>) -> TAResult<()> {
    let settings = load_settings(state).await?;
    if let Err(e) = settings.validate().await {
        return anyhow::anyhow!("invalid settings: {}", e).into_ta_result();
    }

    let data_dir = settings.game_path().join("data");
    do_purge(&data_dir).await.into_ta_result()
}

#[cfg(test)]
mod tests {
    use super::PATCH_PATTERN;
    use regex::Regex;

    fn re() -> Regex {
        Regex::new(PATCH_PATTERN).unwrap()
    }

    #[test]
    fn matches_valid_patch_names() {
        let re = re();
        assert!(re.is_match("0cf14e223de06a26.patch_0"));
        assert!(re.is_match("0cf14e223de06a26.patch_12"));
        assert!(re.is_match("0cf14e223de06a26.patch_0.gpu_resources"));
        assert!(re.is_match("0cf14e223de06a26.patch_3.stream"));
    }

    #[test]
    fn rejects_non_patch_names() {
        let re = re();
        // Not 16 hex chars.
        assert!(!re.is_match("0cf14e223de06a2.patch_0"));
        assert!(!re.is_match("0cf14e223de06a26z.patch_0"));
        // Uppercase hex is not allowed.
        assert!(!re.is_match("0CF14E223DE06A26.patch_0"));
        // Missing index.
        assert!(!re.is_match("0cf14e223de06a26.patch_"));
        // Unrelated files in the data dir must be ignored.
        assert!(!re.is_match("game.exe"));
        assert!(!re.is_match("0cf14e223de06a26.patch_0.bak"));
        // Traversal-looking name must not match.
        assert!(!re.is_match("../0cf14e223de06a26.patch_0"));
    }
}
