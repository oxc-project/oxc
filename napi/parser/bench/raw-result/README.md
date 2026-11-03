# Eager raw-result storage experiment

This is a local, AI-assisted proof of concept against `origin/main` at
`62a8e7818e86abe19e86a7125a017f665ba4448c` (parser and JS printer 0.157.0).
It is not ready to merge: private storage changes observable accessor behavior,
and retaining every AST changes the performance tradeoff.

The implementation in `../../src-js/raw-transfer/eager.js` eagerly deserializes
as before, then puts data and optional tokens in private fields. Shared getters
are installed as own properties. `Reflect.construct(RawResult, [data, tokens],
Object)` gives the result `Object.prototype` while installing the private brand.
The constructor's prototype getters are only descriptor templates; the public
result does not inherit from that prototype.

## Throughput results

Five serial paired fresh-process repetitions on 2026-11-03, Apple M4 Pro,
Node 26.13.0 / V8 14.6.202.34-node.40, matching local release bindings built with
`allocator,tokens`, production local JS printer. These are warmed corpus
measurements of the eager raw entry point, not full-build or cold-start results.
The public sync/async routing is covered by NAPI and contract tests.

| Workload                                  | Main median (range), seconds | POC median (range), seconds | Median change   | Median max RSS MiB, main → POC |
| ----------------------------------------- | ---------------------------- | --------------------------- | --------------- | ------------------------------ |
| Discard results after reading body/errors | 1.441 (1.424–1.462)          | 1.036 (1.010–1.047)         | 28.1% less time | 1132 → 507                     |
| Parse + complete JS print + maps          | 2.966 (2.938–3.015)          | 2.498 (2.445–2.582)         | 15.8% less time | 1155 → 559                     |
| Retain every result                       | 2.020 (1.909–2.272)          | 2.169 (2.098–2.345)         | 7.4% more time  | 4312 → 4269                    |

All five raw and mapped pairs favored the POC (27.3–29.2% and 13.0–17.1%
reductions respectively). Retention had three slower and two faster POC pairs;
it demonstrates no consistent benefit. Post-GC retained heaps were 3939 → 3911
MiB, including preloaded sources. The short-lived result gain should not be
extrapolated to retained AST workloads.

An earlier Node 26.12.1 confirmation was interrupted after severe concurrent
Rust/Node activity inflated samples. Its partial files remain under
`target/raw-result/final/` and are excluded from this table. Earlier generated
private-wrapper samples are under `confirm/`; final samples use the actual edited
`eager.js`. The interactive host's background apps/indexers were not disabled;
final timing ranges and all individual samples are retained in `results.json`.

## Compatibility

The POC preserves ordinary reads, own key order (`program`, `module`, `comments`,
optional `tokens`, `errors`), own accessor descriptors, enumerable/configurable
flags, absence of setters, getter names/arity, read-only assignment, spreading,
structured cloning, sealing/freezing, property deletion/redefinition, and the
Object prototype. Private storage adds no visible own symbols.

It deliberately changes these observable behaviors:

| Operation                                      | Main                  | POC         |
| ---------------------------------------------- | --------------------- | ----------- |
| Getter identity across results                 | Unique                | Shared      |
| Detached getter called with `{}` / `undefined` | Original result field | `TypeError` |
| Getter from result A called on result B        | A's field             | B's field   |
| Ordinary `new Proxy(result, {}).program`       | Works                 | `TypeError` |
| Read through `Object.create(result)`           | Works                 | `TypeError` |

A proxy can supply the branded target as the receiver using
`Reflect.get(target, property, target)`. That workaround does not make this
representation fully compatible. Private data also remains attached to a live
result after every own accessor is deleted or redefined. In three fresh
controls, keeping 128 emptied results alive allowed main's payloads to be
reclaimed by major GC, while all 128 private payloads stayed live until the
owners were discarded. Visible property deletion still succeeds; its storage
lifetime differs. There is no proposal here to silently change the stable JSON
result contract.

`contract.mjs` compares all fields to main's wrapper using the **same local native
binary and generated deserializers**. It covers 64 combinations of JS/TS/JSX/TSX,
ranges, parents and tokens, plus async cases. It also records baseline JSON/raw
differences: raw metadata gets additional `range` fields, and a source string
containing an actual lone surrogate has a different literal representation.
Those differences exist before this POC. Parent deserializers are present in
this checkout and local tests pass. Main's package file whitelist still omits
those parent JS files: `npm pack --dry-run --ignore-scripts --json` confirmed
that only the four non-parent deserializers ship. This preexisting packaging
failure is not fixed by the POC and is distinct from local parent correctness.

## Why closures promote discarded ASTs

