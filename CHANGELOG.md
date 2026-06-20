# Changelog

All notable changes to this Linux fork are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Versions in this fork are namespaced `…-linux.N` to avoid colliding with the
upstream `vX.Y.Z.W` tags. They track the upstream base version (`2.0.0-preview3`)
plus the Linux iteration.

## [2.0.0-preview3-linux.1.1] - 2026-06-19

### Fixed

- Mod folder naming truncated archive names at the first dot. Adding
  `SuperCreditArrows-v2.1-TEST.zip` created the folder `SuperCreditArrows-v2`,
  which collided with other dotted names and produced a false
  "mod directory already exists" error when re-adding a previously purged mod.
  Switched `file_prefix()` to `file_stem()` so only the final extension is
  stripped.

### Changed

- Refactored the four highest-complexity functions — `deploy`, `add_mods`,
  `normalize_paths`, and settings `validate` — into smaller helpers. Behavior is
  unchanged; the deploy/purge round-trip was re-verified end-to-end after the
  refactor.

### Added

- Rust unit tests: archive path-traversal (zip-slip) guard, settings and
  game-path validation, and patch-filename matching.
- Frontend unit tests (vitest): string formatting and settings model.
- SonarQube quality-gate integration, run locally with server URL and token read
  from the system credential store (KWallet); coverage wiring and scoped
  exclusions. New-code coverage 94% (threshold 80%); gate green with 0 bugs,
  0 vulnerabilities, 0 security hotspots, 0 code smells, A/A/A ratings.

### Notes

- No functional change to deploy/purge or on-disk mod layout versus `linux.1`,
  aside from the folder-naming fix above. The build was validated in-game on
  Bazzite and the deploy/purge round-trip confirmed non-destructive.

## [2.0.0-preview3-linux.1] - 2026-06-19

Initial Linux build of upstream `2.0.0.0_preview3`. Built and verified on Bazzite.
A distrobox is only used to build; running it needs nothing but the AppImage.

### Added

- AppImage packaging; runs natively on the host with no distrobox at runtime.
- Steam game-path auto-detection (native, Flatpak, and extra library folders).
- Game-path validation moved into Rust, fixing a false "invalid path" error on
  paths containing spaces (e.g. `Helldivers 2`).
- Application data stored under `~/.local/share/hd2mm` so the packaged app has a
  writable location.
- Library opens on startup when it holds mods, so installed mods are visible.
- Real command error messages are shown and logged; uncaught and popup errors are
  written to `~/.local/share/hd2mm/hd2mm.log`.

### Fixed

- Window close button did nothing on the main page; it now saves profiles and
  closes.
- WebKit black-screen on NVIDIA / suspend-resume worked around
  (`WEBKIT_DISABLE_DMABUF_RENDERER=1`).

### Security

- Path-traversal (zip-slip) guard added to 7z/RAR/zip extraction so a crafted
  archive cannot write outside the mod directory.

[2.0.0-preview3-linux.1.1]: https://github.com/fmthola/Helldivers2ModManager/releases/tag/v2.0.0-preview3-linux.1.1
[2.0.0-preview3-linux.1]: https://github.com/fmthola/Helldivers2ModManager/releases/tag/v2.0.0-preview3-linux.1
