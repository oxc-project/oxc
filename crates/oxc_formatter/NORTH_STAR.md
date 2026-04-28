# NORTH STAR — Stack-Allocated `AstNode` Refactor

> **THIS DOCUMENT IS THE PRIMARY SOURCE OF TRUTH FOR THIS WORK.**
> Re-read it after every meaningful chunk of work and immediately after any
> conversation compaction. If anything in compacted summaries contradicts this
> document, **THIS DOCUMENT WINS**.

## Task summary

Refactor `oxc_formatter`'s `AstNode<T>` wrapper from arena-allocated to stack-allocated.

**Current state:** `AstNode<'a, T>` carries `inner: &'a T`, `parent: AstNodes<'a>`,
`allocator: &'a Allocator`, and `following_span_start: u32`. A single arena lifetime
`'a` is used for both AST data and wrapper, forcing every wrapper into the bump arena.

**Target state:** Two lifetimes — `'me` for the parent's stack frame and `'a` for the
AST data. Wrappers become `Copy`, stack-allocatable, ~32 bytes (vs current ~40).
Children's wrappers borrow their parent's stack frame via `'me`; the lifetime shrinks
naturally as recursion descends.

**Spike result:** [`crates/oxc_formatter/src/ast_nodes/stack_spike.rs`](src/ast_nodes/stack_spike.rs)
validates the lifetime design works for a representative recursive walk
(`Program → IfStatement → Expression → BinaryExpression`), with no `unsafe`, no
lifetime extension hacks, and clean compile.

**Job:** Take the design from spike to full implementation. Touches codegen
templates, hand-written `node.rs` / `iterator.rs` / `lib.rs`, helper structs, and
hundreds of call sites. Then measure perf/memory impact.

## North star principles (verbatim from the user)

> **1.** I'm not imagining the result being the final version that becomes a PR
> ready to be merged. It's more to test the idea, check that it can work in
> practice, and measure the perf/memory usage impact. If it does work, we'd look
> at what is clean and what is messy, and probably start again from scratch and
> do it "properly" with more thought about design decisions, building on what we
> learned from the first attempt.
>
> **2.** I don't care how long it takes. I just want to get on with other work
> while you're doing it. I'd prefer that you just continue going and finish the
> job, rather than stopping to ask me questions. Completion is much more
> important than perfection.
>
> **3.** Lifetimes are important. As you say, that's the hard bit. If we can't
> make it compile without resorting to unsafe lifetime extension, then the
> experiment is a failure. A handful of places marked "TODO: Fix this dodgy
> lifetime extension later" is fine, but it's got to work in vast majority of
> places without that kind of unsafe hackery, to prove the concept.

## Lessons from first attempt — READ BEFORE RESTARTING

The first attempt reached ~73% completion before hitting cascading complexity at
call sites. See [REFACTOR_FINDINGS.md](REFACTOR_FINDINGS.md) for the full
post-mortem. **The lifetime model itself was validated** (no `unsafe` needed); the
issue was the call-site adaptation strategy.

The second attempt operates with these refinements:

1. **Drop `as_ast_nodes` from the codegen entirely.** It was forcing arena allocation
   (or a parallel `AstNodesOwned` enum). Instead, call sites do an inline
   match-and-construct using a small helper:
   ```rust
   impl<'me, 'a, T> AstNode<'me, 'a, T> {
       pub fn with_inner<U>(&self, inner: &'a U) -> AstNode<'me, 'a, U> {
           AstNode { inner, parent: self.parent, following_span_start: self.following_span_start }
       }
   }
   ```
   Call sites change from
   `match X.as_ast_nodes() { AstNodes::Y(it) => ... }` to
   `match X.inner { EnumType::Y(boxed) => { let it = X.with_inner(boxed.as_ref()); ... } }`.
   This eliminates `Allocator` usage in `as_ast_nodes` AND fixes the "10 helpers
   don't have `f`" problem at the same time.

2. **Keep `&'b AstNode<'me, 'a, T>` references in helper structs.** Don't try to
   convert to owned. The owned conversion cascades into too many call-site fixes;
   keeping references is a minimal-diff change. Helper structs become
   `FormatX<'me, 'a, 'b>(&'b AstNode<'me, 'a, T>)`.

3. **Allocator usage shrinks to exactly one site:** `to_arguments`, kept as TODO.

## What "done" looks like

The spike is a proof of concept. The whole task is done when:

1. The full formatter compiles with the new stack-based `AstNode<'me, 'a, T>`.
2. Conformance tests run. Ideal: most pass. Acceptable: some diagnosable failures.
3. Perf/memory impact is measured (vs the `main` baseline).
4. A short summary of what's clean vs messy is written, so the eventual
   "do it again properly" pass has a starting point.

**Do NOT pursue:**

- Pristine code quality.
- PR-mergeable shape.
- 100% conformance parity.
- Cleaning up unrelated issues.

## CRITICAL operating instructions

### Re-read this document at these triggers

- **After every ~20 file edits** (rough proxy for a "chunk of work").
- **Before writing `unsafe` or `transmute`** — confirm there isn't a non-unsafe alternative.
- **Before composing any message asking the user a question** — note 2 says decide and proceed.
- **Whenever you start polishing** — stop, re-read, check whether polish is on the path to "done."
- **Immediately after any conversation compaction** — compaction is the highest-risk drift point.

### Quick self-check (run on every re-read)

Ask yourself, mechanically:

1. **Am I about to ask the user something?** → Decide and proceed instead. If genuinely undecidable, write a `TODO` note and pick a default.
2. **Am I about to write `unsafe` for lifetime reasons?** → Try harder. If still blocked after a real attempt, leave a `// TODO: Fix this dodgy lifetime extension later` and move on.
3. **Am I optimising / refactoring beyond the immediate need?** → Stop. This whole codebase gets rewritten properly later.
4. **Am I scope-creeping into unrelated cleanup?** → Stop. Note it elsewhere; don't fix it.
5. **Have I been working on the same hard sub-problem for a long time?** → Try a different angle once. If still stuck, `TODO` it and move on.

### Anti-patterns

- **Asking the user mid-task.** Violates note 2.
- **`unsafe` / `transmute` for lifetime reasons.** Violates note 3.
- **Polishing the experimental code.** Violates note 1.
- **Refusing to leave `TODO` markers.** They are explicitly allowed by note 3.
- **Treating compile errors as design failures.** They are normal; iterate.
- **Treating compaction as authoritative.** It is not. This document is.

## Compaction directive

**WHEN COMPACTION OCCURS, THIS DOCUMENT BECOMES THE PRIMARY CONTEXT.**

After compaction, the assistant must:

1. Re-read this entire document.
2. Treat the principles above as overriding any compacted summary.
3. Reconstruct task state from the file system (existing edits, todo list, recent
   commits) rather than trusting compacted memory of those details.

**EVERY COMPACTED CONVERSATION SUMMARY MUST PRESERVE THE EXISTENCE AND PATH OF
THIS DOCUMENT.** If you are summarising for compaction, the FIRST line of the
summary should be:

> **NORTH STAR DOCUMENT:** `crates/oxc_formatter/NORTH_STAR.md` — re-read in full before any further action.
