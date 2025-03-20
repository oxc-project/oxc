export * from '@oxc-parser/binding-wasm32-wasi';
import * as bindings from '@oxc-parser/binding-wasm32-wasi';
import { wrap } from './wrap.mjs';

export async function parseAsync(...args) {
  return wrap(await bindings.parseAsync(...args));
}

export function parseSync(filename, sourceText, options) {
  return wrap(bindings.parseSync(filename, sourceText, options));
}
