# Oxc Module Lexer

This is not a lexer. The name "lexer" is used for easier recognition.

## [es-module-lexer](https://github.com/guybedford/es-module-lexer)

Outputs the list of exports and locations of import specifiers, including dynamic import and import meta handling.

Does not have any [limitations](https://github.com/guybedford/es-module-lexer?tab=readme-ov-file#limitations) mentioned in `es-module-lexer`.

- [ ] get imported variables https://github.com/guybedford/es-module-lexer/issues/163
- [ ] track star exports as imports as well https://github.com/guybedford/es-module-lexer/issues/76
- [ ] TypeScript specific syntax
- [ ] TypeScript `type` import / export keyword

## [cjs-module-lexer](https://github.com/nodejs/cjs-module-lexer)

- [ ] TODO

## Benchmark

This is 2 times slower than `es-module-lexer`, but will be significantly faster when TypeScript is processed.

The difference is around 10ms vs 20ms on a large file (700k).
