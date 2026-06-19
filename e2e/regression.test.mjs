// Regression checks driven through tauri-driver. Parameterized by the binary
// under test (APP_UNDER_TEST) so the same checks can run against this fork and
// against a pristine upstream build for comparison.
//
// Run: APP_UNDER_TEST=/path/to/hd2mm node e2e/regression.test.mjs

import { Builder, Capabilities } from "selenium-webdriver";
import { spawn, spawnSync } from "node:child_process";
import { writeSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const APP = process.env.APP_UNDER_TEST || resolve(root, "src-tauri/target/release/hd2mm");
const PORT = Number(process.env.TD_PORT || 4444);
const TAURI_DRIVER = process.env.HOME + "/.cargo/bin/tauri-driver";
const NATIVE_DRIVER = "/usr/bin/WebKitWebDriver";
const MARKERS = ["Deploy", "Purge", "Launch", "Settings", "Drop archives to add", "Game Path"];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const out = (m) => writeSync(1, m + "\n");
const results = {};

const td = spawn(TAURI_DRIVER, ["--port", String(PORT), "--native-driver", NATIVE_DRIVER], { stdio: "ignore", env: { ...process.env, APPIMAGE_EXTRACT_AND_RUN: "1" } });
await sleep(3000);

const caps = new Capabilities();
caps.set("browserName", "wry");
caps.set("tauri:options", { application: APP });

let driver;
try {
  driver = await new Builder().withCapabilities(caps).usingServer(`http://127.0.0.1:${PORT}/`).build();
  await driver.manage().setTimeouts({ implicit: 2000, script: 15000 });

  // 1. Renders
  await driver.wait(async () => (await driver.executeScript("return document.readyState")) === "complete", 15000);
  let body = "";
  await driver.wait(async () => {
    body = await driver.executeScript("return document.body ? document.body.innerText : ''");
    return MARKERS.some((m) => body.includes(m));
  }, 20000).catch(() => {});
  results.renders = MARKERS.some((m) => body.includes(m));
  results.markers = MARKERS.filter((m) => body.includes(m));

  // 2. No false validation error on a valid auto-detected path
  results.no_invalid_error = !body.includes("invalid");

  // 3. Window controls present in the DOM
  results.minimize_present = (await driver.findElements({ css: 'button[aria-label="Minimize"]' })).length > 0;
  results.close_present = (await driver.findElements({ css: 'button[aria-label="Close"]' })).length > 0;

  // 4. Close button: click, then verify the window is actually gone. The click
  // itself can throw if the window dies mid-click, so it is isolated from the
  // verification (getTitle throwing == window closed == success).
  const xs = await driver.findElements({ css: 'button[aria-label="Close"]' });
  if (xs.length === 0) {
    results.close_works = "NO_BUTTON";
  } else {
    try { await xs[0].click(); } catch (e) { results.close_click_threw = e.name; }
    await sleep(2500);
    try {
      await driver.getTitle();
      results.close_works = false;  // window still alive => did NOT close
    } catch {
      results.close_works = true;   // window gone => closed
    }
  }
} catch (e) {
  results.fatal = e.message || String(e);
} finally {
  try { if (driver) await Promise.race([driver.quit().catch(() => {}), sleep(4000)]); } catch {}
  try { td.kill("SIGKILL"); } catch {}
  spawnSync("pkill", ["-9", "-x", "WebKitWebDriver"]);
  spawnSync("pkill", ["-9", "-x", "hd2mm"]);
}

out("REGRESSION_RESULT " + JSON.stringify(results));
process.exit(0);
