# Oxc Lexer

A standalone lexer for JavaScript, TypeScript, and JSX, under development.

- SIMD implementations for x86_64 AVX2 and aarch64 Neon, with a generic implementation for other CPUs.
  CPU feature selection is provided by `oxc_simd`.
- Optional UTF-8 and regular expression validation through `LexOptions`.
- Optional comment and whitespace tokens, line tables, and diagnostic limits.
- Input buffers require `PAD` zero bytes after the source text; see `lex_utf8` and `lex_utf8_arena`.
- Currently available only on little-endian targets.

The parser still uses its own lexer in `oxc_parser`; this crate is not its default lexer.
