# Oxc React Compiler

oxc integration for the Rust port of the [React Compiler](https://github.com/react/react/tree/main/compiler).

## Overview

This crate runs the React Compiler directly over an oxc AST, memoizing React components and hooks.
The compiler _core_ modules are front-end agnostic (they never depend on oxc) and are vendored under
`src/react_compiler*`; the scope conversion lives here, written against the live oxc AST.

## API

- `transform` — run the compiler and return the compiled oxc `Program` plus diagnostics.
- `run` — standalone pass that rewrites a program in place and returns the scoping the rest of the
  pipeline should use.
- `lint` — report diagnostics only, without emitting code.
- `default_plugin_options` / `PluginOptions` — configure which functions are compiled and how.
