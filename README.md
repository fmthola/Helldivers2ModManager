# Helldivers2ModManager — Linux Edition

A Linux-focused fork of [teutinsa/Helldivers2ModManager](https://github.com/teutinsa/Helldivers2ModManager),
a mod manager for the game Helldivers 2.

The upstream project targets Windows. This fork tracks it as `upstream` and adds
Linux support (Steam/Proton path detection, native packaging) **without breaking
Windows behavior**. The preview3 rewrite is built on [Tauri 2](https://tauri.app/)
(Rust + Svelte), which is cross-platform, so the goal here is enablement and
validation rather than a rewrite.

> Read about the original project on the upstream [website](https://teutinsa.github.io/hd2mm-site/index.html).

## Status

🚧 **Early / preview.** Based on the upstream `v2.0.0.0_preview3` tag. Builds and
launches on Linux; broader validation is in progress (see
[Validation Reports](#validation-reports)).

| Area | State |
| --- | --- |
| Build on Linux (Tauri) | ✅ Builds (`pnpm tauri build`) |
| AppImage / `.deb` packaging | ✅ Produced |
| Steam library auto-detection (incl. Flatpak Steam) | ✅ Implemented |
| Deploy / purge mods (patch files into `data/`) | ⏳ Pending end-to-end validation |
| Legacy & V1 manifest mods | ➖ Inherited from upstream (untested on Linux) |
| V2 manifest mods | ❌ Not implemented upstream yet (`todo!()`) |

## Development environment

This fork is being developed and tested on **[Bazzite](https://bazzite.gg/)**
(a Fedora atomic, gaming-focused immutable distribution). Because the host is
immutable, the toolchain runs inside an **Ubuntu [distrobox](https://distrobox.it/)
container**, which keeps build dependencies isolated from the core host system.
Helldivers 2 itself runs on the host via **Steam + Proton**.

If you build on a different distribution, the steps below should still apply —
please consider contributing a [validation report](#validation-reports).

## How modding works on Linux

Helldivers 2 runs under Proton, which executes the real Windows game, so it reads
the same `data/*.patch_*` files regardless of host OS. "Deploying" a mod simply
copies patch-file triplets (`<hex>.patch_N`, `.patch_N.gpu_resources`,
`.patch_N.stream`) into `<game>/data/`; "purge" removes them. No Wine-side
trickery is required.

The game install is detected/validated by the presence of `tools/`, `data/`,
`bin/`, and `bin/helldivers2.exe` — the Windows executable is still present under
Proton, so detection works unchanged.

## Building from source

Requirements: a Rust toolchain, Node.js + `pnpm`, and the Tauri 2 system
libraries (`webkit2gtk-4.1`, GTK 3, `libsoup-3.0`, `librsvg`, etc.).

```bash
# install JS deps
pnpm install

# dev (hot reload)
pnpm tauri dev

# release build + bundles (AppImage + .deb)
pnpm tauri build
```

Artifacts are written to `src-tauri/target/release/bundle/`.

## Running

The AppImage is self-contained (bundles GTK/WebKit):

```bash
./hd2mm_*_amd64.AppImage
# inside a container without FUSE:
APPIMAGE_EXTRACT_AND_RUN=1 ./hd2mm_*_amd64.AppImage
```

On first launch, open **Settings** — the game path should auto-detect from your
Steam libraries (e.g. `~/.local/share/Steam/steamapps/common/Helldivers 2`).
Then add mods and **Deploy**.

## Validation Reports

> This section will hold structured reports from Linux testing. It is intentionally
> scaffolded ahead of the testing work — entries will be filled in as validation
> is performed.

### Tested environments

| Date | Distro / kernel | Steam type | Proton | HD2 build | Result | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| _pending_ | Bazzite (host) + Ubuntu distrobox | Native | _tbd_ | _tbd_ | ⏳ | initial bring-up |

### Test checklist

- [ ] App builds from source on Linux
- [ ] AppImage launches on the Bazzite host
- [ ] `.deb` installs/runs in a Debian/Ubuntu environment
- [ ] Game path auto-detected (native Steam)
- [ ] Game path auto-detected (Flatpak Steam)
- [ ] Game path auto-detected (mod on second drive via `libraryfolders.vdf`)
- [ ] Add mod from `.zip`
- [ ] Add mod from `.7z`
- [ ] Add mod from `.rar`
- [ ] Path-traversal guard rejects a malicious archive (security regression test)
- [ ] Deploy writes patch files into `<game>/data/`
- [ ] Purge removes deployed patch files
- [ ] Game launches modded via Steam (`steam://launch/553850`)

### Report log

_No reports yet._

## Security

A path-traversal (zip-slip) vulnerability in 7z/RAR extraction was fixed in this
fork (a crafted archive could write outside the mod directory). See commit
`fix: prevent path traversal (zip-slip) in 7z/RAR extraction`. Additional
hardening notes (CSP, manifest asset paths) are tracked for future work.

> ⚠️ Modding online games can carry risk with anti-cheat. Use at your own
> discretion and purge mods before playing if unsure.

## Relationship to upstream

- `upstream` → https://github.com/teutinsa/Helldivers2ModManager (Windows-first source)
- `origin` → this fork (Linux-focused)

Linux work lives on the `linux/main` branch. Focused changes may be split into
branches such as `linux/steam-library-detection` or `linux/appimage-packaging`
and proposed upstream as pull requests where appropriate.

## License

MIT, same as upstream.
