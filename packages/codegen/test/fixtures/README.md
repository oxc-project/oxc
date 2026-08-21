# Local codegen fixtures

Put a `.js`, `.cjs`, `.mjs`, `.jsx`, `.ts`, `.cts`, `.mts`, or `.tsx` reproduction in this directory
(subdirectories are supported). `fixtures.test.ts` parses each file in unambiguous mode and checks both
the normal and source-map printers against Rust `oxc_codegen`.

Use a hand-built AST test for input shapes a parser cannot produce, such as historical TS-ESLint AST
layouts. See `test/print.test.ts` for that style of test.
