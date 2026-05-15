export * from "@oxc-parser/binding-wasm32-wasi";
import * as bindings from "@oxc-parser/binding-wasm32-wasi";
import { wrap } from "./wrap.js";

export { default as visitorKeys } from "./generated/visit/keys.js";

export async function parse(...args) {
  return wrap(await bindings.parse(...args));
}

export function parseSync(filename, sourceText, options) {
  return wrap(bindings.parseSync(filename, sourceText, options));
}
