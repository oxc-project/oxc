import fs from "node:fs";
import { join as pathJoin } from "node:path";

const packageDir = pathJoin(import.meta.dirname, "..");
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

const browserPath = pathJoin(packageDir, "transform.wasip1-browser.js");
const browser = fs.readFileSync(browserPath, "utf-8").replaceAll(/[ \t]+$/gmu, "");
fs.writeFileSync(browserPath, browser);
