import fs from 'node:fs';

// patch to use async init to workaround sync compile limit 8MB
const filename = './playground.wasi-browser.js';
let data = fs.readFileSync(filename, 'utf-8');
data = data.replace(
  '__emnapiInstantiateNapiModuleSync(__wasmFile',
  'await (await import("@napi-rs/wasm-runtime")).instantiateNapiModule(__wasmFile',
);
fs.writeFileSync(filename, data);
