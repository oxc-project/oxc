# Coding agent guides for `apps/oxfmt`

Oxfmt is the integration layer that dispatches between the Rust formatters built on `oxc_formatter_core` and Prettier.

## Overview

The `oxfmt` implemented under this directory serves several purposes.

- JS/Rust hybrid CLI using `napi-rs`
  - Full feature set like CLI, Stdin, LSP, and more
  - Format many languages with embedded language formatting support like Prettier
  - Entry point: `src-js/cli.ts` which uses `run_cli()` from `src/main_napi.rs`
  - Build with `pnpm build`
- Pure Rust CLI
  - Limited feature set, CLI usage only, no LSP, no Stdin support
  - Formats only languages which we rewrite formatter in Rust
  - Entry point: `main()` in `src/main.rs`
  - Build with `cargo build --no-default-features`
- Node.js API using napi-rs
  - Caller-supplied options, no config discovery
  - Entry point: `src-js/index.ts` which uses `format()` from `src/main_napi.rs`
  - Build with `pnpm build`

When making changes, consider the impact on all paths.

Cross-cutting behavior is applied through several entry points:

- CLI: `src/cli/walk_runner.rs` and `src/cli/walk.rs`
- Stdin: `src/cli/stdin_runner.rs`
- LSP: `src/lsp/server_formatter.rs`
- NAPI direct-document API: `src/api/format_api.rs`
- NAPI `textToDoc()` API for `prettier-plugin-oxfmt`: `src/api/text_to_doc_api.rs`

Check the relevant path before assuming that behavior is shared across entry points.

### Platform considerations

Oxfmt is built for multiple platforms (Linux, macOS, Windows) and architectures.

When working with file paths in CLI code, be aware of Windows path differences:

- Use `std::path::Path` / `PathBuf` instead of manual string manipulation with `/`
- Be cautious with path comparisons and normalization across platforms
  - Avoid hardcoding `/` as a path separator; prefer `Path::join()`
  - Windows uses `\` as a path separator and has drive letter prefixes (e.g., `C:\`)

## CLI implementations

Oxfmt shares code with Oxlint regarding its CLI implementation.

- Rust implementation: `crates/oxc_config`
- JS implementation: `apps/shared`

Please exercise extra caution when making changes to these files.

### Ignore architecture

Ignore handling is intentionally split across entry points.
Use the entry-point map above; keep detailed behavior and rationale next to the relevant implementation and tests.

The stable distinction is:

- Formatter-owned ignores control formatting eligibility where the entry point supports them
  - These include `.prettierignore`, CLI `--ignore-path`, `!` patterns and config `ignorePatterns`
  - They exclude an explicitly requested document
- Git-derived ignores scope filesystem discovery
  - `.gitignore` and `.git/info/exclude`
  - They do not exclude an explicitly requested document

## Formatter implementations

Oxfmt utilizes different implementations depending on the file extension and filename:

- Tier 1: Rust implementations using `oxc_formatter`, `oxc_formatter_json`, etc found in this repository
- Tier 2: Rust implementations using external libraries like `oxc_toml`
- Tier 3: Delegations to Prettier via NAPI-JS calls (e.g., for Vue or Markdown)
- Tier 4: Delegations to Prettier that require additional Prettier plugins (e.g., for Svelte)

NOTE: Rust written formatters never fall back to Prettier, since they exist to reduce the dependency on Prettier.

### Embedded language formatting

Embedded languages (e.g. css-in-js, CSS front matter YAML) go through the `FormatDispatcher` (defined in `oxc_formatter_core`) assembled by `src/core/embed/dispatcher.rs`.
Routing is ONE table (`dispatcher::route`): `Native` languages (css/graphql/yaml/json/...) get a Rust branch, the `Prettier` set (html/angular/markdown) goes to the Prettier Doc→IR channel (`embed/prettier_doc.rs`, napi only), everything else is deliberately preserved.

Vocabulary: "fallback" = the dispatcher's optional `PrettierDocFallback` slot (a build/root may not install one).
The pure Rust build runs fallback-less, so non-native embeds (html-in-js, TOML/custom front matter) deliberately stay verbatim.

Three roots install `SessionServices`, all via `embed/services.rs::for_root` (one definition per build; the napi one takes the `ExternalServices` transport, and adds the Prettier fallback / string embedder / Tailwind sorter to the registry dispatcher): the JS/TS and CSS file roots (`core/format.rs`, `PhysicalFile` sessions, both builds) and the Vue/Svelte `<script>` root (`api/text_to_doc_api.rs`, `VirtualDocument` session, napi only).

Which languages may dispatch AT ALL from a given host is the host crate's own gate (e.g. `oxc_formatter_css` dispatches only `yaml`/`toml` front matter); the shared `route()` table then decides who serves the language. Adding a dispatch call to a host crate is therefore a routing decision (check `route()` and the embedded conformance when doing so. A root needing a bespoke service set would assemble the `SessionServices` struct literally), none does today.
`embeddedLanguageFormatting: off` installs no dispatcher, every builder consults the same off-gate, `ResolvedDispatchConfig::is_embedded_formatting_enabled`.
Tracing span namespaces: `oxfmt::embed::` = pure Rust work, `oxfmt::external::` = napi-crossing calls.

Per-language options are NOT built up front: `ResolvedDispatchConfig` maps them lazily at dispatch time (`OnceLock`-memoized) from the host file's resolved config, including the Prettier options JSON for the JS-side consumers. `src/core/external_services.rs` bridges the napi callbacks into these factories.

A separate string-out channel (the session's `string_embedder` service, NOT the dispatcher) carries JSDoc's string-in/string-out consumer:

- JSDoc fenced code blocks: routing follows ONE rule, the same `dispatcher::route` table
  - a `Native` fence language formats through `FormatSession::dispatch_to_string` via a thin string adapter (`embed/jsdoc_fence.rs::format_native_fence`, EVERY build, the pure Rust build wires it via `services::for_root`)
  - md/html/angular fences stay on the Prettier string path (`embed/prettier_string.rs`, napi only; their Doc→IR conversion has unrepresentable cases);
  - everything else stays verbatim
  - the embedder carries the caller's effective print width; both branches honor it (native via `PrintWidth` override, Prettier via `printWidth` in the options JSON), so a fence prints at the same width a JS/TS snippet in the same position would (see `upstream-jsdoc-bugs.md` #11 for the deliberate divergence from upstream's flat `printWidth - 4`)

NOTE: The string-out channel outlives the md/html/angular rewrites; its full exit criterion is owned by `oxc_formatter_core`'s AGENTS.md (domain (4)).
The half owned here: JSDoc's string-out is NOT structural, fences can move to IR-out (session dispatch inside the comment IR) once the printer grows a per-line prefix mechanism for the `*` continuation, deferred for verification time, not by design.

### Tailwind CSS class sorting

Tailwind class sorting (`sortTailwindcss`) splits responsibilities:

- Rust collects classes into `FormatElement::TailwindClass`
  - `className` etc. in JS/TS, `@apply` in CSS — incl. css-in-js and JSDoc fenced blocks
- and the JS side sorts them in one batch
  - `sortTailwindClasses` → tailwind's `getClassOrder`, which needs the resolved Tailwind config

Embedded boundaries carry classes through `DispatchPayload::tailwind_classes`;
each embed site consumes the doc via `DispatchPayload::into_doc(collector)`, which merges them into the parent's class space.

The four data paths (JS/TS top-level / standalone CSS / embedded CSS / JSDoc fenced CSS) are documented at `embed::services::for_root` (napi definition).
No CSS goes to Prettier for this; the pure Rust build never collects at all (both mappers gate collection behind napi, since no sorter exists there).

Consequently, managing these various formatter implementations and handling their respective options are also part of Oxfmt's responsibilities.

## Verification

```sh
cargo c
cargo c --no-default-features
cargo c --features detect_code_removal
```

Also run `clippy` for the same configurations and resolve all warnings.

Run tests with:

```sh
# Run unit test in Rust
cargo t
# Run E2E test
pnpm build-dev && pnpm t
# Update snapshots
pnpm t -u

