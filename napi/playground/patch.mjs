import fs from 'node:fs';

// patch to use async init to workaround sync compile limit 8MB
// Waiting for https://github.com/napi-rs/napi-rs/issues/2513
const filename = './playground.wasi-browser.js';
let data = fs.readFileSync(filename, 'utf-8');
data = data.replace(
  '__emnapiInstantiateNapiModuleSync(__wasmFile',
  'await (await import("@napi-rs/wasm-runtime")).instantiateNapiModule(__wasmFile',
).replace(
  `export const Oxc = __napiModule.exports.Oxc`,
  `
import { jsonParseAst } from "../parser/wrap.mjs"

export function Oxc() {
  const oxc = new __napiModule.exports.Oxc();
  return new Proxy(oxc, {
    get(_target, p, _receiver) {
      if (p === 'ast') {
        return jsonParseAst(oxc.astJson);
      }
      if (typeof oxc[p] === 'function') {
        return oxc[p].bind(oxc);
      }
      return Reflect.get(...arguments);
    }
  })
}
`,
);
fs.writeFileSync(filename, data);
