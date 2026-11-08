# Oxc Lexer Macros

Procedural macros for Oxc's JavaScript and TypeScript lexer.

- `cpu_multiversion` compiles a function for each supported CPU feature set.
- `cpu_version` defines code for a single CPU feature set.
- `test_cpus` defines a test function that runs for each supported CPU feature set.

These macros are intended for use within `oxc_lexer`.
