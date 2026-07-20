# Task: Rewrite property-name mangling on current main as a local review stack

## Stages

### Stage 1: Core property mangler and minifier integration
- **Goal**: Add a three-phase property mangling engine (`collect`, deterministic global `assign`, `rewrite`) with independent minifier options, cache semantics, occurrence-frequency ordering, esbuild-style per-occurrence quoted handling, exact reservations, annotations, and a pre-compression single rewrite.
- **Depends on**: none
- **Parallel**: no
- **Success criteria**: Focused property tests cover supported AST positions, mixed quoted/unquoted occurrences, annotations, cache behavior (including duplicate targets and `false`), frequency ordering, determinism, shorthand expansion, and compression interaction; `cargo test -p oxc_minifier` and crate clippy pass.
- **Status**: Not Started

### Stage 2: NAPI surface, result cache, docs, and benchmarks
- **Goal**: Expose top-level `mangleProps` without coupling it to identifier mangling, validate regex/cache input, return the merged cache, document unsafe semantics and regex dialect, and add meaningful enabled/disabled benchmark coverage.
- **Depends on**: Stage 1
- **Parallel**: no
- **Success criteria**: NAPI unit/integration tests cover options, errors, property-only mangling, cache round trips, and debug output; benchmark code builds and compares the end-to-end enabled/disabled paths.
- **Status**: Not Started

### Stage 3: Transformer provenance and compiler integration
- **Goal**: Preserve whether transformer-generated strings came from unquoted or quoted property keys, consume that provenance in the property mangler, and integrate the pass after lowering but before compression without silently breaking decorators, class fields, super access, exponentiation assignment, or object-rest exclusions.
- **Depends on**: Stage 1
- **Parallel**: no
- **Success criteria**: Positive and negative provenance tests pass; compiler integration passes transformed TS/JSX/decorator/class-field cases; unsupported enum behavior is explicit; no dummy-span provenance is accepted.
- **Status**: Not Started

### Stage 4: Full verification and local stack handoff
- **Goal**: Run repository-prescribed verification, review minsize and snapshots, confirm each stacked branch is independently reviewable, and record any intentionally deferred Rolldown-only render-boundary work.
- **Depends on**: Stages 1-3
- **Parallel**: no
- **Success criteria**: Minifier full verification, relevant transformer/NAPI tests, formatting, and clippy pass; worktree is clean; local stacked branch names and commit ranges are documented for review.
- **Status**: Not Started
