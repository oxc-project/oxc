// Re-export the same source-phase import as both default and named exports.
// This should be allowed by the no-named-as-default rule.
import source wasm from './source-phase.wasm';

export { wasm, wasm as default };
