# STEP 1 PLAN — Restore Conformance via Arena Alloc at Flatten Sites

> **THIS DOCUMENT IS THE PRIMARY SOURCE OF TRUTH FOR THIS PHASE OF WORK.**
> Re-read it after every meaningful chunk of work and immediately after any
> conversation compaction. If anything in compacted summaries contradicts this
> document, **THIS DOCUMENT WINS**.

> **`NORTH_STAR.md` describes the spike phase, which is COMPLETE.**
> [`REFACTOR_FINDINGS.md`](REFACTOR_FINDINGS.md) is the spike's post-mortem.
> Both are historical context for _why_ this plan exists. **This document
> describes what to do next.**

## ⛔ DO NOT STOP UNTIL DONE ⛔

> **EXPLICIT USER DIRECTIVE — HIGHEST PRIORITY:**
>
> **CONTINUE WITHOUT STOPPING UNTIL THE WORK IS DONE.**
>
> Do not pause for review, do not give status updates that imply stopping.
> **Keep going.** "Status check" / "checkpoint with user" is **NOT** a valid
> stopping reason.
>
> "Done" means all four criteria in [What "done" looks like](#what-done-looks-like)
> below.
>
> Stopping triggers (the ONLY valid reasons to stop):
>
> 1. Hitting a fundamental design wall that requires user input on architecture.
> 2. Reaching all "done" criteria.

## Background

The spike (see [`REFACTOR_FINDINGS.md`](REFACTOR_FINDINGS.md)) proved the
stack-allocated `AstNode<'me, 'a, T>` design works for **recursive descent**
but breaks at **flatten-style** sites that build a `Vec<AstNode<...>>` whose
entries each reference a parent wrapper. The lifetime issue: parent references
in `AstNodes<'me, 'a>` need `&'me AstNode<...>`, and a wrapper sitting on the
local stack frame can't satisfy `'me` for any sibling/cousin in the flat
sequence.

The spike worked around this by setting `parent: x.parent` (inheriting the
_grandparent_ instead of pointing at the immediate parent). That compiles but
silently corrupts `parent()`-walking queries, which is the source of the
~17 percentage-point conformance regression.

## Goal of this phase

Restore conformance to baseline (`746/753 JS, 591/601 TS`) by **re-introducing
arena allocation at the broken sites only**, while preserving the spike's
32-byte stack-allocated wrapper for the rest of the codebase.

After step 1, `oxc_formatter` is in a coherent, mergeable state:

- Wrapper is 32 bytes (vs main's 40), `Copy`, no `Allocator` field.
- Per-node arena alloc is gone on the descent paths.
- Per-flatten-parent arena alloc remains at flatten sites (much cheaper than
  main's per-node alloc).
- Conformance matches main.

## Approach (b) — `Allocator` on `Formatter`

The allocator field stays _off_ `AstNode`. Instead it lives on `Formatter` (or
is threaded via `&mut Formatter`, which most call sites already have). Code
that needs to allocate a wrapper into the arena calls `f.allocator().alloc(...)`.

This is non-negotiable for this phase: putting `Allocator` back on `AstNode`
would forfeit the wrapper-size win that drove half the spike's perf gain.

## Translation rule for broken sites

Wherever the spike currently does

```rust
parent: x.parent          // grandparent — wrong
```

replace with

```rust
let arena_x: &'a AstNode<'a, 'a, SomeType<'a>> = f.allocator().alloc(*x);
parent: AstNodes::SomeVariant(arena_x)   // immediate parent — correct
```

The arena-allocated wrapper has `'me = 'a`. Since `AstNode<'me, 'a, T>` is
covariant in `'me` and `'a` outlives any stack frame's `'me`, an `'a`-bound
wrapper is acceptable in any `'me` slot.

The cost is **one arena alloc per flatten-parent**, not per node. In a chain
`a + b + c + d + e` that's ~5 allocs vs main's per-node allocs.

## Sites to fix

Every site listed below currently uses the spike's grandparent workaround and
needs the translation rule applied. Found by greppng for `parent: .*\.parent`
on AstNode constructions in non-generated code.

- `src/print/binary_like_expression.rs::BinaryLikeExpression::left` and
  `::right` — match arms for `LogicalExpression` and `BinaryExpression`.
- `src/print/return_or_throw_statement.rs::ReturnAndThrowStatement::argument` —
  match arms for `ReturnStatement` and `ThrowStatement`.
- `src/print/template/mod.rs::TemplateLike::quasis` — match arms for
  `TemplateLiteral` and `TSTemplateLiteralType`.
- `src/print/jsx/element.rs::AnyJsxTagWithChildren::children` — match arms for
  `Element` and `Fragment`.
- `src/print/arrow_function_expression.rs` arrow-chain `next` construction
  inside `ArrowFunctionLayout::for_arrow`'s loop.
- `src/utils/assignment_like.rs::AssignmentLike::get_right_expression` — match
  arms for `VariableDeclarator`, `AssignmentExpression`, `ObjectProperty`,
  `PropertyDefinition`, `AccessorProperty`.
- `src/utils/assignment_like.rs::get_innermost_expression` — match arms for
  `UnaryExpression`, `TSNonNullExpression`, `AwaitExpression`, `YieldExpression`.
- `src/print/mod.rs` — `TSModuleDeclaration` body-loop (around line 1675).
- `src/print/parameters.rs::FormalParametersIter::from` — `items` and `rest`
  AstNode construction.

## Stubbed paths to un-stub

The spike also disabled some traversal-heavy code with `false &&` gates,
`return None` stubs, or `#[cfg(any())]` blocks. These need real
implementations using the same translation rule:

- `src/print/call_like_expression/mod.rs` — `false &&` gate on
  `MemberChain::from_call_expression`. Re-enable. The `MemberChain` machinery
  itself needs fixing (next bullet).
- `src/utils/member_chain/mod.rs::chain_members_iter` — currently emits only
  the root call. Restore the full traversal, allocating wrappers in the arena
  where re-binding `next` requires `'me` lifetime.
- `src/utils/expression.rs::ExpressionLeftSide::left` — currently returns
  `None`. Restore using arena-allocated parent wrappers.
- `src/utils/assignment_like.rs::is_poorly_breakable_member_or_call_chain` —
  body is `false`-stubbed; original is gated by `#[cfg(any())]`. Restore.
- `src/print/object_like.rs::should_hug` — replaced `unreachable!()` with
  `return false` to mask broken parent chain. Once parents are correct,
  restore the `unreachable!()`.

## Patches needed in `node.rs` / `iterator.rs`

The wrapper has no `Allocator` field. But construction methods that allocate
need access to one. Two options:

1. Methods that synthesize wrappers (e.g. the to-be-restored versions of
   `BinaryLikeExpression::left`) take `&mut Formatter` (or `&'a Allocator`)
   as an explicit argument. Cleaner but more verbose.
2. The `AstNode<'me, 'a, T>` enum variants gain a paired arena-construction
   method on the wrapper itself, which still requires an `&'a Allocator`
   parameter.

Take option 1. Pass `f` (or `f.allocator()`) explicitly to construction methods
that need to allocate. No magical state on the wrapper.

## Conformance gate

Step 1 is **not done** until:

```
cargo run --release -p oxc_prettier_conformance
```

reports **JS: 746/753, TS: 591/601** (matching main exactly).

A regression of even one case is grounds to keep going, not declare done.

## Performance check

After conformance is restored, run:

```
cargo bench --bench formatter --no-default-features --features compiler
```

Compare against main baseline (already captured in
[`REFACTOR_FINDINGS.md`](REFACTOR_FINDINGS.md)). Spike showed 5–13% speed-up;
expect the post-step-1 number to be smaller (because flatten-site allocs
return) but still positive — say 3–8% on the larger files. Update the findings
doc with the new numbers.

## Out of scope for step 1

These are step 2 (or later) concerns. Do **NOT** address them in this phase:

- Replacing flatten algorithms with analyse-then-recurse rewrites.
- Removing `Allocator` usage entirely.
- Replacing `parent: AstNodes<'me, 'a>` with an ancestor stack on `Formatter`.
- Lazy `following_span_start`.
- Codegen rewrites in `tasks/ast_tools/`.
- Any algorithmic restructuring of member-chain layout, JSX child
  classification, etc.
- `to_arguments` Vec allocation — kept as-is (synthesis case, not flatten).

If you find yourself reaching for any of these to "make this site cleaner,"
**stop**. The boundary between step 1 (mechanical) and step 2 (algorithmic) is
load-bearing. Step 1 is mechanical translation; step 2 is creative refactor.

## What "done" looks like

1. `cargo check -p oxc_formatter --lib` clean.
2. `cargo run --release -p oxc_prettier_conformance` reports baseline numbers
   exactly (JS: 746/753, TS: 591/601).
3. `cargo bench --bench formatter --no-default-features --features compiler`
   shows perf at least at parity with main, ideally still positive.
4. [`REFACTOR_FINDINGS.md`](REFACTOR_FINDINGS.md) updated with step-1 numbers
   and a brief "what's still messy" section pointing at step 2.

## CRITICAL operating instructions

### Re-read this document at these triggers

- **After every ~20 file edits** (rough proxy for a "chunk of work").
- **Before writing `unsafe` or `transmute`** — there should be no need in
  step 1. If you reach for it, the right answer is almost always "use
  `f.allocator().alloc(...)` instead."
- **Before composing any message asking the user a question** — decide and
  proceed.
- **Whenever you start polishing** — stop, re-read, check whether polish is
  on the path to "done."
- **Immediately after any conversation compaction.**

### Quick self-check (run on every re-read)

1. **Am I about to ask the user something?** → Decide and proceed. If
   genuinely undecidable, write a `TODO` note and pick a default.
2. **Am I about to redesign a flatten site instead of fixing parents?** →
   Stop. That's step 2. Apply the translation rule and move on.
3. **Am I changing codegen / ast_tools?** → Stop. Out of scope for step 1.
4. **Am I scope-creeping into unrelated cleanup?** → Stop. Note it elsewhere.
5. **Am I about to merge before conformance hits 746/591?** → Don't.

### Anti-patterns

- **Asking the user mid-task.** Violates "do not stop."
- **`unsafe` / `transmute` for lifetime reasons.** The arena alloc is the
  legitimate escape hatch.
- **Algorithmic refactor of flatten sites.** Step 2.
- **Removing the `Allocator` from `Formatter`.** Step 2.
- **Treating partial conformance gain as "done."** Baseline parity is the bar.
- **Treating compaction as authoritative.** This document is.

## Compaction directive

**WHEN COMPACTION OCCURS, THIS DOCUMENT BECOMES THE PRIMARY CONTEXT.**

After compaction, the assistant must:

1. Re-read this entire document.
2. Treat its principles as overriding any compacted summary.
3. Reconstruct task state from the file system (existing edits, todo list,
   recent commits, conformance runner output) rather than trusting compacted
   memory.

**EVERY COMPACTED CONVERSATION SUMMARY MUST PRESERVE THE EXISTENCE AND PATH OF
THIS DOCUMENT.** If you are summarising for compaction, the FIRST line of the
summary should be:

> **STEP 1 PLAN DOCUMENT:** `crates/oxc_formatter/STEP_1_PLAN.md` — re-read in full before any further action.
