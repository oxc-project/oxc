# `oxc_formatter` stack-allocated `AstNode` spike — findings

## Goal

Test whether the `oxc_formatter`'s arena-allocated `AstNode<'a, T>` wrapper
(`AstNode { inner: &'a T, parent: AstNodes<'a>, allocator: &'a Allocator,
following_span_start: u32 }`) can be replaced with a stack-allocated
`AstNode<'me, 'a, T>` that drops the `allocator` field, enabling `Copy`,
shrinking from ~40 to ~32 bytes, and avoiding per-node arena allocation.

## Status: Step 1 complete — at conformance parity

The original spike compiled but stubbed several traversal paths to dodge a
lifetime issue, regressing conformance to 81.67% / 90.02% (JS / TS).
[`STEP_1_PLAN.md`](STEP_1_PLAN.md) describes the patterns used to fix every
broken site without re-introducing per-node arena allocation. After step 1:

- `cargo check -p oxc_formatter --lib` clean.
- Conformance back to baseline: **JS 746/753, TS 591/601** (matches main
  exactly).
- Benchmarks show a consistent **~5% speed-up** vs main on every file
  measured. The spike's larger 10–13% gains were inflated by stubbed paths;
  the durable gain is smaller but real.
- Formatter arena footprint drops **30–57%** vs main (per-node alloc gone +
  smaller wrapper); peak RSS drops **5–13%** on small files, **30–37%** on
  large ones (the floor of binary/libc overhead masks savings on small inputs).

## Design

```rust
pub struct AstNode<'me, 'a, T> {
    pub inner: &'a T,
    pub parent: AstNodes<'me, 'a>,
    pub following_span_start: u32,
}

impl<T> Copy for AstNode<'_, '_, T> {}
impl<T> Clone for AstNode<'_, '_, T> { fn clone(&self) -> Self { *self } }
```

`'me` is the parent's stack-frame lifetime (the lifetime against which `parent`
is borrowed). `'a` is the arena/AST lifetime. As recursion descends, `'me`
shrinks naturally: the wrapper for a child sits on the current frame and
borrows its parent for `'me`.

## Performance

Benchmarks via `cargo bench --bench formatter --no-default-features --features compiler`.
Numbers in µs (lower = better). Main and step 1 measured back-to-back on the same
machine with criterion's saved-baseline mechanism (`--save-baseline`); the spike
column is from an earlier run kept for historical context.

| File                       |    main | spike (historical) |  step 1 | step 1 Δ vs main |
| -------------------------- | ------: | -----------------: | ------: | ---------------: |
| RadixUIAdoptionSection.jsx |   33.57 |              33.97 |   31.73 |       **−5.48%** |
| errors.ts                  |   62.60 |              59.71 |   60.05 |       **−4.07%** |
| Search.tsx                 |  198.10 |             175.02 |  186.37 |       **−5.92%** |
| core.js                    |  209.05 |             182.00 |  196.54 |       **−5.98%** |
| next.ts                    |  301.27 |             266.19 |  284.87 |       **−5.44%** |
| index.tsx                  |  539.47 |             476.77 |  512.13 |       **−5.07%** |
| handle-comments.js         |  359.88 |             335.38 |  342.26 |       **−4.90%** |
| types.ts                   | 1205.71 |            1147.60 | 1133.31 |       **−6.00%** |
| App.tsx                    | 6168.15 |            5406.50 | 5839.58 |       **−5.33%** |

The speed-up vs main is a consistent **~5%** across files. The spike's larger
10–13% gains on big files reflected stubbed paths skipping real work; with
conformance restored those gains shrink, but the wrapper-size win + reduced
per-node allocations still produce a real, durable improvement.

## Conformance

| Suite | Baseline (main)  | Spike            | Step 1           |
| ----- | ---------------- | ---------------- | ---------------- |
| JS    | 746/753 (99.07%) | 615/753 (81.67%) | 746/753 (99.07%) |
| TS    | 591/601 (98.34%) | 541/601 (90.02%) | 591/601 (98.34%) |

Step 1 matches main exactly on both suites.

## Memory

Two complementary measurements: arena footprint (the formatter's own
allocations into the `bumpalo` arena) and peak RSS (the OS-level high-water
mark of the whole process).

### Formatter arena footprint

[`examples/mem_usage.rs`](examples/mem_usage.rs) parses each fixture into a
fresh `Allocator`, records `used_bytes`, formats, and records `used_bytes`
again. The delta is the formatter's own arena cost (the AST itself is the
same on both branches, since the parser is unchanged).

| File                       | AST KiB | main fmt KiB | step 1 fmt KiB | reduction |
| -------------------------- | ------: | -----------: | -------------: | --------: |
| RadixUIAdoptionSection.jsx |    20.7 |        134.2 |           90.5 |   −32.55% |
| errors.ts                  |    36.0 |        254.7 |          174.9 |   −31.34% |
| Search.tsx                 |    93.8 |        669.7 |          407.4 |   −39.17% |
| core.js                    |    83.0 |        663.4 |          407.7 |   −38.54% |
| next.ts                    |   124.3 |       1029.3 |          547.4 |   −46.82% |
| index.tsx                  |   193.0 |       1559.0 |         1118.1 |   −28.28% |
| handle-comments.js         |   145.2 |       1184.2 |          724.2 |   −38.85% |
| types.ts                   |  1016.9 |       6539.5 |         2798.9 |   −57.20% |
| App.tsx                    |  1902.6 |      19055.0 |        11918.4 |   −37.45% |

