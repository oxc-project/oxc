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
Numbers in µs (lower = better). Step 1 column is post-conformance-restoration (the
stubbed-out fast paths from the spike have been re-implemented, so the gain is
realistic rather than inflated).

| File       | Baseline (main) | Spike  | Spike Δ    | Step 1 | Step 1 Δ  |
| ---------- | --------------- | ------ | ---------- | ------ | --------- |
| (small)    | 33.34           | 33.97  | +1.9%      | 31.64  | **−5.1%** |
| errors.ts  | 62.87           | 59.71  | **−5.0%**  | 59.73  | **−5.0%** |
| Search.tsx | 196.94          | 175.02 | **−11.1%** | 186.45 | **−5.3%** |
| core.js    | 208.41          | 182.00 | **−12.7%** | 196.10 | **−5.9%** |
| next.ts    | 298.56          | 266.19 | **−10.8%** | 282.73 | **−5.3%** |
| index.tsx  | 535.10          | 476.77 | **−10.9%** | 520.22 | **−2.8%** |
| (medium)   | 359.97          | 335.38 | **−6.8%**  | 338.95 | **−5.8%** |
| types.ts   | 1203.4          | 1147.6 | **−4.6%**  | 1133.2 | **−5.8%** |
| App.tsx    | 6147.5          | 5406.5 | **−12.1%** | 5802.3 | **−5.6%** |

After step 1 the speed-up vs main settles at a consistent **~5%** across files.
The spike's larger gains (10–13% on big files) reflected stubbed paths that
skipped real work; with conformance restored those gains shrink, but the
wrapper-size win + reduced per-node allocations still produce a real, durable
improvement.

## Conformance

| Suite | Baseline (main)  | Spike            | Step 1           |
| ----- | ---------------- | ---------------- | ---------------- |
| JS    | 746/753 (99.07%) | 615/753 (81.67%) | 746/753 (99.07%) |
| TS    | 591/601 (98.34%) | 541/601 (90.02%) | 591/601 (98.34%) |

Step 1 matches main exactly on both suites.

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
- **Step 2 (out of scope here)** would replace the remaining flatten
  algorithms with analyse-then-recurse rewrites to drop those allocations
  too, and explore an ancestor-stack `Formatter` to remove the typed
  `AstNodes` parent altogether.
