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
  - Entry point: `src-js/index.ts` which uses `format()` from `src/main_napi.rs`
  - Build with `pnpm build`

When making changes, consider the impact on all paths.

### Platform considerations

Oxfmt is built for multiple platforms (Linux, macOS, Windows) and architectures.

When working with file paths in CLI code, be aware of Windows path differences:

- Use `std::path::Path` / `PathBuf` instead of manual string manipulation with `/`
- Be cautious with path comparisons and normalization across platforms
  - Avoid hardcoding `/` as a path separator; prefer `Path::join()`
  - Windows uses `\` as a path separator and has drive letter prefixes (e.g., `C:\`)

### Formatter implementations

Oxfmt utilizes different implementations depending on the file extension and filename:

- Tier 1: Rust implementations using `oxc_formatter`, `oxc_formatter_json`, etc found in this repository
- Tier 2: Rust implementations using external libraries like `oxc_toml`
- Tier 3: Delegations to Prettier via NAPI-JS calls (e.g., for Vue or Markdown)
- Tier 4: Delegations to Prettier that require additional Prettier plugins (e.g., for Svelte)

NOTE: Rust written formatters never fall back to Prettier, since they exist to reduce the dependency on Prettier.

#### Embedded language formatting

Embedded languages (e.g. css-in-js, CSS front matter YAML) go through the `FormatDispatcher` (defined in `oxc_formatter_core`) assembled by `src/core/embed/dispatcher.rs`:
a Rust branch per `NativeLanguage` (css/graphql/yaml/json/...), plus the Prettier Doc→IR fallback (`embed/prettier_fallback.rs`, napi only) for the rest.
The pure Rust build runs fallback-less, so non-native embeds (html-in-js, TOML/custom front matter) deliberately stay verbatim.

Three roots install `SessionServices` (each built from `ResolvedDispatchConfig::for_root`):

| Root                                             | Session           | napi build                                           | pure build          |
| ------------------------------------------------ | ----------------- | ---------------------------------------------------- | ------------------- |
| JS/TS file (`core/format.rs`)                    | `PhysicalFile`    | standard profile                                     | native-only profile |
| CSS file (`core/format.rs`)                      | `PhysicalFile`    | fallback-less literal (dispatcher + Tailwind sorter) | same, sorter-less   |
| Vue/Svelte `<script>` (`api/text_to_doc_api.rs`) | `VirtualDocument` | standard profile                                     | -                   |

The service builders are SERVICE PROFILES, not host bindings: `ExternalFormatter::session_services` (napi standard set) and `fence::session_services` (pure native-only set) serve any host with that shape. (a future Markdown host reuses them as-is.)
Whether a Tier 1 host's root takes the Prettier fallback for its embeds (e.g. md-in-graphql before the md Rust port) is a per-host policy decision made explicitly at that root, never a builder default; the CSS root's fallback-less literal in `format.rs` is the current example.
`embeddedLanguageFormatting: off` installs no dispatcher, every builder consults the same off-gate, `ResolvedDispatchConfig::is_embedded_formatting_enabled`.

Per-language options are NOT built up front: `ResolvedDispatchConfig` maps them lazily at dispatch time (`OnceLock`-memoized) from the host file's resolved config, including the Prettier options JSON for the JS-side consumers. `src/core/external_formatter.rs` bridges the napi callbacks into these factories.

A separate string-out channel (the session's `string_embedder` service, NOT the dispatcher) carries the string-in/string-out consumers:

- JSDoc fenced code blocks: routing follows ONE rule
  - a fence language in the `NativeLanguage` registry formats through the dispatcher via a thin string adapter (`embed/fence.rs::format_native_fence`, EVERY build, the pure Rust build wires it via `fence::session_services`)
  - md/html/angular fences stay on the Prettier string path (`string_channel.rs`, napi only; their Doc→IR conversion has unrepresentable cases);
  - everything else stays verbatim
- temporary html-in-js fallback (`format_js_in_html_as_fallback` in `oxc_formatter/src/print/template/embed/html.rs`)

NOTE: These string-out channel is temporary workaround, should be replaced by native implementations and Prettier usage should be eliminated in the future.
JSDoc's string-out is also NOT structural: fences can move to IR-out (session dispatch inside the comment IR) once the printer grows a per-line prefix mechanism for the `*` continuation, but deferred for verification time, not by design.

#### Tailwind CSS class sorting

Tailwind class sorting (`sortTailwindcss`) splits responsibilities:

- Rust collects classes into `FormatElement::TailwindClass`
  - `className` etc. in JS/TS, `@apply` in CSS — incl. css-in-js and JSDoc fenced blocks
- and the JS side sorts them in one batch
  - `sortTailwindClasses` → tailwind's `getClassOrder`, which needs the resolved Tailwind config

Embedded boundaries carry classes through `DispatchResult::tailwind_classes`;
each embed site consumes the doc via `DispatchResult::into_doc(collector)`, which merges them into the parent's class space.

The four data paths (JS/TS top-level / standalone CSS / embedded CSS / JSDoc fenced CSS) are documented at `ExternalFormatter::session_services`.
No CSS goes to Prettier for this; the pure Rust build never collects at all (both mappers gate collection behind napi, since no sorter exists there).

Consequently, managing these various formatter implementations and handling their respective options are also part of Oxfmt's responsibilities.

### CLI implementations

Oxfmt shares code with Oxlint regarding its CLI implementation.

- Rust implementation: `crates/oxc_config`
- JS implementation: `apps/shared`

Please exercise extra caution when making changes to these files.

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
pnpm build-dev && pnpm conformance
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