# Run conformance test for xxx-in-js and js-in-xxx
pnpm build-dev && pnpm download-fixtures && pnpm conformance
```

To manually verify the CLI behavior after building:

```sh
pnpm build-test

# Show help
node ./dist/cli.js --help
# Stdin (Prettier's `--config=<cfg> <file>` equivalent)
cat <file> | node ./dist/cli.js --config=<cfg> --stdin-filepath=<file>
# With log
OXC_LOG=debug node ./dist/cli.js --threads=1 <file>
```

NOTE: `pnpm build-dev` combines `pnpm build-js` and `pnpm build-napi`, so you don't need to run them separately.

To compare formatting output with Prettier:

```sh
# Use a shared config file (e.g., fmt.json) because Oxfmt and Prettier have different default printWidth.
# Example fmt.json: { "printWidth": 80 }
cat <file> | node ./dist/cli.js --config=fmt.json --stdin-filepath=<file>
node ./node_modules/prettier/bin/prettier.cjs --config=fmt.json <file>
```

## Test organization (`test/` directory)

Tests are organized into specific domains, each with its own structure.

### `test/api/`: Formatting result tests

Focuses on verifying formatting output. Use the Node.js API. No fixtures, test inputs are inline in each test file.

- Multiple `*.test.ts` files coexist in a flat directory (no subdirectories)
- Snapshots are colocated in `__snapshots__/` by Vitest

### `test/cli/`: CLI fixture-driven tests

A single `cli.test.ts` auto-discovers and runs all fixture directories via `utils.ts`.

- Each fixture directory contains:
  - `options.json` — array of test cases (args, cwd, env, stdin, etc.)
  - `fixtures/` — input files for the test cases
  - `*.snap.md` — file snapshots (one per test case, named `0.snap.md`, `1.snap.md`, …)
- Adding a new CLI test: create a new directory with `options.json` and `fixtures/`, then run the test to generate snapshots
  - `fixtures/` represents a single project structure
  - Multiple test cases (different args, cwd, etc.) against the same structure go in one `options.json`
  - If a scenario needs a different project layout, create a separate directory
  - Name related directories with a shared prefix (e.g., `nested_config/`, `nested_config_no_root/`)
- If exceptional test cases are required, place a separate `*.test.ts` file for them

### `test/lsp/`: LSP integration tests

Each test directory follows the 1:1:1 rule:

- 1 test file (`*.test.ts` with the same name as the directory)
- 0 or 1 `fixtures/` directory
- Snapshots are colocated in `__snapshots__/` by Vitest

Shared helpers are in `utils.ts` at the `test/lsp/` level.

## After updating `Oxfmtrc` (`src/core/oxfmtrc.rs`)

When modifying the `Oxfmtrc` struct (and configuration options):

- Run `just formatter-schema-json` to update `npm/oxfmt/configuration_schema.json`
- Run `just formatter-config-ts` to regenerate `src-js/config.generated.ts` from the schema
- Run `cargo test -p website_formatter` to update schema markdown snapshots
  - Then, `cargo insta accept`
