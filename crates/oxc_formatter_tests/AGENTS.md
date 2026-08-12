# Coding agent guides for `crates/oxc_formatter_tests`

Test infrastructure shared by the formatter crates (e.g. `oxc_formatter`, `oxc_formatter_json`, `oxc_formatter_css`, etc).

- `codegen`: build-script helper — consumers call `generate_tests` from `build.rs` via `[build-dependencies]` to emit one `#[test]` per fixture file
- `harness`: fixture runtime — consumers implement `FixtureFormatter` in `tests/fixtures/mod.rs` via `[dev-dependencies]`

## Dependency rule

This crate depends only on `oxc_formatter_core` — NEVER on language crates.
Consumers use it from both `[build-dependencies]` and `[dev-dependencies]`; a dependency on a language crate would make that a build-dep cycle, which cargo rejects.

## insta stays consumer-side

`build_fixture_snapshot` returns the snapshot body without calling `insta::assert_snapshot!`.
Each consumer invokes the macro from its own `tests/fixtures/mod.rs` so the recorded `source:` header points to the consumer crate (required by `INSTA_REQUIRE_FULL_MATCH=1` in CI). Consumers therefore need their own `insta` dev-dep.

## Verification

```sh
cargo c -p oxc_formatter_tests
```

Exercised through the fixture tests of its consumers (`cargo test -p <formatter crate>`).
