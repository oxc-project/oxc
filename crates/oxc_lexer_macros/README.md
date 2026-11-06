# Oxc Lexer Macros

Procedural macros for Oxc's JavaScript and TypeScript lexer.

- `cpu_multiversion` compiles a function for each supported CPU feature set.
- `cpu_version` defines code for a single CPU feature set.
- `cpu_impl` makes the methods in an `impl` block unreachable except on a specific architecture.

These macros are intended for use within `oxc_lexer`.
