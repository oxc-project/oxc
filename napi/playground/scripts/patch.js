import fs from 'node:fs';
import { join as pathJoin } from 'node:path';

const path = pathJoin(import.meta.dirname, '../playground.wasi-browser.js');

let data = fs.readFileSync(path, 'utf-8');
data = data.replace(
  `export const Oxc = __napiModule.exports.Oxc`,
  `
import { jsonParseAst } from "../parser/src-js/wrap.js"

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
fs.writeFileSync(path, data);
