// Re-export a source-phase import as default and the module's default export as
// a named export. This should not be allowed by the no-named-as-default rule.
import source wasm from './source-phase.wasm';

export { wasm as default };
export { default as wasm } from './source-phase.wasm';
