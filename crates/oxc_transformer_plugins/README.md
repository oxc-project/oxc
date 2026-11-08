# Oxc Transformer Plugins

Standalone AST transforms for bundler integrations.

- `InjectGlobalVariables` inserts imports for configured global identifiers or member expressions.
  This can supply shims such as `Buffer` or `process` from caller-selected modules.
- `ReplaceGlobalDefines` replaces configured global identifiers, property chains, and `typeof`
  expressions with constant expressions, as in bundler `define` options. Run the minifier
  afterward to fold constants and eliminate dead branches.
- `ModuleRunnerTransform` rewrites ESM imports and exports for Vite's module runner, using
  helpers such as `__vite_ssr_import__` and `__vite_ssr_exports__`. The host supplies these helpers.

The transforms operate on Oxc ASTs and use semantic scoping information. They can be applied
independently; this crate does not automatically select or load polyfills for a target runtime.
