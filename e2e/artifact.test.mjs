// End-to-end test for the shippable artifact (the AppImage).
//
// smoke.test.mjs drives the dev binary. This drives the packaged AppImage that
// users actually run, through tauri-driver + WebKitWebDriver, to confirm the
// bundle itself launches and renders.
//
// Requires the bundle to exist: scripts/build.sh
// Run: node e2e/artifact.test.mjs   (or: pnpm run test:artifact)
// Exit: 0 pass, 1 fail.

import { Builder, Capabilities } from "selenium-webdriver";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { existsSync, mkdirSync, readdirSync, writeFileSync, writeSync } from "node:fs";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const appimageDir = resolve(root, "src-tauri/target/release/bundle/appimage");
const evidenceDir = resolve(root, "docs/evidence");
const shot = resolve(evidenceDir, "e2e-appimage-window.png");

const TAURI_DRIVER = resolve(process.env.HOME, ".cargo/bin/tauri-driver");
const NATIVE_DRIVER = "/usr/bin/WebKitWebDriver";
const PORT = 4444;
const EXPECTED = ["Deploy", "Purge", "Launch", "Settings", "Drop archives to add"];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
// Unbuffered output: the extracted AppImage child keeps the event loop alive, so
// rely on synchronous writes plus an explicit exit rather than flush-on-exit.
const out = (m) => writeSync(1, m + "\n");
const errOut = (m) => writeSync(2, m + "\n");

let passed = false;

function findAppImage() {
  if (!existsSync(appimageDir)) return null;
  const f = readdirSync(appimageDir).find((n) => n.endsWith(".AppImage"));
  return f ? resolve(appimageDir, f) : null;
}

function cleanup(td) {
  try { td && td.kill("SIGKILL"); } catch { /* gone */ }
  // Use exact process-name matching (-x), not -f: a -f cmdline match can
  // accidentally kill unrelated processes whose arguments contain these names.
  spawnSync("pkill", ["-9", "-x", "WebKitWebDriver"]);
  spawnSync("pkill", ["-9", "-x", "hd2mm"]);
}

async function main() {
  const appImage = findAppImage();
  if (!appImage) {
    errOut(`FAIL: no AppImage in ${appimageDir} (run: scripts/build.sh)`);
    return;
  }
  if (!existsSync(NATIVE_DRIVER)) {
    errOut(`FAIL: WebKitWebDriver not found: ${NATIVE_DRIVER}`);
    return;
  }
  mkdirSync(evidenceDir, { recursive: true });
  out(`artifact: ${appImage.replace(root + "/", "")}`);

  // Run the AppImage without FUSE (containers/headless): extract and run.
  const env = { ...process.env, APPIMAGE_EXTRACT_AND_RUN: "1" };

  out("start: tauri-driver");
  const td = spawn(TAURI_DRIVER, ["--port", String(PORT), "--native-driver", NATIVE_DRIVER], {
    stdio: "ignore",
    env,
  });
  await sleep(2500);

  const caps = new Capabilities();
  caps.set("browserName", "wry");
  caps.set("tauri:options", { application: appImage });

  let driver;
  try {
    out("connect: webdriver");
    driver = await new Builder()
      .withCapabilities(caps)
      .usingServer(`http://127.0.0.1:${PORT}/`)
      .build();

    await driver.wait(async () => {
      return (await driver.executeScript("return document.readyState")) === "complete";
    }, 20000, "document never reached readyState=complete");

    let found = [];
    await driver.wait(async () => {
      const text = await driver.executeScript("return document.body ? document.body.innerText : ''");
      found = EXPECTED.filter((s) => text.includes(s));
      return found.length > 0;
    }, 20000).catch(() => {});

    out(`matched UI markers: ${JSON.stringify(found)}`);
    const png = await driver.takeScreenshot();
    writeFileSync(shot, Buffer.from(png, "base64"));
    out("screenshot: docs/evidence/e2e-appimage-window.png");

    passed = found.length > 0;
    out(passed ? "PASS: AppImage launched and UI rendered" : "FAIL: no expected UI markers");
  } catch (e) {
    errOut(`FAIL: ${e.message || e}`);
  } finally {
    if (driver) await Promise.race([driver.quit().catch(() => {}), sleep(5000)]);
    cleanup(td);
  }
}

await main();
process.exit(passed ? 0 : 1);