The formatter's arena footprint drops by **30–57%** vs main. This matches
design intent: per-node arena alloc on descent is gone, only flatten/chain
sites allocate, and the wrapper itself shrunk 40 → 32 bytes when it does
land in the arena. The big-file `fmt/ast` ratio drops from 6–10× on main to
3–6× on step 1.

`Allocator::used_bytes` over-counts on both branches (it includes Vec excess
capacity and dropped-but-not-freed allocations), so the relative comparison
is what matters; absolute numbers are upper bounds.

### Peak RSS

[`examples/peak_rss.rs`](examples/peak_rss.rs) is a minimal binary that
parses + formats a single file and exits, suitable for `/usr/bin/time -l`.
Best of three runs on each fixture, on a quiet system.

| File                       | main MB | step 1 MB | reduction |
| -------------------------- | ------: | --------: | --------: |
| RadixUIAdoptionSection.jsx |    2.91 |      2.75 |    −5.38% |
| errors.ts                  |    3.19 |      3.03 |    −4.90% |
| Search.tsx                 |    3.81 |      3.53 |    −7.38% |
| core.js                    |    3.62 |      3.41 |    −6.03% |
| next.ts                    |    4.20 |      3.78 |   −10.04% |
| index.tsx                  |    4.92 |      4.30 |   −12.70% |
| handle-comments.js         |    4.33 |      3.92 |    −9.39% |
| types.ts                   |   10.05 |      6.28 |   −37.48% |
| App.tsx                    |   23.25 |     16.14 |   −30.58% |

Peak RSS includes a constant ~2.5 MB floor (binary code, libc, dyld,
allocator chunk overhead) on every run. On small files that floor dominates,
so reductions look modest. On large files the formatter's arena is the
dominant fraction and the savings show through directly: types.ts down 37%,
App.tsx down 31% (≈7 MB peak saved on App.tsx alone). The shape tracks the
arena-footprint table, dampened by the constant overhead.

## What works

- Stack-allocated wrappers compile and run.
- The lifetime model is sound — no `unsafe` lifetime extension was needed.
- Most simple formatting paths produce correct output (var decls,
  if/else, for loops, classes, basic expressions, JSX, template literals,
  conditional expressions, simple call expressions, etc.).

## What didn't work in the spike — and how step 1 fixed it

The auto-generated child getters in `ast_nodes/generated/ast_nodes.rs` look
like:

```rust
pub fn left<'this>(&'this self) -> AstNode<'this, 'a, Expression<'a>> {
    AstNode {
        inner: &self.inner.left,
        parent: AstNodes::BinaryExpression(self),  // self: &'this AstNode<...>
        following_span_start: ...,
    }
}
```

The child borrows `self` for `'this` (a borrow lifetime against the calling
context's wrapper), and the resulting `AstNode<'this, ...>` has a _shorter_
lifetime than `'me`. With arena-allocated wrappers the wrapper itself lived in
the arena, so `'this` and `'me` collapsed to a single arena lifetime — that
hid the variance issue. With stack-allocated wrappers, `'this < 'me` is real,
and any code that wants to thread the result back through a method returning
`AstNode<'me, ...>` (e.g. union enums like `BinaryLikeExpression::left()`,
`ReturnAndThrowStatement::argument()`, `TemplateLike::quasis()`,
`AnyJsxTagWithChildren::children()`, traversal loops in
`chain_members_iter`/`get_innermost_expression`) hit this wall in the spike.

The spike worked around it by inheriting the wrapper's _grandparent_ as the
new wrapper's `parent`, which compiled but silently corrupted `parent()`-
walking queries — the source of the −17 pp conformance regression.

### Step 1 resolution

[`STEP_1_PLAN.md`](STEP_1_PLAN.md) captures the three patterns that replaced
the workaround. Roughly:

- **Pattern A — getter on `&'this self`**: the call site uses the result in
  scope only, so we let the result lifetime narrow to `'this` and use the
  matched reference as the parent. Zero allocations.
- **Pattern B — chain loop**: each iteration arena-allocates the _current_
  wrapper to give the next iteration's wrapper a `'me`-bound parent reference.
  One alloc per chain step.
- **Pattern C — Vec-pushing flatten**: the entry point arena-allocates the
  input wrapper once so its no-alloc getters return `'me`-bound children.
  One alloc per recursion level.

The mental model is _lifetime narrowing via covariance_, not widening:
`bumpalo::alloc(&self, T) -> &mut T` returns a reference whose lifetime is
inferred to satisfy `T: 'r`, so for an `'me`-bound `T` we get a `'me`-bound
reference. Putting that reference into `AstNodes::Variant(...)` produces a
parent slot the borrow checker accepts.

`Allocator` lives on `Formatter` (not on `AstNode`), so the wrapper stays at
32 B and `Copy`. Construction methods that need to allocate take
`&Formatter` (or `&Allocator`) as an explicit argument.

## Bottom line

- **Lifetime design works** without `unsafe` lifetime extension.
- **Wrapper is 32 B** (vs main's ~40 B) and `Copy`.
- **Per-node arena alloc on descent is gone**; only flatten/chain sites
  allocate, and only at chain-step granularity.
- **Conformance is at parity with main** (746/591) and a consistent
  **~5% speed-up** holds across files of every size.
- **Memory drops noticeably**: formatter arena footprint −30 to −57% vs main;
  peak RSS −5 to −13% on small files, −30 to −37% on large files.
- **Step 2 (out of scope here)** would replace the remaining flatten
  algorithms with analyse-then-recurse rewrites to drop those allocations
  too, and explore an ancestor-stack `Formatter` to remove the typed
  `AstNodes` parent altogether.
