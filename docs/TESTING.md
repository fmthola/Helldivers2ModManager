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
pnpm run test:e2e
```

What it checks:

- App launches.
- UI renders.
- Screenshot captured.

Evidence: `docs/evidence/e2e-tests.txt`, `docs/evidence/e2e-app-window.png`.

## 4. SonarQube loop

Server: self-hosted SonarQube Community. `http://localhost:9000`.

Config: `sonar-project.properties`. Key `helldivers2modmanager-linux`.

The loop:

1. Scan. `~/sonarqube/scan.sh .`
2. Report. `~/sonarqube/report.sh helldivers2modmanager-linux`
3. Validate. `~/sonarqube/validate.sh helldivers2modmanager-linux`
4. Fix the issues.
5. Re-scan.
6. Repeat until the gate passes.

`validate.sh` exit codes: `0` pass, `1` fail, `2` error.

Screenshots come from the SonarQube web UI itself. Capture them with:

```bash
SONAR_ADMIN_PASSWORD=... node scripts/capture-sonar-evidence.mjs
```

- `docs/evidence/sonar-dashboard.png` — quality gate + ratings.
- `docs/evidence/sonar-issues.png` — open issues.
- `docs/evidence/sonar-measures.png` — measures.

Last run. Gate passed. Bugs 0. Vulnerabilities 0. Ratings A/A/A.

The loop in action (this fork):

| Scan | Bugs | Reliability | Gate |
|------|------|-------------|------|
| 1 | 9 | C | OK |
| 2 | 0 | A | FAIL (1 new issue) |
| 3 | 0 | A | FAIL (same rule) |
| 4 | 0 | A | PASS |

Fixed: 7 CSS font fallbacks, 2 nullish-coalescing bugs, 1 stringify smell.

Still open: 16 smells. All pre-existing. 4 are cognitive-complexity refactors (`deploy`, `add_mods`, `normalize_paths`, `validate`). Tracked. Not blocking.

Evidence: `docs/evidence/sonar-report.txt`.

## Evidence

All artifacts live in `docs/evidence/`.

| File | Proves |
|------|--------|
| `rust-tests.txt` | Backend tests pass. |
| `frontend-build.txt` | Frontend compiles. |
| `e2e-tests.txt` | App launched via WebDriver. |
| `e2e-app-window.png` | The running window. |
| `sonar-report.txt` | Quality gate result (text). |
| `sonar-dashboard.png` | Quality gate + ratings (SonarQube UI). |
| `sonar-issues.png` | Open issues (SonarQube UI). |
| `sonar-measures.png` | Measures (SonarQube UI). |

Re-generate them by re-running each layer above.

## Run everything

```bash
pnpm test            # rust + e2e
pnpm run build:checked
~/sonarqube/scan.sh . && ~/sonarqube/validate.sh helldivers2modmanager-linux
```
