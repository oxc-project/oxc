# Benchmark

See https://codspeed.io/oxc-project/oxc

## Locally

```sh
# On PR branch
cargo bench -p oxc_benchmark --bench parser -- --save-baseline pr
git checkout main
cargo bench -p oxc_benchmark --bench parser -- --save-baseline main
critcmp # via `cargo binstall critcmp`
```

Focused newline and indentation workloads share their sources between lexer and parser groups:

```sh
cargo bench -p oxc_benchmark --no-default-features --features compiler --bench lexer -- lexer_newlines
cargo bench -p oxc_benchmark --no-default-features --features compiler --bench parser -- parser_newlines
```

These cover LF/CRLF, indentation and blank lines, comments, irregular whitespace,
ASI, EOF tails, and punctuation after newlines. Paired LF/space controls keep the
same source lengths and token sequences where line breaks do not affect syntax.

Source construction, allocator creation and diagnostic preflight happen outside
timing. Each iteration creates the lexer/parser, processes the source and trivia,
and resets the arena. Parser options match the existing parser corpus group.
Criterion filters match substrings; run separate commands for separate groups.
