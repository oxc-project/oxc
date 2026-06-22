# Coding agent guides for `crates/oxc_transformer`

## Testing transformer behavior

Most transformer behavior regressions should be tested through transform conformance fixtures, not Rust integration tests in this crate.

Use:

```text
tasks/transform_conformance/tests/<transform-name>/test/fixtures/<case-name>/
```

Each fixture normally contains:

```text
input.js
output.js
```

Use `input.ts`, `input.tsx`, or `input.jsx` when the regression depends on TypeScript or JSX syntax. The transform options are usually inherited from the nearest `options.json`.

Run a focused fixture with:

```sh
cargo run -p oxc_transform_conformance -- --filter <transform-name>/test/fixtures/<case-name>
```

Run all transform conformance tests with:

```sh
cargo run -p oxc_transform_conformance
```

If the fixture has an `exec.js` runtime assertion, include `--exec`:

```sh
cargo run -p oxc_transform_conformance -- --filter <transform-name>/test/fixtures/<case-name> --exec
```

## OXC-specific regressions

When the case is OXC-specific, put it under that transform's local `oxc` fixture group:

```text
tasks/transform_conformance/tests/<transform-name>/test/fixtures/oxc/<case-name>/
```

For example, optional chaining regressions belong under:

```text
tasks/transform_conformance/tests/babel-plugin-transform-optional-chaining/test/fixtures/oxc/<case-name>/
```

with:

```text
input.js
output.js
```

or `input.ts` if the case depends on TypeScript syntax. The shared optional chaining options live in:

```text
tasks/transform_conformance/tests/babel-plugin-transform-optional-chaining/test/fixtures/oxc/options.json
```

## When to use crate integration tests

Use `crates/oxc_transformer/tests/integrations/` for transformer API behavior, target selection, helper loading, or cross-cutting Rust harness coverage.

Do not add ordinary transform output regressions there when a `tasks/transform_conformance` fixture can cover the behavior.
