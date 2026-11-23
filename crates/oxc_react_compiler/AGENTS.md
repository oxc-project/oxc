# Coding agent guides for `crates/oxc_react_compiler`

These rules also apply to the React Compiler lint rules in `crates/oxc_linter` and their Oxlint fixtures.

## Overview

Port of the upstream React Compiler on [`main`](https://github.com/react/react/tree/main/compiler): the Rust port in [`compiler/crates`](https://github.com/react/react/tree/main/compiler/crates) and the TypeScript [`babel-plugin-react-compiler`](https://github.com/react/react/tree/main/compiler/packages/babel-plugin-react-compiler) it is ported from.
The modules under `src/react_compiler_*` mirror the upstream `react_compiler_*` crates and TypeScript passes.

- `compile` is used by the transformer and `napi/transform-react`
- `lint` is used by the Oxlint React Compiler rules through `crates/oxc_linter/src/utils/react_compiler.rs`

## Upstream alignment

- Treat upstream `main` as the spec: the Rust port in `compiler/crates` first, then the TypeScript it mirrors; keep pass structure, names, error messages, and error categories aligned
  - Do not pin behavior to a published npm release such as `babel-plugin-react-compiler@1.0.0`; when a release and `main` differ, follow `main`
  - Compare against an up-to-date checkout of `main`, and cite the upstream file and commit when porting a change
- Diverge only when upstream emits incorrect output, crashes or hangs, reports a lint false positive, or for an Oxlint-only extension
  - Start the code comment with "Unlike the TS implementation," and explain why
  - Add a fixture that covers the divergence
- Compare changes with upstream `main`, and lint behavior with `eslint-plugin-react-hooks` built from `main`, using the same environment options
  - Classify each difference as an intentional fix, an Oxlint-only extension, or behavior already on `main`

## Tests

Add fixtures only; do not add Rust unit tests or inline linter `Tester` cases.

| Behavior                                       | Location                                                                         |
| ---------------------------------------------- | -------------------------------------------------------------------------------- |
| Compiler output and diagnostics                | `fixtures/`, snapshotted to `tests/snapshots/` (subdirectories joined with `__`) |
| Oxlint integration (rule options, file gating) | `apps/oxlint/test/fixtures/react_<topic>/` with `.oxlintrc.json` and `files/`    |
| `transform-react` options                      | `napi/transform-react/test/fixtures/<name>/`                                     |

- Put pragmas on the first line as `// @key:value`, with JSON-quoted strings such as `@compilationMode:"annotation"`, so upstream's `parseConfigPragmaForTests` reads them too
  - The snapshot harness defaults to `compilationMode: all` and `panicThreshold: all_errors`
  - Put explanatory comments after the pragma line
- Keep upstream fixture names and paths
  - Prefix fixtures that are expected to fail with `error.invalid-`, `error.todo-`, or `error.bug-`
  - When a change makes such a fixture compile, rename it without the prefix and remove the old snapshot
- Use `@outputMode:"lint"` for lint-only behavior; the snapshot records `Lint-mode diagnostics (differ from transform)` when `lint` and `compile` disagree
- Snapshots are Oxc's golden output; never edit them by hand, and review them with the `insta-snapshots` skill
- Executable fixtures must export a usable `FIXTURE_ENTRYPOINT` with real inputs for `just sprout`, which compares original and compiled runtime behavior; missing entrypoints and `TodoAdd` placeholders fail CI
- Cover unchanged and changed inputs with `sequentialRenders` when a fixture exercises memoization across renders
- Keep snapshot and Sprout pragma configuration together in `tests/common/mod.rs`; document concrete snapshot-only runtime limitations in `tasks/sprout/src/exclusions.ts`

## Lint mode and Oxlint

- `crates/oxc_linter/src/utils/react_compiler.rs` mirrors the `COMPILER_OPTIONS` of `eslint-plugin-react-hooks`; document every Oxlint-specific option inline
  - Lint-specific behavior comes from those options, not from overrides of compiler logic; for example, ESLint suppressions are ignored because the options enable both `validateHooksUsage` and `validateExhaustiveMemoizationDependencies`
- The file gate ports `mayContainReactCode` from `eslint-plugin-react-hooks`
- Enable `lint_skip_reactive_scope_passes` only when both `react/todo` and `react/invariant` are off
  - Never skip a pass that records Todo or Invariant diagnostics while either rule is enabled
- Code the compiler cannot compile, such as `try`/`finally`, may still be lowered for lint analysis, as long as the Todo is still recorded
- Keep upstream messages, categories, and rule mapping; report each diagnostic once per location
- Mark Oxlint-only rule options as such, and regenerate the schema with `just linter-schema-json`
- When changing which files React Compiler rules run on, run `cargo lint-timings`

## Performance

The compiler runs on every component or hook file in Oxlint and the transformer.

- Avoid recursion whose depth scales with input size, such as CFG or member-chain depth; use explicit worklists
- Avoid exponential algorithms
- Bail out with a Todo on pathological input
- Add cheap early returns before expensive validation passes
- Benchmark hot-path changes:
  - `cargo bench -p oxc_benchmark --bench react_compiler --features react_compiler`
  - `cargo bench -p oxc_benchmark --bench linter`

## Verification

- Always: `cargo test -p oxc_react_compiler` and `just lint -p oxc_react_compiler`
- For compiler behavior changes: `just sprout` (use `just sprout --filter <path>` while debugging)
- If Oxlint is affected: `cargo test -p oxc_linter`, then `pnpm --dir apps/oxlint build-test && pnpm --dir apps/oxlint test`
- If `transform-react` is affected: `pnpm --dir napi/transform-react build-test && pnpm --dir napi/transform-react test`
