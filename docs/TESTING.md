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

Screenshots come from the SonarQube web UI itself:

```bash
scripts/sonar-evidence.sh
```

- `docs/evidence/sonar-dashboard.png` — quality gate + ratings.
- `docs/evidence/sonar-issues.png` — open issues.
- `docs/evidence/sonar-measures.png` — measures.

The loop in action (this fork):

| Scan | Bugs | Reliability | Gate |
|------|------|-------------|------|
| 1 | 9 | C | OK |
| 2 | 0 | A | FAIL (1 new issue) |
| 3 | 0 | A | FAIL (same rule) |
| 4 | 0 | A | PASS |

Fixed: 7 CSS font fallbacks, 2 nullish-coalescing bugs, 1 stringify smell.

Still open: pre-existing smells, not yet acted on. 4 are cognitive-complexity
refactors (`deploy`, `add_mods`, `normalize_paths`, `validate`). Shown in
`docs/evidence/sonar-issues.png`. Tracked. Not blocking.

That table was the violation-fixing loop. Coverage was then added and is reported
to SonarQube. It is low (10.2% overall), so the gate is now red on new-code
coverage (59.8% < 80%). Raising coverage is open work. Security and reliability
stay clean.

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
scripts/build.sh         # build
pnpm test                # rust + e2e
pnpm run build:checked    # i18n + frontend check
scripts/sonar.sh         # coverage + scan + gate
scripts/sonar-evidence.sh # capture UI screenshots
```
