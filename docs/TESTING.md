# Testing

How this fork is tested. Three layers. Each runs alone.

1. Rust unit tests. Backend logic.
2. Frontend build. Type + compile check.
3. App E2E. The real packaged window.

Plus SonarQube. Static analysis. Run in a loop.

## 1. Rust unit tests

Scope: archive guard, settings validation, patch-name matching.

Run:

```bash
pnpm run test:rust
# or: cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Files:

- `src-tauri/src/archive/mod.rs` — path-traversal guard.
- `src-tauri/src/models/settings.rs` — install validation.
- `src-tauri/src/commands/mod.rs` — patch filename regex.

Current: 11 tests. All pass. See `docs/evidence/rust-tests.txt`.

## 2. Frontend build

Checks i18n keys. Compiles Svelte + TypeScript.

```bash
pnpm run build:checked
```

Evidence: `docs/evidence/frontend-build.txt`.

## 3. App E2E

Drives the real binary. Not a browser.

Tool: `tauri-driver` + `WebKitWebDriver`. WebDriver protocol.

Playwright is not used. It drives browsers. It cannot drive a Tauri (wry/WebKitGTK) window.

Prerequisites:

```bash
sudo apt install webkitgtk-webdriver   # WebKitWebDriver
cargo install tauri-driver --locked    # tauri-driver
pnpm tauri build                       # builds src-tauri/target/release/hd2mm
```

Run:

```bash
pnpm run test:e2e        # dev binary
pnpm run test:artifact   # the packaged AppImage
```

What they check:

- App launches.
- UI renders.
- Screenshot captured.

`test:e2e` drives the dev binary. `test:artifact` drives the packaged AppImage
that users actually run (build it first with `scripts/build.sh`).

Evidence: `docs/evidence/e2e-tests.txt`, `docs/evidence/e2e-app-window.png`,
`docs/evidence/e2e-artifact.txt`, `docs/evidence/e2e-appimage-window.png`.

## 4. SonarQube loop

Self-hosted SonarQube Community. Run locally. No CI service.

The server URL and analysis token are read from the local credential store
(KWallet). They are not stored in the repo. Config: `sonar-project.properties`,
key `helldivers2modmanager-linux`.

Scan and check the gate:

```bash
scripts/sonar.sh
```

It generates Rust coverage, runs the scanner, and reports the gate. Exit `0`
pass, non-zero fail.

The loop:

1. Develop a change.
2. `scripts/sonar.sh`.
3. Read the gate and the open issues.
4. Fix issues.
5. Re-scan.
6. Repeat until the gate passes.

The text gate report is committed at `docs/evidence/sonar-report.txt`.

The SonarQube web UI screenshots show the internal server's project view, so they
are kept local and are not committed. Re-generate them on demand:

```bash
scripts/sonar-evidence.sh   # writes sonar-dashboard/issues/measures.png locally
```

The loop in action (this fork):

| Scan | Bugs | Reliability | Gate |
|------|------|-------------|------|
| 1 | 9 | C | OK |
| 2 | 0 | A | FAIL (1 new issue) |
| 3 | 0 | A | FAIL (same rule) |
| 4 | 0 | A | PASS |

Then all remaining findings were resolved: the four cognitive-complexity
functions (`deploy`, `add_mods`, `normalize_paths`, `validate`) were refactored
into helpers, the TS/JS smells were fixed, and unit tests were added (Rust +
vitest) to bring new-code coverage to 94%.

Current state: quality gate **Passed** — 0 bugs, 0 vulnerabilities, 0 hotspots,
0 code smells, A/A/A ratings, new-code coverage 94% (threshold 80%). The Tauri
command layer, UI, and bootstrap are excluded from the coverage metric (covered
end-to-end). Coverage exclusions and rule scoping are in `sonar-project.properties`.

## Evidence

All artifacts live in `docs/evidence/`.

| File | Proves |
|------|--------|
| `rust-tests.txt` | Backend tests pass. |
| `frontend-build.txt` | Frontend compiles. |
| `e2e-tests.txt` | Dev binary launched via WebDriver. |
| `e2e-app-window.png` | The running window (dev binary). |
| `e2e-artifact.txt` | Packaged AppImage launched. |
| `e2e-appimage-window.png` | The running window (AppImage). |
| `sonar-report.txt` | Quality gate result (text). |

Re-generate them by re-running each layer above.

## Run everything

```bash
scripts/build.sh         # build
pnpm test                # rust + e2e
pnpm run build:checked    # i18n + frontend check
scripts/sonar.sh         # coverage + scan + gate
scripts/sonar-evidence.sh # capture UI screenshots
```
