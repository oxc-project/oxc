import fs from "node:fs";
import { join as pathJoin } from "node:path";

const path = pathJoin(import.meta.dirname, "../index.js");

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
