import fs from "node:fs";
import { join as pathJoin } from "node:path";

import { disableReusedWorkers } from "../../disable-reused-workers.mjs";

const packageDir = pathJoin(import.meta.dirname, "..");
disableReusedWorkers(pathJoin(packageDir, "minify.wasi-browser.js"));

const path = pathJoin(packageDir, "index.js");

let data = fs.readFileSync(path, "utf-8");
data = data.replace(
  "\nif (!nativeBinding) {",
  (s) =>
    `
if (!nativeBinding && globalThis.process?.versions?.["webcontainer"]) {
  try {
    nativeBinding = require('./webcontainer-fallback.cjs');
  } catch (err) {
    loadErrors.push(err)
  }
}
` + s,
);
fs.writeFileSync(path, data);

const wasip1BrowserPath = pathJoin(packageDir, "minify.wasip1-browser.js");
const wasip1Browser = fs.readFileSync(wasip1BrowserPath, "utf-8").replaceAll(/[ \t]+$/gmu, "");
fs.writeFileSync(wasip1BrowserPath, wasip1Browser);
