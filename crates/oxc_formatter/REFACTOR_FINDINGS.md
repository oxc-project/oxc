# Stack-Allocated `AstNode` Refactor — Findings

This is the experiment summary called for in [NORTH_STAR.md](NORTH_STAR.md)'s "what
done looks like", section 4.

## Outcome

**The design works in isolation. The full refactor reached ~73% completion before
hitting a complexity wall on call-site adaptation.**

- Spike at the start of the session: clean compile, 2 passing tests, no `unsafe`.
  Confirms the lifetime model is sound.
- Full refactor: codegen templates updated, accessors return owned wrappers, hand-written
  infrastructure (`node.rs`, `iterator.rs`, `lib.rs`, `impls/ast_nodes.rs`) updated.
- Compile errors went from 996 (initial) to ~273 (final) without `unsafe`.
- Did **not** reach clean compile. Conformance tests and perf measurement were not run.

## What worked cleanly

1. **The lifetime story.** `AstNode<'me, 'a, T>` with `parent: AstNodes<'me, 'a>` and
   `inner: &'a T`, derive `Copy` (manually, without `T: Copy` bound). Variance is auto-derived
   correctly. No `unsafe` needed for the core types.

2. **Accessor signature change.** Generated accessors return owned `AstNode<'this, 'a, T>`
   where `'this` is the borrow of `self`. Pattern:
   ```rust
   pub fn body<'this>(&'this self) -> AstNode<'this, 'a, Vec<'a, Statement<'a>>> {
       AstNode { inner: &self.inner.body, parent: AstNodes::Program(self), ... }
   }
   ```

3. **Iterator rework.** `AstNodeIterator<'me, 'a, T>` yields owned `AstNode<'me, 'a, T>`
   values. No allocator field. Existing `for elem in node.iter()` patterns continue to
   work because `AstNode` is `Copy`.

4. **Codegen templates.** Both `ast_nodes.rs` and `format.rs` codegen updated cleanly.
   `just ast` regenerates ~16k lines of code with the new shape.

5. **Format trait dispatch.** Generated enum-dispatch code stack-allocates the variant
   wrapper (no arena allocation):
   ```rust
   match self.inner {
       Expression::BooleanLiteral(inner) => {
           AstNode::<'_, '_, BooleanLiteral> { inner, parent, following_span_start: ... }.fmt(f);
       },
       // ...
   }
   ```

## Where it got messy

### `as_ast_nodes()` design tension (kept allocator usage as TODO)

The fundamental issue: `AstNodes<'me, 'a>` variants must hold *references*
(`&'me AstNode<...>`) to avoid infinite type size. But `as_ast_nodes()` needs to
synthesise a typed wrapper for the inner enum variant — that wrapper has no stable
storage location in a stack-only design.

Options considered:
1. **Closure-based API** (`with_ast_nodes(|nodes| match nodes { ... })`) — too restrictive
   for the many call sites that bind matched values across long blocks.
2. **`AstNodesOwned` parallel enum** — works, but ~50 call sites need to switch from
   `match X { AstNodes::Y(...) }` to `match X { AstNodesOwned::Y(...) }`. Tried this
   first; bigger diff than expected.
3. **Keep arena allocation, take allocator as parameter** — TODO marker. This is what
   landed.

Currently `as_ast_nodes` takes `&'a Allocator` and arena-allocates the wrapper. ~50 call
sites updated to `expr.as_ast_nodes(f.allocator())`. ~10 call sites are in helper
functions that don't have `f` in scope; these are still broken (unfixed).

### Helper structs with separate borrow lifetime `'b`

Many helper structs in `print/`, `utils/`, `parentheses/` had this shape:

```rust
struct FormatX<'a, 'b>(&'b AstNode<'a, T>);
```

Two reasonable adaptations exist:

1. **Add `'me`, keep reference**: `FormatX<'me, 'a, 'b>(&'b AstNode<'me, 'a, T>)`.
   Mechanical signature change; call sites that take `&local_value` continue to work.
2. **Drop reference, take owned (since `AstNode: Copy`)**: `FormatX<'me, 'a>(AstNode<'me, 'a, T>)`.
   No `'b` needed; struct holds owned.