`lifetime.mjs` separates allocation from observation in different event-loop
turns so WeakRef's keep-alive rule does not retain the test payloads. It creates
128 young payload graphs, keeps only WeakRefs, requests exactly two minor GCs,
then requests a major GC in a subsequent turn. Trace parsing verifies the
collector sequence. `%InYoungGeneration` is only used in this diagnostic;
throughput samples contain no intrinsics or profiling.

Node 26.12.1's V8 source allocates new accessor pairs in old space in
[`FactoryBase::NewAccessorPair`](https://github.com/nodejs/node/blob/v26.12.1/deps/v8/src/heap/factory-base.cc#L67).
[`LookupIterator::TransitionToAccessorProperty`](https://github.com/nodejs/node/blob/v26.12.1/deps/v8/src/objects/lookup.cc#L808)
stores the getter in an accessor pair (or copies an existing pair).
The fast-map path also creates accessor pairs in
[`Map::TransitionToAccessorProperty`](https://github.com/nodejs/node/blob/v26.12.1/deps/v8/src/objects/map.cc#L2235).
[`Scavenger::ScavengePage`](https://github.com/nodejs/node/blob/v26.12.1/deps/v8/src/heap/scavenger.cc#L2444)
visits old-to-new remembered slots.
The inferred causal chain is an old accessor pair retaining a per-result
getter, its context/bound arguments, and the AST. Minor GC scans references from
old space without establishing that the old owner is still globally reachable.
A dead wrapper can therefore leave its payload alive through minor collections
until major GC reclaims the accessor pair. Shared getters have no reference to
per-result data, and a dead private-field owner can die in young space.

This explanation combines matched engine source, repeated controlled collection
experiments and whole-corpus GC traces. It does not rely on fast-property probes
or infer allocation generation from a heap snapshot taken after major GC.
The exact fraction of the corpus promotion caused by each internal reference is
not instrumented in a custom V8 build.

All three fresh-process repetitions produced the same lifetime result:

| Representation                                                    | Payloads after two minor GCs | Live wrappers | Payloads after major GC |
| ----------------------------------------------------------------- | ---------------------------: | ------------: | ----------------------: |
| Closure / direct field capture / explicit accessor / bound getter |                    128 / 128 |             0 |                       0 |
| Private storage / plain data                                      |                      0 / 128 |             0 |                       0 |

The shared getter itself remains live in the private-storage control; it carries
no per-result payload. Every payload was verified young before collection, and
surviving closure payloads were old after the two minor collections.

Separate Node 26.13.0 corpus GC traces (same V8 14.6.202.34-node.40 as 26.12.1):

| Flags                          | Main promoted MB | POC promoted MB | Main minor/major GCs | POC minor/major GCs |
| ------------------------------ | ---------------: | --------------: | -------------------: | ------------------: |
| Default                        |           1514.7 |            36.6 |               98 / 6 |             194 / 0 |
| No allocation-site pretenuring |           3811.5 |            45.8 |              234 / 6 |             196 / 1 |
| No Maglev                      |           1339.4 |            43.7 |               88 / 6 |             197 / 1 |

Default trace GC pauses totaled 419.5ms versus 53.3ms; GC-event allocation deltas
were 4.056GB versus 4.040GB. These are instrumented observations, excluding
native allocations and the uncollected tail, not unprofiled throughput results.
Disabling adaptive pretenuring increases counted promotion in main; its
allocation strategy should not be confused with the unconditional old-space
allocation of accessor pairs. Neither control removes the representation gap.

A separate allocation-sampling run, including objects collected by both minor
and major GC, estimated 4,064,048,456 JS bytes for main and 4,053,931,600 bytes
for the POC (3.785 versus 3.776 GiB, about 0.25% less). This independently
supports lifetime/promotion as the material difference. Sampling is approximate,
excludes native allocation, and its elapsed times are not benchmark samples.

Moving closures into a helper, capturing fields separately, defining accessor
properties explicitly and binding getters were screened as alternatives. None
established a comparable improvement while preserving receiver behavior. The
bound/explicit-definition screens also change getter names. WeakMap and printer
experiments from the earlier investigation were not repeated without new evidence.

## Reproduction

Run commands from the repository root. Set `CORPUS` to the read-only reproduction
root containing `src/`. No script writes there or imports its installed parser
or printer. Node 22+ and a raw-transfer-supported platform are required. The
V8 intrinsics and GC trace parser are diagnostic tools for the recorded Node
26 engine; adapt them when investigating another engine version.

```sh
CORPUS=/Users/cameron/gitlab/cclark/codex-webview-vite-oxc-repro
pnpm install --frozen-lockfile
just ast
pnpm -C napi/parser run build-dev --release --features allocator,tokens
pnpm -C packages/codegen build
node napi/parser/bench/raw-result/prepare.mjs
node napi/parser/bench/raw-result/contract.mjs
node napi/parser/bench/raw-result/run.mjs "$CORPUS" baseline,poc raw,maps,retain 3 paired
node napi/parser/bench/raw-result/run.mjs "$CORPUS" baseline,poc verify 1 correctness
node napi/parser/bench/raw-result/diagnose.mjs "$CORPUS"
node napi/parser/bench/raw-result/summarize.mjs paired
node --expose-gc --allow-natives-syntax napi/parser/bench/raw-result/lifetime.mjs closure delete
node --expose-gc --allow-natives-syntax napi/parser/bench/raw-result/lifetime.mjs private delete
```

The runner launches serial fresh processes and rotates variant order. Sources
are preloaded and hashed; two evenly spaced warmup passes precede forced GC.
There is no forced GC inside the measured pass. Natural GC is included.
`raw` consumes AST body/error counts; `maps` prints the complete AST with source
maps; `retain` holds **every result** until after the pass and reads them again
outside timing. A final forced GC and memory report are outside timing.
The worker exits after synchronously saving results, excluding the buffer
cache's idle timer from process wall time.

`verify` compares entire ASTs, comments, diagnostics and module metadata with
JSON transfer, then exact code and source maps, ordered imports, output reparse
and printer fixed points. Its wall time is not a throughput sample.

`OXC_BENCH_LIMIT=256` selects an evenly spaced smoke corpus. For separate
instrumentation use `OXC_BENCH_PROFILE=cpu` or `heap` with `run.mjs` and one run.
`diagnose.mjs` runs repeated lifetime probes, then corpus GC traces with default
flags, allocation-site pretenuring disabled, and Maglev disabled. Engine flags
are diagnostic controls, not production recommendations.

Experiments and raw artifacts live under ignored `target/raw-result/`. Each
paired run saves a build manifest with native/generated-JS/printer hashes.
`prepare.mjs` extracts main's wrapper from the pinned base, redirects all imports
to this worktree, and asserts its patch boundaries. Set `OXC_BENCH_BASE` only
when intentionally adapting the baseline to another revision.

Required focused checks:

```sh
pnpm -C napi/parser run build-test
RUN_RAW_TESTS=1 RUN_RAW_RANGE_TESTS=1 RUN_RAW_TOKENS_TESTS=1 pnpm -C napi/parser test
cargo coverage parser
just fmt
pnpm exec oxlint -c oxlintrc.json --deny-warnings --report-unused-disable-directives \
  napi/parser/src-js/raw-transfer/eager.js napi/parser/test/raw-result.test.ts \
  napi/parser/bench/raw-result
```

The test build replaces the release binding. Rebuild the release binding before
benchmarking; do not mix benchmark samples from different profiles.

## Verification results

- `just ast`: regenerated current-main JS/Rust layouts with no tracked changes.
- Matching parser release build with `allocator,tokens` and production JS printer build completed.
- `pnpm -C napi/parser run build-test` completed; all **178,470** NAPI tests passed
  with raw, range/parent and token suites enabled, including 13 new contract tests.
- `cargo coverage parser` completed against the pinned Test262/Babel/TypeScript/misc
  suites, with no snapshot changes. Existing expected parser failures remain.
- Full corpus checks for both wrappers passed: 33,579,124 AST nodes and 214,521
  ordered imports; generated output was 217,840,905 bytes with SHA-256
  `c56c4b008d33671dc659dc7f48e677d98d96cdb789173e36acc8934e56818cc5`.
  Exact source maps, output reparse and printer fixed points matched.
- `just fmt`, focused Oxlint, and `git diff --check` passed.
- Browser tests were not run: eager raw transfer is Node-specific; no WASM or JSON
  implementation changed. No Rust implementation changed, so Rust lint and
  allocation snapshots were not added to these binding/conformance checks.

## Scope and next steps

The external corpus has 32,267 files, 197,999,377 UTF-8 bytes, and SHA-256
`599ac3c424add45883bb9a894ae5ba7c631d9f03b6a887ddf2245e037608de12`.
It is predominantly synthetic ASCII/numeric-array source, with one non-ASCII
source file. Targeted Unicode cases establish behavior, not Unicode-heavy
throughput. One machine and closely related Node/V8 versions do not establish a
general parser ranking.

Before considering an upstream change, agree on an explicit experimental result
contract (including detached getters and proxies), test realistic bounded AST
retention, and repeat on supported Node/V8 versions and a diverse corpus.
A V8-side experiment allocating per-result accessor pairs in young space would
further test the mechanism while preserving receiver-independent closures.
No Rust grammar hot-function ranking is inferred from the stripped npm binary.

The earlier repro already uses raw transfer with JSON fallback in project-owned
transforms; FormatJS retains its own JSON parser. Its noisy full-build medians
(JSON 268.7s, raw 294.5s, with indexing contention) did not establish a full-build
speedup. Corpus results here cannot establish one. Earlier printer token-array
and flush-threshold experiments did not justify production changes.
