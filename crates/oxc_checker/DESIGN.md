# oxc_checker — Design

Experimental **eager** TypeScript type checker built on **isolated declarations**.

## Premise

tsc/tsgo are lazy whole-program checkers because TS files have no checkable
boundary: export types are inferred from implementations, so a file's interface
is only discoverable _by_ checking, recursively, across a cyclic module graph.

`isolatedDeclarations` restores the boundary: every export's type is
syntactically derivable from its own file. This checker **forces** that mode
and exploits it:

1. Surface extraction needs no other files → no dependency order, no cycle
   problem, every phase embarrassingly parallel.
2. The cross-file "type information" is the set of _declared surfaces_
   (unevaluated), never inferred — so it can be computed once, frozen, and
   shared read-only across checking threads.
3. A file's internal types are scratch — computed during that file's check and
   dropped. Memory is O(one AST + all surfaces), not O(all ASTs).

Prior art: Flow "types-first", Hack decl tables, tsc project references.

## Pipeline

```
load (parallel, wave-based)                                 loader.rs
  walk tsconfig dir → parse file → force IsolatedDeclarations transform
  → lower .d.ts AST into a Send+Sync surface (ir.rs types, name refs pending)
  → resolve import specifiers (oxc_resolver, tsconfig-aware)
  → newly resolved files (e.g. node_modules *.d.ts) join the next wave

link (single-threaded, cheap)                               link.rs
  assign SymbolIds, build per-file export tables, chase re-export chains
  (export * / export { x } from), rewrite pending name refs → SymbolId,
  concatenate per-file type vecs into one frozen TypeTable
  ⇒ ProgramEnv: Send + Sync, the only data checking threads share

check (parallel per file, any order)                        check/
  re-parse the file (AST is per-thread scratch; oxc parse speed makes
  "AST as cache, not state" viable) → lower local annotations against env
  → infer initializers → relate → diagnostics
```

The oxc AST cannot be the shared environment: nodes contain `Cell` fields
(`node_id`), so the AST is `!Sync` by construction. The owned IR in `ir.rs` is
the load-bearing design decision, not an optimization.

## Type representation (`ir.rs`)

`TypeTable` = append-only `Vec<Type>` with fixed ids for intrinsics. During
checking each thread layers a `LocalTypes` scratch table on top (ids ≥
`env.types.len()` are local) — the same "shared immutable env + per-task
scratch" split tsgo gets via per-checker state, with the sharing tsgo lacks.

`Type::Ref` points at a `SymbolId` (alias/interface/class/...), resolved
structurally on demand by `relate` with cycle guards. Constructs v0 does not
model (generics instantiation, conditional/mapped types, typeof, namespaces)
lower to `Type::Unsupported`.

## Relation semantics (`check/relate.rs`)

`relate` returns a tri-state: `True` / `False` / `Unknown`. Diagnostics are
emitted **only on `False`** — anything touching an unmodeled construct returns
`Unknown` and stays silent. v0 is therefore sound-for-what-it-reports: no
false positives by construction, growing coverage by moving constructs out of
`Unsupported`.

## v0 scope

Reported: TS2307 (module not found), TS2305 (no exported member), TS1192 (no
default export), TS2322 (not assignable: variable init, return statements),
TS1360 (satisfies), plus all forced isolated-declarations violations and parse
errors.

Not yet modeled (lower to `Unsupported` → `Unknown` → silent): generics
instantiation, conditional/mapped/indexed/typeof types, lib.d.ts globals,
declaration merging, `declare global` / augmentations, namespaces, `export =`,
function assignability, narrowing, excess-property checks, JS/checkJs.

Narrowing note: because flow narrowing is unmodeled, identifier-sourced
`return` checks are skipped entirely (a returned identifier may have been
narrowed). Variable initializers are only checked at straight-line module top
level, where no guard can narrow — the one residual false-positive vector is
top-level `asserts` assertion functions, accepted for v0.

## Growth path

- M1: lib.d.ts environment (the env builder already loads arbitrary `.d.ts`;
  needs symbol merging across lib files) + property access / call checking.
- M2: generics instantiation + real structural relations (port semantics from
  tsgo `relater.go`).
- M3: surface hashing → file-level incremental rechecking; LSP.
