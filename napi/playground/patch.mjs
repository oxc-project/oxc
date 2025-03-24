import fs from 'node:fs';

const filename = './playground.wasi-browser.js';
let data = fs.readFileSync(filename, 'utf-8');
data = data.replace(
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
      const value = oxc[p];
      if (typeof value === 'function') {
        return value.bind(oxc);
      }
      return value;
    }
  })
}
`,
);
fs.writeFileSync(filename, data);
