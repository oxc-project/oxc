# Oxc allocations stats

This task keeps track of the number of system allocations as well as arena allocations and the total number of bytes allocated. This is used for monitoring possible regressions in allocations and improvements in memory usage.

## Callsite aggregation (optional)

You can also aggregate where allocations occur by source location, similar in spirit to dhat-rs. This is disabled by default because collecting backtraces for every allocation is expensive.

- Enable tracking: set `OXC_ALLOC_SITES=1`
- Optional sampling to reduce overhead: set `OXC_ALLOC_SAMPLE=N` to record 1 in `N` allocations (default `1000`).

Example:

```
OXC_ALLOC_SITES=1 OXC_ALLOC_SAMPLE=100 cargo run -p oxc_track_memory_allocations
```

At the end of the run, you'll see a "Top allocation sites" section like:

```
Top allocation sites (sampled 1 in 100)
Location                               | Allocations
----------------------------------------------------
crates/oxc_parser/src/lexer.rs:123     |         152
crates/oxc_semantic/src/builder.rs:45  |         101
...
```
