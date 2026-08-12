# Coding agent guides for `crates/oxc_formatter_tests`

Test infrastructure shared by the formatter crates (e.g. `oxc_formatter`, `oxc_formatter_json`, `oxc_formatter_css`, etc).

- `codegen`: build-script helper
  - Consumers call `generate_tests` from `build.rs` via `[build-dependencies]` to emit one `#[test]` per fixture file
- `harness`: fixture runtime
  - Consumers implement `FixtureFormatter` in `tests/fixtures/mod.rs` via `[dev-dependencies]`
- `suite`: Prettier test-suite provisioning
  - `ensure_prettier_suite()` maintains the pinned suite at `prettier/` (gitignored), no separate clone step anywhere
- `conformance` (feature `conformance`): Prettier-conformance machinery
  - Spec parsing, snapshot matching, report building
  - Consumers add a `tests/conformance.rs` target with a `ConformanceConfig` + format callback and pin the report with `insta`;
  - the crate's own `Cargo.toml` documents why the feature exists
  - Option-set → typed-options mapping is shared per crate via `tests/fixtures/options.rs` (`#[path]`-included by both the fixture harness and the conformance target)

## Prettier suite

Pinned by the `prettier` version in `apps/oxfmt/package.json` (the bundled oracle); provisioning mechanics live in `src/suite.rs`'s rustdocs. Tarball extraction always yields LF content, so Windows needs no autocrlf care. CI only mounts `prettier/` as a cache volume, there is no clone step to keep in sync.

- Bumping Prettier = bump `apps/oxfmt/package.json` + regenerate conformance snapshots (one change set; the suite re-provisions itself on the next run)
- Provisioning failure is a loud failure everywhere it runs (a silent skip would let conformance rot green; needs network + curl/tar)
  - Environments that cannot provision opt out in `ci.yml`, not in crate code: CI's cross jobs (s390x/armv7) pass `-- --skip prettier_conformance`
    - Every suite-provisioning consumer names its conformance test fn `prettier_conformance*`, that name prefix is the contract the skip filter relies on
    - The inverse holds too: tests over COMMITTED fixtures (e.g. `oxc_formatter`'s `jsdoc` fixture-pair test) deliberately avoid the prefix so cross targets still run them
- Requires `curl` and `tar` on PATH (standard on macOS, Linux, and Windows 10+)

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
