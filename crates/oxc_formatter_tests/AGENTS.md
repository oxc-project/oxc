# Coding agent guides for `crates/oxc_formatter_tests`

Test infrastructure shared by the formatter crates (e.g. `oxc_formatter`, `oxc_formatter_json`, `oxc_formatter_css`, etc).

- `codegen`: build-script helper
  - Consumers call `generate_tests` from `build.rs` via `[build-dependencies]` to emit one `#[test]` per fixture file
- `harness`: fixture runtime
  - Consumers implement `FixtureFormatter` in `tests/fixtures/mod.rs` via `[dev-dependencies]`
  - Asserts `FixtureFormatter::fingerprint` is the same for input and output; consumers take it from the crate's own parse (`parse_for_format`), what it captures is consumer-owned
  - Option-set → typed-options mapping is per crate, in `tests/fixtures/options.rs`

## Dependency rule

This crate NEVER depends on a formatter language crate (`oxc_formatter`, `oxc_formatter_json`, ...).
Consumers use it from both `[build-dependencies]` and `[dev-dependencies]`; a dependency on a language crate would make that a build-dep cycle, which cargo rejects.

## insta stays consumer-side

`build_fixture_snapshot` returns the snapshot body without calling `insta::assert_snapshot!`.
Each consumer invokes the macro from its own `tests/fixtures/mod.rs` so the recorded `source:` header points to the consumer crate (required by `INSTA_REQUIRE_FULL_MATCH=1` in CI). Consumers therefore need their own `insta` dev-dep.

## Verification

```sh
cargo c -p oxc_formatter_tests
```

Exercised through the fixture tests of its consumers (`cargo test -p <formatter crate>`).
