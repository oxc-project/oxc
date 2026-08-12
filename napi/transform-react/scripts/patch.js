import fs from "node:fs";
import { join as pathJoin } from "node:path";

const packageDir = pathJoin(import.meta.dirname, "..");
const path = pathJoin(packageDir, "index.js");

let data = fs.readFileSync(path, "utf-8");
data = data.replace(
  "\nif (!nativeBinding) {",
  (source) =>
    `
if (!nativeBinding && globalThis.process?.versions?.["webcontainer"]) {
  try {
    nativeBinding = require('./webcontainer-fallback.cjs');
  } catch (err) {
    loadErrors.push(err)
  }
}
` + source,
);
fs.writeFileSync(path, data);

const workerPath = pathJoin(packageDir, "wasi-worker-browser.mjs");
const worker = fs.readFileSync(workerPath, "utf-8").replaceAll(/[ \t]+$/gmu, "");
fs.writeFileSync(workerPath, worker);

const browserPath = pathJoin(packageDir, "transform-react.wasip1-browser.js");
const browser = fs.readFileSync(browserPath, "utf-8").replaceAll(/[ \t]+$/gmu, "");
fs.writeFileSync(browserPath, browser);
