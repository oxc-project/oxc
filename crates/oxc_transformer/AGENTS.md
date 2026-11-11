# Coding agent guides for `crates/oxc_transformer`

## Supported output target

Oxc Transformer lowers modern JavaScript to ES2015 (ES6) syntax. It does not
generate complete ES5 output, and `es5` is rejected as a target.

A target for an older runtime may still enable the transforms Oxc implements,
but the result is not guaranteed to run in a pre-ES2015 environment. Before
treating target-dependent behavior as a regression, verify that public target
selection can produce the transform combination. Conformance fixtures that
enable plugins directly may create unsupported combinations.

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

## Pull request descriptions

A behavior fix should let a reviewer see the failure without reconstructing it. Lead with the problem in prose, then include an example in this shape:

### Example

Input:

```js
// the smallest code that reproduces the bug
```

Before output:

```js
// output from the transformer example at the parent commit
```

After output:

```js
// output from the transformer example at the change
```

Explain the observable difference after the outputs, such as a runtime value or a thrown error, when the output alone does not show it.

Capture both outputs with the transformer example rather than writing them by hand. Save the input outside the repository and run the example while the bug still reproduces:

```sh
cargo run -p oxc_transformer --example transformer -- /tmp/repro.ts --target es2015
```

Use its `Transformed:` block as the before output, apply the change, run the same command again, and use the new block as the after output. If you no longer have the original checkout, run it in a detached worktree of the parent commit (`git worktree add --detach /tmp/oxc-parent <parent-commit>`); that checkout builds in its own `target/` directory.

State the options and target used for the example when the result depends on them. When the case reproduces on the [playground](https://playground.oxc.rs/), include a link containing the example code and options; the playground runs the released version. The repository's `oxc-playground-url` skill builds the share link.

When the printed output looks correct but behaves differently, add the case as a conformance fixture with an `exec.js` assertion and run it with `--exec`. Report the runtime value or thrown error, and state the target or the fixture's `options.json` when the result depends on them.

For a refactor, say what must stay identical and how it was compared, such as byte-identical conformance reports and exec results between the parent commit and the change.
