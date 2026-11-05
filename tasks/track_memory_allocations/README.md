# Oxc allocations stats

This task keeps track of the number of system allocations as well as arena allocations and the total number of bytes allocated. This is used for monitoring possible regressions in allocations and improvements in memory usage.

Update the snapshots with `cargo allocs`.

## Optional per-rule linter tracking

Run `cargo run -p oxc_track_memory_allocations --profile coverage --features linter`
to write only `tasks/track_memory_allocations/allocs_linter.yaml`, grouped by file
and plugin/rule name. This is opt-in: normal `cargo allocs` runs and CI do not run
it, including CI builds using `--all-features`.

Linter-enabled builds do not measure or rewrite the normal stage snapshots:
the linter enables additional semantic-analysis features that change their
allocation counts. Use `cargo allocs` without the feature for those snapshots.

The generated report is gitignored. Within each file, rules are sorted by
`sys alloc bytes` descending, then `sys allocs` descending, then rule name.
Values reflect the current run without snapshot tolerances. File size is omitted
from the per-rule metrics.

Only native rule callbacks (`should_run`, `run_once`, and `run`) are measured,
with the regular eligibility and AST type filters. Parsing, semantic analysis,
configuration, context construction, and context teardown are excluded. Type-aware
rules implemented by tsgolint are omitted. Skipped native rules have zero activity
unless their `should_run` callback allocates.

Rules run individually in rule-ID order, rather than interleaving their node
callbacks as the production runner does. They share a fresh per-file context.
Diagnostic storage growth is charged to the rule that triggers it, and lazy shared
work (such as React Compiler analysis) is charged to its first consumer. Diagnostics
remain alive until all rules finish. For these measurements, `arena size` is the
arena growth during that rule rather than the cumulative arena size.

## Metrics

Recorded per file and per stage (parser, semantic, transformer, minifier, formatter):

- `file size` - size of the source text
- `sys allocs` / `sys reallocs` / `sys deallocs` - system allocator activity during the stage
- `sys alloc bytes` - total bytes requested from the system allocator during the stage (allocation sizes plus reallocation growth)
- `sys peak growth` - high-water mark of live heap memory during the stage, relative to the live bytes when the stage started; captures transient heap the stage needs even when it frees it again before finishing
- `arena allocs` / `arena reallocs` - arena allocator activity during the stage
- `arena size` - bytes used in the arena at the end of the stage (cumulative: the arena is not reset between stages, so this includes the AST built by the parser)

Arena chunk allocations and deallocations are excluded from all `sys` metrics: chunk sizes are quantized and platform-dependent (whether an arena needs one more chunk depends on byte totals that vary across architectures), and for long-lived arenas chunk growth reflects the arena's history rather than the stage's own behavior. Arenas mark their chunk operations for the tracking allocator to skip.

## Cross-platform reproducibility

Allocation counts are identical on all platforms: the task pins mimalloc as the global allocator and excludes arena chunk allocations from the system allocation counts. `arena size` is not exactly reproducible: type layout differs slightly across architectures (e.g. hashbrown's table control groups are 16 bytes on x86_64 vs 8 bytes on aarch64), which shifts arena byte totals by up to ~128 bytes per file. So that a snapshot generated on any platform passes CI on the others, the snapshots are YAML: `cargo allocs` parses the committed snapshot and keeps a byte metric's committed value when the measured one is within that metric's tolerance (`arena size`: 1024 bytes; `sys alloc bytes` and `sys peak growth`: 1%, minimum 4 kB); only differences beyond the tolerance rewrite the value.
