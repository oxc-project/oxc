import fs from "node:fs";
import { join as pathJoin } from "node:path";

import { disableReusedWorkers } from "../../disable-reused-workers.mjs";

const packageDir = pathJoin(import.meta.dirname, "..");
disableReusedWorkers(pathJoin(packageDir, "transform.wasi-browser.js"));

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
