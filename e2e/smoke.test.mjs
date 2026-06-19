// End-to-end smoke test for the real, packaged Linux app.
//
// Drives the actual built binary through tauri-driver + WebKitWebDriver
// (the official Tauri WebDriver path). Playwright is NOT used: it cannot
// drive a Tauri (wry/WebKitGTK) window, only browsers.
//
// Run: node e2e/smoke.test.mjs
// Exit: 0 pass, 1 fail.

import { Builder, Capabilities } from "selenium-webdriver";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const appBinary = resolve(root, "src-tauri/target/release/hd2mm");
const evidenceDir = resolve(root, "docs/evidence");
const shot = resolve(evidenceDir, "e2e-app-window.png");

const TAURI_DRIVER = resolve(process.env.HOME, ".cargo/bin/tauri-driver");
const NATIVE_DRIVER = "/usr/bin/WebKitWebDriver";
const PORT = 4444;

// Strings that prove the UI actually rendered (button tooltips + drop hint).
const EXPECTED = ["Deploy", "Purge", "Launch", "Settings", "Drop archives to add"];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function fail(msg) {
  console.error(`FAIL: ${msg}`);
  process.exitCode = 1;
}

async function main() {
  if (!existsSync(appBinary)) {
    fail(`app binary not found: ${appBinary} (run: pnpm tauri build)`);
    return;
  }
  if (!existsSync(NATIVE_DRIVER)) {
    fail(`WebKitWebDriver not found: ${NATIVE_DRIVER}`);
    return;
  }
  mkdirSync(evidenceDir, { recursive: true });

  console.log("start: tauri-driver");
  const td = spawn(TAURI_DRIVER, ["--port", String(PORT), "--native-driver", NATIVE_DRIVER], {
    stdio: "inherit",
  });
  // Give the intermediary time to bind.
  await sleep(2500);

  const caps = new Capabilities();
  caps.set("browserName", "wry");
  caps.set("tauri:options", { application: appBinary });

  let driver;
  try {
    console.log("connect: webdriver");
    driver = await new Builder()
      .withCapabilities(caps)
      .usingServer(`http://127.0.0.1:${PORT}/`)
      .build();

    // Wait for the document to load, then for the Svelte app to actually
    // render its UI (the SPA mounts after readyState=complete).
    await driver.wait(async () => {
      const state = await driver.executeScript("return document.readyState");
      return state === "complete";
    }, 15000, "document never reached readyState=complete");

    let found = [];
    await driver.wait(async () => {
      const text = await driver.executeScript("return document.body ? document.body.innerText : ''");
      found = EXPECTED.filter((s) => text.includes(s));
      return found.length > 0;
    }, 20000, "app UI never rendered an expected marker").catch(() => {});

    const title = await driver.getTitle();
    const bodyText = await driver.executeScript("return document.body ? document.body.innerText : ''");
    console.log(`title: ${JSON.stringify(title)}`);
    console.log(`body text length: ${bodyText.length}`);
    console.log(`matched UI markers: ${JSON.stringify(found)}`);

    const png = await driver.takeScreenshot();
    writeFileSync(shot, Buffer.from(png, "base64"));
    console.log("screenshot: docs/evidence/e2e-app-window.png");

    if (found.length === 0) {
      console.error(`--- body innerText (first 600 chars) ---\n${bodyText.slice(0, 600)}\n---`);
      fail(`no expected UI markers present; expected one of ${JSON.stringify(EXPECTED)}`);
    }

    if (!process.exitCode) console.log("PASS: app launched and UI rendered");
  } catch (e) {
    fail(e.message || String(e));
  } finally {
    if (driver) await driver.quit().catch(() => {});
    td.kill("SIGTERM");
  }
}

await main();
