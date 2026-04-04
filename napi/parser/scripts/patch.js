import fs from "node:fs";
import { join as pathJoin } from "node:path";

function patchWasiWorker(path) {
  if (!fs.existsSync(path)) {
    return;
  }

  let data = fs.readFileSync(path, "utf-8");
  data = data.replace(/\nconst errorOutputs = \[\]\n/, "\n");
  fs.writeFileSync(path, data);
}

const filename = "./src-js/bindings.js";
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

patchWasiWorker(pathJoin(import.meta.dirname, "../src-js/wasi-worker-browser.mjs"));
