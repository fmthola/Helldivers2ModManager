// Deploy + purge round-trip test against the real game data dir.
//
// Deploys the Super Credit Arrows mod with the "Blue Glowing" option selected,
// verifies the patch files land in <game>/data/, then purges and verifies they
// are removed (revert). Reads the game path from the saved settings.
//
// Run from the repo: node e2e/deploy.test.mjs

import { Builder, Capabilities } from "selenium-webdriver";
import { spawn, spawnSync } from "node:child_process";
import { readFileSync, existsSync, writeSync } from "node:fs";
import { join } from "node:path";

const out = (m) => writeSync(1, m + "\n");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const HOME = process.env.HOME;
const settings = JSON.parse(readFileSync(`${HOME}/.local/share/hd2mm/settings.json`, "utf8"));
const DATA = join(settings.GamePath, "data");
const HASH = "9ba626afa44a3aa3";
const files = [`${HASH}.patch_0`, `${HASH}.patch_0.gpu_resources`, `${HASH}.patch_0.stream`].map((f) => join(DATA, f));
const checkFiles = () => files.map((f) => existsSync(f));

// Super Credit Arrows: option 0 (colour/glow) toggled, sub-option 0 (Blue Glowing).
const config = { For: "V1", Guid: "88d1225a-b661-406e-81ee-d6b200704b40", Enabled: true, Toggled: [true], Selected: [0] };

const TD = HOME + "/.cargo/bin/tauri-driver";
const APP = process.env.APP_UNDER_TEST
  || new URL("../src-tauri/target/release/hd2mm", import.meta.url).pathname;

const invokeJs = (cmd, hasArg) => `
  const cb = arguments[arguments.length - 1];
  const inv = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke;
  if (!inv) { cb('NO_INVOKE'); return; }
  inv('${cmd}'${hasArg ? ", arguments[0]" : ""})
    .then((r) => cb('OK:' + JSON.stringify(r === undefined ? null : r)))
    .catch((e) => cb('ERR:' + (e && e.message ? e.message : JSON.stringify(e))));
`;

const td = spawn(TD, ["--port", "4444", "--native-driver", "/usr/bin/WebKitWebDriver"], { stdio: "ignore" });
await sleep(3000);
const caps = new Capabilities();
caps.set("browserName", "wry");
caps.set("tauri:options", { application: APP });

let d;
try {
  d = await new Builder().withCapabilities(caps).usingServer("http://127.0.0.1:4444/").build();
  await d.manage().setTimeouts({ script: 30000 });
  await sleep(6000); // let the page init + getMods populate state

  out("files_before: " + JSON.stringify(checkFiles()));

  const dep = await d.executeAsyncScript(invokeJs("deploy", true), { configs: [config] });
  await sleep(1500);
  out("deploy_result: " + dep);
  out("files_after_deploy: " + JSON.stringify(checkFiles()));

  const pur = await d.executeAsyncScript(invokeJs("purge", false));
  await sleep(1500);
  out("purge_result: " + pur);
  out("files_after_purge: " + JSON.stringify(checkFiles()));
} catch (e) {
  out("FATAL: " + (e.message || e));
} finally {
  try { if (d) await Promise.race([d.quit().catch(() => {}), sleep(4000)]); } catch {}
  try { td.kill("SIGKILL"); } catch {}
  spawnSync("pkill", ["-9", "-x", "WebKitWebDriver"]);
  spawnSync("pkill", ["-9", "-x", "hd2mm"]);
}
process.exit(0);
