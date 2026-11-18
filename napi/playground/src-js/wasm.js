import { Oxc as OxcBinding } from "../playground.wasi-browser.js";
import { jsonParseAst } from "../../parser/src-js/wrap.js";

export * from "../playground.wasi-browser.js";
export { default } from "../playground.wasi-browser.js";

export function Oxc() {
  const oxc = new OxcBinding();
  return new Proxy(oxc, {
    get(_target, p, _receiver) {
      if (p === "ast") {
        return jsonParseAst(oxc.astJson);
      }
      const value = oxc[p];
      if (typeof value === "function") {
        return value.bind(oxc);
      }
      return value;
    },
  });
}
