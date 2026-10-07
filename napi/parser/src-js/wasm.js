export * from "@oxc-parser/binding-wasm32-wasip1";
import * as bindings from "@oxc-parser/binding-wasm32-wasip1";
import { wrap } from "./wrap.js";

export { default as visitorKeys } from "./generated/visit/keys.js";

export async function parse(...args) {
  return wrap(await bindings.parse(...args), args[2]?.attachComments === true);
}

export function parseSync(filename, sourceText, options) {
  return wrap(bindings.parseSync(filename, sourceText, options), options?.attachComments === true);
}
