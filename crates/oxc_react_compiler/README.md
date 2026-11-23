# Oxc React Compiler

oxc integration for the Rust port of the [React Compiler](https://github.com/react/react/tree/main/compiler).

## Overview

This crate owns the oxc &harr; `react_compiler_ast` (Babel) conversion layer and runs the React
Compiler over an oxc AST, memoizing React components and hooks. The compiler _core_ crates are
front-end agnostic (they never depend on oxc), so they are consumed from crates.io as a published
fork; the AST and scope conversion lives here, written against the live oxc AST.

Ambient TypeScript variable declarations are erased from compiled functions. Their references
remain external values rather than runtime locals, hoisted bindings, or captured context.

## API

- `compile` — run the compiler and return a `CompileResult` with the rewrite to apply
  (via `CompileOutput::transform`) plus diagnostics.
- `lint` — report diagnostics only, without emitting code; each finding is tagged with its
  `ErrorCategory`.
- `PluginOptions` — configure which functions are compiled and how.
