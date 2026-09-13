import fs from "node:fs";
import { join as pathJoin } from "node:path";

import { disableReusedWorkers } from "../../disable-reused-workers.mjs";

const packageDir = pathJoin(import.meta.dirname, "..");
disableReusedWorkers(pathJoin(packageDir, "src-js/parser.wasi-browser.js"));

const filename = pathJoin(packageDir, "src-js/bindings.js");
let data = fs.readFileSync(filename, "utf-8");

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

data += `const { getBufferOffset, parseRaw, parseRawSync } = nativeBinding
export { getBufferOffset, parseRaw, parseRawSync }
`;

fs.writeFileSync(filename, data);
