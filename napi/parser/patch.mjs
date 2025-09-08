import fs from 'node:fs';

const filename = './bindings.mjs';
let data = fs.readFileSync(filename, 'utf-8');

data = data.replace(
  '\nif (!nativeBinding) {',
  (s) =>
    `
if (!nativeBinding && globalThis.process?.versions?.["webcontainer"]) {
  try {
    nativeBinding = require('./webcontainer-fallback.js');
  } catch (err) {
    loadErrors.push(err)
  }
}
` + s,
);

data += `const { getBufferOffset, parseAsyncRaw, parseSyncRaw } = nativeBinding
export { getBufferOffset, parseAsyncRaw, parseSyncRaw }
`;

fs.writeFileSync(filename, data);
