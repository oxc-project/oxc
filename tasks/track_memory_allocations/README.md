# Oxc allocations stats

This task keeps track of the number of system allocations as well as arena allocations and the total number of bytes allocated. This is used for monitoring possible regressions in allocations and improvements in memory usage.

Update the snapshots with `cargo allocs`.

## Cross-platform reproducibility

Allocation counts are identical on all platforms: the task pins mimalloc as the global allocator and excludes arena chunk allocations from the system allocation counts. `arena size` is not exactly reproducible: type layout differs slightly across architectures (e.g. hashbrown's table control groups are 16 bytes on x86_64 vs 8 bytes on aarch64), which shifts arena byte totals by up to ~128 bytes per file. So that a snapshot generated on any platform passes CI on the others, the snapshots are YAML: `cargo allocs` parses the committed snapshot and keeps its `arena size` value when the measured one is within `ARENA_SIZE_TOLERANCE` (1024 bytes) of it; only differences beyond the tolerance rewrite the value.
