# `oxc_formatter` stack-allocated `AstNode` spike — findings

## Goal

Test whether the `oxc_formatter`'s arena-allocated `AstNode<'a, T>` wrapper
(`AstNode { inner: &'a T, parent: AstNodes<'a>, allocator: &'a Allocator,
following_span_start: u32 }`) can be replaced with a stack-allocated
`AstNode<'me, 'a, T>` that drops the `allocator` field, enabling `Copy`,
shrinking from ~40 to ~32 bytes, and avoiding per-node arena allocation.

## Status: Compiles, but incomplete

`cargo check -p oxc_formatter --lib` is clean (0 errors). Stubbed-out paths
mean conformance regresses from main's 99.07% / 98.34% (JS / TS) to **81.67%
/ 90.02%**. Benchmarks show consistent 5–13% speed-up on every file
(below).

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

Benchmarks via `cargo bench --bench formatter`. Numbers in µs (lower = better).

| File              | Baseline (main) | Spike   | Δ      |
| ----------------- | --------------- | ------- | ------ |
| (small)           |     33.34       |    33.97|  +1.9% |
| errors.ts         |     62.87       |    59.71| **−5.0%** |
| Search.tsx        |    196.94       |   175.02| **−11.1%** |
| core.js           |    208.41       |   182.00| **−12.7%** |
| next.ts           |    298.56       |   266.19| **−10.8%** |
| index.tsx         |    535.10       |   476.77| **−10.9%** |
| (medium)          |    359.97       |   335.38| **−6.8%** |
| types.ts          |   1203.4        |  1147.6 | **−4.6%** |
| App.tsx           |   6147.5        |  5406.5 | **−12.1%** |

The speed-up is consistent across all sizes. **Caveat:** part of the gain comes
from stubbed-out paths (member-chain layout, `ExpressionLeftSide::left()`,
`is_poorly_breakable_member_or_call_chain`) which now skip work the baseline
does. With those re-implemented, the gain may be smaller — but still likely
positive given the allocation savings.

## Conformance

| Suite | Baseline (main) | Spike     | Regression  |
| ----- | --------------- | --------- | ----------- |
| JS    | 746/753 (99.07%)| 615/753 (81.67%) | −131 cases (−17.4 pp) |
| TS    | 591/601 (98.34%)| 541/601 (90.02%) | −50  cases (−8.3 pp) |

The bulk of regressions are member-chain formatting — the path is currently
gated off via a `false &&` short-circuit in
`call_like_expression/mod.rs::write` so the broken `MemberChain` path never
runs.

## What works

- Stack-allocated wrappers compile and run.
- The lifetime model is sound — no `unsafe` lifetime extension was needed.
- Most simple formatting paths produce correct output (var decls,
  if/else, for loops, classes, basic expressions, JSX, template literals,
  conditional expressions, simple call expressions, etc.).

## What doesn't work — and why

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
context's wrapper), and the resulting `AstNode<'this, ...>` has a *shorter*
lifetime than `'me`. With arena-allocated wrappers the wrapper itself lived in
the arena, so `'this` and `'me` collapsed to a single arena lifetime — that
hid the variance issue. With stack-allocated wrappers, `'this < 'me` is real,
and any code that wants to thread the result back through a method returning
`AstNode<'me, ...>` (e.g. union enums like `BinaryLikeExpression::left()`,
`ReturnAndThrowStatement::argument()`, `TemplateLike::quasis()`,
`AnyJsxTagWithChildren::children()`, traversal loops in
`chain_members_iter`/`get_innermost_expression`) hits a hard lifetime wall.

### Workaround applied throughout the spike

For each broken site, I bypassed the auto-generated getter and constructed the
child `AstNode` directly from arena pointers, inheriting the **wrapper's
parent** rather than pointing at the immediate syntactic parent:

```rust
match self {
    Self::BinaryExpression(expr) => AstNode {
        inner: &expr.inner.left,
        parent: expr.parent,            // grandparent, not BinaryExpression
        following_span_start: expr.inner.right.span().start,
    },
    ...
}
```

This satisfies the borrow checker (everything has lifetime `'me` or `'a`) but
breaks `parent()` traversal: a child's parent now points to the wrapper's
outer parent, skipping a level. Most format passes only read `inner` and
spans, so they tolerate it; some — notably the
"is `({ foo }: { foo: string })` a single-param hug?" check in
`object_like.rs` — actually walk `parent()` and panic. That panic is a
canary: anywhere this trick is applied, parent-walking queries become
silently wrong.

## What a real fix would need

Either:

1. **Change the auto-generated getters to take `self` by value** and return
   `AstNode<'me, 'a, ...>`. This requires `AstNodes::BinaryExpression(self)`
   to take a reference with `'me` lifetime, which isn't satisfiable from a
   local stack frame — the parent ref needs to live somewhere that outlives
   `'me`. Options:
   - Allocate the wrapper at the parent's call site (caller passes
     `&'me AstNode<...>`), pushing the cost onto every recursive call.
   - Stash a small pool of wrappers in the formatter's `Formatter` and reuse
     slots — but then `parent` becomes a stable handle into that pool, not a
     direct reference.

2. **Drop the typed `AstNodes` parent and use a generic ancestor chain.**
   `parent` becomes `Option<&'me dyn ParentNode<'a>>` or similar, decoupling
   parent identity from a per-variant enum. Loses ergonomics but sidesteps
   the variance problem entirely.

3. **Keep arena allocation for wrappers** but make them `Copy` by storing the
   arena pointer directly (`*const AstNode`) rather than `&'me AstNode`.
   `unsafe` and dangerous, but avoids redesigning the parent system.

Option 1 (caller-supplied `&'me AstNode`) is the most idiomatic Rust answer.
It would mean every getter call at the formatter's top level has the form

```rust
let wrapped = caller.wrap(b.as_ref());
something_using(&wrapped);
```

instead of the current

```rust
something_using(caller.with_inner(b.as_ref()));
```

— a non-trivial API change but mechanical.

## Bottom line for the user

- **Lifetime design works** without `unsafe` lifetime extension.
- **Allocation pressure drops** (no per-node arena allocation, `~32 B` vs
  `~40 B`) and that translates to a real **5–13% bench speed-up** even with
  stubbed paths.
- **Auto-generated `parent()` chain is the blocker** for a clean port:
  variance forces every `match self / Self::Variant(node)` traversal to lose
  the outer `'me` lifetime, and the only easy escape (inheriting the
  grandparent) silently breaks parent-walking queries.
- **Path forward:** the wrapper itself is sound; the fix lives in
  `tasks/ast_tools/src/generators/formatter/ast_nodes.rs` — getters need to
  consume `self` and require a caller-supplied parent slot with `'me`
  lifetime. Without that change, the spike can't reach baseline parity.
