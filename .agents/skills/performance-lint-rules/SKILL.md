---
name: performance-lint-rules
description: Performance review guidance for Oxc linter rule implementations. Use only when reviewing Rust rule code under crates/oxc_linter/src/rules/ or when explicitly auditing those rules for performance improvements.
---

## Performance Guidelines

### Prefer top-level node kind checks

Put node kind checks at the rule entry point. If a rule only handles a few syntactic forms, start `run` with an `AstKind` match and return for all other nodes, even when a helper filters again internally. This lets lintgen derive narrower `NODE_TYPES` and avoids dispatching the rule on unrelated AST nodes.

After changing the relevant node kinds for a rule, regenerate the rule runner with `cargo lintgen` and consider adding or updating `assert_rule_runs_on_node_types` coverage in `crates/oxc_linter/src/rule.rs`.

Implement only the needed entry point. If a rule is a whole-file pass over semantic indexes, use `run_once` by itself. Implementing both `run` and `run_once` prevents useful node-type narrowing.

### Do cheaper checks first

Order checks from cheapest and most selective to most expensive. Return quickly for common non-matches before doing semantic lookups, allocations, or deeper traversal.

- Matching a small fixed string set with `matches!` before semantic checks.
- Rejecting lowercase identifiers before global-object checks when only constructors can match.
- Checking whether a JSX attribute starts with `aria-` before lowercasing it.
- Checking for required syntax such as a `key` prop before looking up callback parameter symbols.
- Checking `source_range(span).contains("this")` before running a visitor that only finds `this`.

### Delay expensive context

Most files do not contain lint errors. Do not prepare diagnostics, labels, help text, fix data, ancestors, symbols, JSX element types, or replacement strings until the rule has found a syntactic candidate that could actually report.

### Iterate over the smallest set possible

- Use `run_once` when the rule only needs a whole-file pass and does not need to run on every node.
- Iterate over symbols instead of AST nodes when looking for references to specific names.
- Prefer targeted lists or semantic data over broad AST traversal when available.
- For name-based binding checks, use `ctx.scoping().get_binding(scope_id, name)` instead of scanning every binding in `get_bindings(scope_id)`.
- For global or unresolved identifier checks, start from `ctx.scoping().root_unresolved_references().get(name)` for the small set of relevant names instead of visiting every `IdentifierReference`.
- When iterating unresolved references, still verify the reference is the right kind: skip references with a symbol, type-only references, and nodes whose `AstKind` is not the expected identifier or member access.
- When checking imported specifiers or exported names, iterate the concrete specifiers or precomputed export set rather than scanning all root bindings for every item.

Use precomputed `FxHashSet`s only when many symbols need the same membership test. Prefer keyed semantic lookup when each lookup already has an exact name.

### Avoid unnecessary regular expressions

Avoid regular expressions when byte or string checks are enough: `contains`, `starts_with`, `ends_with`, or matching a small fixed set.

For hot comment or string scanning paths, prefer a cheap `memchr` or byte search to reject most inputs, then parse only candidates. Preserve regex semantics when replacing one, especially identifier boundaries, optional prefixes, and multiline whitespace.

### Avoid heap allocations

- Use copy-on-write utilities when a value usually does not need to change.
- Avoid intermediate `Vec`s and `String`s when iteration or borrowed data is enough.
- Keep temporary data on the stack when practical.
- Delay allocation until a diagnostic, fix, or transformed value is actually needed.
- Use allocation-free ASCII comparisons such as `starts_with_ignore_case` before calling `cow_to_ascii_lowercase`.
- Use byte scans such as `as_bytes().array_windows()` for simple ASCII patterns like escape sequences.
- Reserve hash maps or sets when the final size is known.
- Avoid building a `HashSet` just to check names that can be looked up directly in scoping data.
