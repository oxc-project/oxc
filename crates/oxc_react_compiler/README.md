# Oxc React Compiler

oxc integration for the Rust port of the [React Compiler](https://github.com/facebook/react/pull/36173).

## Overview

This crate owns the oxc &harr; `react_compiler_ast` (Babel) conversion layer and runs the React
Compiler over an oxc AST, memoizing React components and hooks. The compiler _core_ crates are
front-end agnostic (they never depend on oxc), so they are consumed from crates.io as a published
fork; the AST and scope conversion lives here, written against the live oxc AST.

## API

- `transform` — run the compiler and return the compiled oxc `Program` plus diagnostics.
- `run` — standalone pass that rewrites a program in place and returns the scoping the rest of the
  pipeline should use.
- `lint` — report diagnostics only, without emitting code.
- `default_plugin_options` / `PluginOptions` — configure which functions are compiled and how.
