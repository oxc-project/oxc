# oxc_linter_tsrs

Type-aware lint rules for oxlint, in Rust, in-process.

This crate is a port of [tsgolint](https://github.com/oxc-project/tsgolint) (oxlint's type-aware backend, Go, on
typescript-go) on top of [tsrs](https://github.com/maschwenk/tsrs), the Rust port of the TypeScript 7 compiler. All
60 of tsgolint 7.0.2003's rules are ported and pass tsgolint's own rule test cases (`tests/cases/`, run by
`tests/rule_tests.rs`). It came from tsrslint, a standalone
port (not public yet) that runs as a drop-in `tsgolint` binary; here it is a library.

## How oxlint uses it

`oxc_linter`'s `tsrs` feature links this crate. `oxlint --type-aware` then runs the rules on a thread of the oxlint
process instead of spawning `tsgolint`:

```
                      before                                   after (--features tsrs)
  oxlint ──JSON payload──▶ tsgolint (Go)             oxlint ──Payload value──▶ oxc_linter_tsrs::linter::spawn
         ◀──framed JSON diagnostics──                        ◀──protocol::Output values (channel)──
```

The payload and message types are tsgolint's headless protocol (`protocol.rs`), which `oxc_linter` already speaks,
so the file → tsconfig assignment, the programs, the rules and their options are unchanged; only the process
boundary and the JSON round trip are gone. `oxc_linter::tsgolint` keeps the subprocess path:

- `OXLINT_TSGOLINT_PATH=<binary>` runs that binary as before (stock tsgolint, or a tsrslint build, for comparison).
- `OXLINT_TYPE_AWARE_BACKEND=tsgolint` looks the `tsgolint` binary up in `node_modules` as before.
- Otherwise, with the feature compiled in, the rules run in-process. No download, no `oxlint-tsgolint` package.

```sh
cargo build --release -p oxlint --features allocator,tsrs
./target/release/oxlint --type-aware
```

Knobs from tsrslint still apply: `TSRSLINT_CHECKERS=<n>` (checkers per program; default 6 on machines with ≥ 8
threads, else 4), `TSRSLINT_SCHEDULE=locality`, `TSRSLINT_CHECKER_STATS=1`, `TSRSLINT_MEM_STATS=1`, and
`OXC_LOG=debug` for the run's timeline on stderr.

## Layout

- `src/linter.rs`: a run — payload → programs (one per tsconfig, plus an inferred one) → rules per file on tsrs's
  checkers → `protocol::Output` to a `Sink`. `spawn` is the entry point for a host; `run` is the stdout variant the
  standalone binary used.
- `src/rules/`: one module per rule, a port of `internal/rules/<rule>` in tsgolint.
- `src/rule.rs`, `src/utils/`: the rule context (reporting, fixes, suggestions) and ports of tsgolint's helpers
  (ts-api-utils, typescript-eslint's utils).
- `src/tsconfig.rs`, `src/sched.rs`, `src/overlayfs.rs`: file → tsconfig assignment, file scheduling across
  checkers with tail stealing, and the source-override file system the LSP path uses.
- `tests/rule_tests.rs` + `tests/cases/*.json` + `tests/fixtures/`: tsgolint's rule tests, extracted from its Go
  test files (`cargo test -p oxc_linter_tsrs --release --test rule_tests`; two `no-deprecated` cases that need
  `@types/node` resolvable from `tests/fixtures` are skipped, since this workspace does not install it). `tests/sink_api.rs`: the
  in-process entry point.

## Status and caveats

- tsrs ports TypeScript 7.1.0-dev.20260929; stock tsgolint 7.0.2002 is built on TypeScript 7.0, so a few printed
  types differ in message text (union member order, `typeof import("…")` specifiers). Rules, files and positions
  match on every project compared so far.
- The program for each tsconfig is built per run; the LSP path (`lint_source`) builds one per request, as the
  subprocess did. Keeping a program alive across requests, with tsrs's incremental checking, is the next step the
  in-process design makes possible.
- The lint thread runs with a 512 MiB stack, as the standalone binary did; type checking recurses deeply.
- The crate keeps edition 2021 (tsrs's) for now.
