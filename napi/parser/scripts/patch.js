import fs from "node:fs";

const filename = "./src-js/bindings.js";
const webcontainerFallback = `
if (!nativeBinding && globalThis.process?.versions?.["webcontainer"]) {
  try {
    nativeBinding = require('./webcontainer-fallback.cjs');
  } catch (err) {
    loadErrors.push(err)
  }
}
`;
const rawTransferExports = `const { getBufferOffset, parseRaw, parseRawSync } = nativeBinding
export { getBufferOffset, parseRaw, parseRawSync }
`;
let data = fs.readFileSync(filename, "utf-8");

if (!data.includes(webcontainerFallback.trim())) {
  data = data.replace("\nif (!nativeBinding) {", (s) => webcontainerFallback + s);
}

if (!data.includes(rawTransferExports.trim())) {
  data += rawTransferExports;
}

fs.writeFileSync(filename, data);
