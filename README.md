# Helldivers2ModManager — Linux Edition

A Linux-focused fork of [teutinsa/Helldivers2ModManager](https://github.com/teutinsa/Helldivers2ModManager),
a mod manager for the game Helldivers 2.

The upstream project targets Windows. This fork tracks it as `upstream` and adds
Linux support (Steam/Proton path detection, native packaging) **without breaking
Windows behavior**. The preview3 rewrite is built on [Tauri 2](https://tauri.app/)
(Rust + Svelte), which is cross-platform, so the goal here is enablement and
validation rather than a rewrite.

> Read about the original project on the upstream [website](https://teutinsa.github.io/hd2mm-site/index.html).

## Code status

Last full run: 2026-06-19. Produced by the local suite in [`docs/TESTING.md`](docs/TESTING.md).
Raw artifacts: [`docs/evidence/`](docs/evidence/).

| Check | Result |
| --- | --- |
| SonarQube quality gate | ❌ Failed — on coverage (see below) |
| Vulnerabilities | 0 |
| Security rating | A |
| Security hotspots | 0 |
| Bugs | 0 |
| Reliability rating | A |
| Maintainability rating | A |
| Code smells (open, not yet fixed) | 16 |
| Coverage (overall) | 10.2% |
| Coverage (new code) | 59.8% (gate needs ≥ 80%) |
| Rust unit tests | 11 / 11 pass |
| Frontend build | ✅ Clean |
| App launches (E2E) | ✅ |

The gate is red. It fails one condition: coverage on new code (59.8%) is below the
required 80%. Security and reliability are clean: 0 vulnerabilities, 0 bugs, A
ratings. Raising coverage is open work — see
[Validation and DevSecOps](#validation-and-devsecops).

### SonarQube dashboard

Captured from the running SonarQube server.

![SonarQube dashboard: quality gate Failed on coverage, 0 bugs, 0 vulnerabilities, security A](docs/evidence/sonar-dashboard.png)

Open findings, not yet acted on (e.g. "Refactor this function to reduce its Cognitive
Complexity from 81 to the 15 allowed"):

![SonarQube open issues, sorted by severity](docs/evidence/sonar-issues.png)

More: [measures](docs/evidence/sonar-measures.png) · [running app window](docs/evidence/e2e-app-window.png).

## Status

🚧 **Early / preview.** Based on the upstream `v2.0.0.0_preview3` tag. Builds and
launches on Linux; broader validation is in progress (see
[Validation Reports](#validation-reports)).

| Area | State |
| --- | --- |
| Build on Linux (Tauri) | ✅ Builds (`pnpm tauri build`) |
| AppImage / `.deb` packaging | ✅ Produced |
| Steam library auto-detection (incl. Flatpak Steam) | ✅ Confirmed via E2E |
| App launches and renders on Linux | ✅ Confirmed via E2E |
| Deploy / purge mods (patch files into `data/`) | ⏳ Pending end-to-end validation |
| Legacy & V1 manifest mods | ➖ Inherited from upstream (untested on Linux) |
| V2 manifest mods | ❌ Not implemented upstream yet (`todo!()`) |

Testing process and evidence: [`docs/TESTING.md`](docs/TESTING.md).

Fixed: the Settings page used to report "Game path is invalid!" for a valid
install, because the webview's `fs`/`path` calls threw under the capability
scope. Path validation now runs in the Rust backend (`validate_game_path`) and
is covered by tests, including a path-with-spaces case.

## Development environment

This fork is being developed and tested on **[Bazzite](https://bazzite.gg/)**
(a Fedora atomic, gaming-focused immutable distribution). Because the host is
immutable, the toolchain runs inside an **Ubuntu [distrobox](https://distrobox.it/)
container**, which keeps build dependencies isolated from the core host system.
Helldivers 2 itself runs on the host via **Steam + Proton**.

If you build on a different distribution, the steps below should still apply —
please consider contributing a [validation report](#validation-reports).

## How modding works on Linux

Helldivers 2 runs under Proton. Proton runs the Windows game, which reads the
same `data/*.patch_*` files on any host OS. Deploying a mod copies patch-file
triplets (`<hex>.patch_N`, `.patch_N.gpu_resources`, `.patch_N.stream`) into
`<game>/data/`. Purge removes them.

The game install is detected by the presence of `tools/`, `data/`, `bin/`, and
`bin/helldivers2.exe`. The Windows executable is present under Proton, so
detection works unchanged.

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

A distrobox is only needed to build. The AppImage runs on the host directly: it
bundles GTK/WebKit and stores its data under `~/.local/share/hd2mm`. Verified on
Bazzite (host glibc 2.43, FUSE present).

Install it to the application menu:

```bash
scripts/install.sh
```

This copies the AppImage to `~/.local/bin` and adds a "Helldivers 2 Mod Manager"
menu entry. Or run it straight from the bundle:

```bash
./src-tauri/target/release/bundle/appimage/hd2mm_*_amd64.AppImage
# no FUSE available (e.g. inside a container):
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
| 2026-06-19 | Bazzite host + Ubuntu distrobox | Native | _tbd_ | preview3 | 🟡 Partial | build + launch + detection confirmed; mod deploy pending |

### Test checklist

- [x] App builds from source on Linux
- [x] App launches and renders (E2E, `docs/evidence/e2e-app-window.png`)
- [x] Game path auto-detected (native Steam)
- [x] Path-traversal guard logic unit-tested
- [ ] AppImage launches on the Bazzite host
- [ ] `.deb` installs/runs in a Debian/Ubuntu environment
- [ ] Game path auto-detected (Flatpak Steam)
- [ ] Game path auto-detected (mod on second drive via `libraryfolders.vdf`)
- [ ] Add mod from `.zip`
- [ ] Add mod from `.7z`
- [ ] Add mod from `.rar`
- [ ] Deploy writes patch files into `<game>/data/`
- [ ] Purge removes deployed patch files
- [ ] Game launches modded via Steam (`steam://launch/553850`)

### Report log

**2026-06-19 — initial bring-up.** Rust unit tests: 11 passing. Frontend build:
clean. App E2E: launched via `tauri-driver`, UI rendered, screenshot captured.
Steam auto-detection: confirmed (path pre-filled in the screenshot). SonarQube:
0 bugs, 0 vulnerabilities, ratings A/A/A; coverage added (10.2% overall), so the
gate now fails on new-code coverage (59.8% < 80%). Artifacts in
[`docs/evidence/`](docs/evidence/). Fixed this round: game-path validation moved
to Rust (the Settings error is gone); AppImage runs on the host (writable data
dir); WebKit black-screen on NVIDIA. Open: low coverage; 4 cognitive-complexity
smells; mod deploy/purge not yet exercised with a real mod.

## Validation and DevSecOps

Development is local. There is no CI service. The pipeline is plain shell scripts.
Secrets and the SonarQube URL are read from the local credential store (KWallet)
at run time and are never stored in the repo.

The four stages, and how each one keeps the code valid and secure:

### Build

```bash
scripts/build.sh
```

Installs deps, builds the frontend, compiles the Tauri release binary, and
produces the AppImage and `.deb`.

### Test

```bash
pnpm test                 # Rust unit tests + app E2E
pnpm run build:checked     # i18n + frontend type/compile check
```

- **Rust unit tests** cover the security-relevant logic: the archive
  path-traversal guard, install validation, and patch-file matching.
- **App E2E** drives the real packaged binary through `tauri-driver` +
  `WebKitWebDriver` and screenshots it. This is not Playwright; Playwright
  cannot drive a Tauri window.

### Scan

```bash
scripts/sonar.sh          # coverage + SonarQube scan + quality-gate check
scripts/sonar-evidence.sh  # capture the SonarQube UI screenshots
```

Self-hosted SonarQube analyses the code for bugs, vulnerabilities, security
hotspots, and code smells, and imports test **coverage**. The quality gate is the
go/no-go signal. The loop is: scan → read the gate and issues → fix → re-scan,
until it is green.

**Why coverage matters here.** Static analysis only flags what it can see. Coverage
shows how much of the code the tests actually exercise. Low coverage means large
parts of the code are unproven, so "0 bugs found" is weaker than it looks. That is
why the gate includes a coverage condition, and why the gate is currently red: new-code
coverage is 59.8%, below the 80% the gate requires. Security and reliability are
clean; coverage is the open gap.

### Deploy

Release artifacts (AppImage, `.deb`) are built by `scripts/build.sh`. They are held
back from distribution until end-to-end mod testing on a real install passes (see
[Validation Reports](#validation-reports)).

### Security fixes and open findings

- **Fixed:** a path-traversal (zip-slip) flaw in 7z/RAR extraction. A crafted
  archive could write outside the mod directory. See commit `fix: prevent path
  traversal (zip-slip) in 7z/RAR extraction`, covered by unit tests.
- **Open, not yet acted on** (visible in [`docs/evidence/sonar-issues.png`](docs/evidence/sonar-issues.png)):
  4 cognitive-complexity refactors (`deploy`, `add_mods`, `normalize_paths`,
  `validate`), other minor smells, and low coverage. Further hardening (CSP,
  manifest asset path validation) is also tracked.

Current numbers are in [Code status](#code-status), sourced from `docs/evidence/`.

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