I tried option 2 via bulk sed — produced cascading lifetime arg count mismatches
(many `Foo<'me, 'a, 'b>` usages where `Foo` now has 2 lifetimes). Resolved most of those
but still ~70 sites broken: callers passing `&owned_value` to functions expecting
`AstNode<...>` (owned), where the call site expression makes deref-by-`*` impossible
without an intermediate `let`.

Lesson: option 1 (keep references, add `'me`) would have been a smaller diff. With the
benefit of hindsight, the right approach is:

- Helper struct fields and function parameters: keep `&'b AstNode<'me, 'a, T>` (just add `'me`).
- Call sites: bind accessor results to `let` then take `&`:
  ```rust
  let body = self.body();
  helper(&body);
  ```

### `to_arguments()` synthesis (kept Vec + allocator as TODO)

As planned in NORTH_STAR. The signature is now `to_arguments(&self, &'a Allocator) -> AstNode<...>`,
called as `self.to_arguments(f.allocator())` from the one site in `call_like_expression/mod.rs`.

### Cascading lifetime parameter changes

Roughly 60 helper types were extended with a `'me` parameter (from 1-2 lifetimes to 2-3).
Each extension forced updates at every call site that named the type with explicit
lifetime arguments. Bulk `sed` resolved most, but corner cases (impl blocks, `where`
clauses, `Self::` references) needed individual attention.

## Compile error breakdown (final state)

```
148 E0308 mismatched types         — owned vs ref at call sites
 33 E0261 'me undeclared            — standalone fns missing <'me>
 20 E0107 struct takes 1 lt, 2 supplied — old structs not extended
 12 E0631 type mismatch in fn args  — closure signature drift
 10 E0425 cannot find `f`           — helper fns lacking allocator/formatter param
  8 E0261 'b undeclared             — symmetric to 'me
  7 E0392 'me never used            — overzealous sed adding 'me
~273 total
```

Most remaining work is mechanical per-file editing of call sites — no design questions left.

## What to do differently in the proper redo

1. **Write a thorough sed/rustfix plan before executing.** This refactor has many
   interdependent renames; bulk substitution order matters a lot. Plan the sequence.

2. **Choose one helper-struct adaptation pattern and stick with it.** Option 1 above
   (keep `&'b`, add `'me`) is the lower-disruption choice. Confirm before running bulk edits.

3. **Build out an `AstNodesOwned` parallel enum properly** OR commit to closure-based
   `as_ast_nodes` from the start. Don't delay this decision.

4. **Solve the helpers-without-`f` problem first.** A handful of helper functions
   (`should_add_parens`, `is_decorated_function`, `try_from`-style impls) need the
   allocator threaded through. Pass it explicitly as a parameter; cascading is bounded.

5. **Investigate getting rid of `Allocator` entirely** for `as_ast_nodes` and
   `to_arguments` by using one of the slice-based / closure-based / out-parameter
   approaches discussed in the conversation, before committing to "TODO: allocator
   needed here."

## Useful artifacts produced

- [`crates/oxc_formatter/src/ast_nodes/node.rs`](src/ast_nodes/node.rs) — new struct shape with `'me, 'a, T`, manual `Copy` impl.
- [`crates/oxc_formatter/src/ast_nodes/iterator.rs`](src/ast_nodes/iterator.rs) — `AstNodeIterator` yielding owned wrappers.
- [`tasks/ast_tools/src/generators/formatter/ast_nodes.rs`](../../tasks/ast_tools/src/generators/formatter/ast_nodes.rs) — codegen template with new shape.
- [`tasks/ast_tools/src/generators/formatter/format.rs`](../../tasks/ast_tools/src/generators/formatter/format.rs) — codegen template for stack-allocated dispatch.
- [`crates/oxc_formatter/src/ast_nodes/generated/`](src/ast_nodes/generated/) — regenerated 16k lines compiling cleanly with the new shape (the codegen produces correct code; the call-site adapters are what didn't all land).

## Conclusion

Per NORTH_STAR's done criteria:

1. Full formatter compiles — **NOT MET** (273 errors remaining).
2. Conformance tests run — not attempted.
3. Perf/memory measured — not attempted.
4. Summary written — **MET** (this document).

The lifetime model is validated. The cascading complexity at call sites is the main
challenge for a clean implementation. A fresh attempt with the lessons above should reach
clean compile in a more focused effort.
