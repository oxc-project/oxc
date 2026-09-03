# Comment Preservation Architecture

## Problem

The parser records comments relative to token boundaries. Codegen prints AST nodes. Those are different coordinate systems:

- some token boundaries, such as `function`, generator `*`, commas, and closing delimiters, do not have their own AST node;
- transforms can move, replace, clone, or remove nodes;
- node spans and output order are not guaranteed to remain in global source order;
- comment annotations have semantics beyond their text;
- querying a source-offset index at every AST boundary is too expensive for codegen.

The architecture must bridge the two coordinate systems once, before codegen, and give every printable source comment one stable owner.

```text
lexer/parser comments
        │
        │ token-relative spans and flags
        ▼
attachment collector
        │ runs during either a lightweight AST-indexing walk
        │ or semantic traversal
        │
        │ NodeId + placement, grouped by NodeId
        ▼
transforms preserve or deliberately replace NodeIds
        │
        ▼
codegen claims comments while printing existing nodes
```

## Goals

1. Print every enabled parser-produced comment exactly once.
2. Keep comments near their semantic host when nodes move.
3. Preserve PURE, NO_SIDE_EFFECTS, legal, coverage, and property-key annotation behavior.
4. Remain idempotent across parse/transform/codegen cycles.
5. Avoid a source-offset lookup or range scan on every codegen node.
6. Avoid assumptions about global node, span, or output order.
7. Add no cost when comments are disabled or codegen is not requested.

“Exactly once” is the primary correctness invariant. Exact original whitespace is not. Placement should be locally faithful where the AST exposes enough structure and deterministic otherwise.

## Non-goals  

- Reconstructing the original token stream.
- Preserving arbitrary whitespace around comments.
- Making ESTree or JavaScript codegen consume attached comments in this change.
- Making source offsets the runtime identity of a comment host.

## Core invariants

### One comment, one owner

A printable comment has one entry in one ownership table. Codegen must not maintain a NodeId copy and a source-offset copy of the same comment.

Dual ownership is fundamentally unsafe: a local source-gap emitter can consume one copy while a later NodeId boundary consumes the other. Attempts to synchronize two stores after every claim are both error-prone and costly.

### NodeId is the host identity

AST addresses are not stable enough to be comment identities. Arena addresses can change across clones and reconstructed nodes, and an address map cannot express deliberate identity preservation through transforms.

The attachment pass uses semantic `NodeId` values. Node IDs used as comment hosts must obey this transform contract:

- moving an existing node preserves its `NodeId`;
- replacing a node while preserving its semantic role may transfer its `NodeId` deliberately;
- synthesized nodes receive fresh IDs and inherit no source comments by default;
- cloning a node does not duplicate its source comments by default;
- removing a node leaves its comments unclaimed for fallback unless the transform explicitly rehomes them;
- a generated node must never accidentally reuse a removed source node's ID.

Span equality must not be required for a valid claim: moving a node or regenerating its output can make spans stale. Spans remain comment metadata and fallback ordering keys, not host identity.

### Source offsets stop at the attachment boundary

`Comment.attached_to` is an input to the semantic attachment pass. Normal codegen placement must not query it.

Once attachment is built, codegen asks only:

```rust,ignore
comments.take_before(node_id)
comments.take_after(node_id)
comments.take_inside(node_id)
```

There is no monotonic source cursor and no generic `take_between(start, end)` in the codegen hot path.

### Claims are destructive

Each comment is claimable once. A successful claim clears its presence bit or marks its sidecar entry consumed. Annotation recovery and ordinary printing use the same claim operation.

## Data model

Parser comments remain the source of truth for text, kind, annotation content, and newline flags.

The semantic pass produces a compact sidecar owned with the `Program`:

```rust,ignore
struct CommentAttachments<'a> {
    // Dense prefix sums indexed by NodeId.
    host_offsets: Box<'a, [u32]>,
    comments: Box<'a, [AttachedComment]>,
}

struct AttachedComment {
    comment: Comment,
    placement: CommentPlacement,
    same_line: bool,
}

enum CommentPlacement {
    Before,
    After,
    Inside,
}
```

`host_offsets[node_id]..host_offsets[node_id + 1]` is the node's comment range. A presence bitset lets comment-free nodes return immediately without touching the range.

Comments within a host are kept in source order. Leading and trailing claims are independent views over that one range; claiming one placement does not drain another.

### Three placements are sufficient

`Before`, `After`, and `Inside` are the complete placement model. Token-only boundaries do not create additional attachment categories.

When punctuation has no AST node, the attachment pass collapses the comment onto the nearest meaningful AST host:

- a comment before the next child is `Before` that child;
- a same-line comment after the previous child is `After` that child;
- a comment after the final child is `After` that child;
- a comment in a childless container is `Inside` the container.

For example, comments among `function`, generator `*`, a function name, and its parameter list are attached before or after the nearest identifier or parameter node. They remain in source order, but codegen does not attempt to recreate every original token gap.

Similarly, comments around commas and closing delimiters belong to the adjacent child. If the container is empty, they belong `Inside` the container.

## Attachment pass

Attachment is opt-in. It runs in one of two modes:

1. A lightweight AST-indexing walk assigns dense `NodeId`s and collects attachments without scopes, symbols, references, or CFG construction.
2. Semantic construction collects the same attachments while it assigns the same dense `NodeId`s during its existing traversal.

Parse-only consumers which do not request attachment run neither pass. If the standalone attachment pass is followed by semantic construction without an intervening structural AST change, semantic construction deterministically assigns the same IDs.

### Inputs

- completed AST;
- semantic `NodeId` values;
- parser comments in source order;
- source text for same-line classification;
- parser token-boundary metadata already stored on each comment.

### Assignment rules

The default depth-first gap rules are:

1. Before the first child: `Before` the child.
2. On the same line after a child: `After` the child.
3. Between siblings on separate lines: `Before` the next child.
4. After the final child: `After` the final child.
5. Parent with no children: `Inside` the parent.

No second placement system is introduced for composite nodes. Functions, lists, parameters, arguments, object/class bodies, TypeScript lists, and JSX containers use the same three rules.

Annotations may override the default host only when their semantics require it. For example, a PURE comment which did not apply remains an ordinary trailing comment on the preceding host; it must not be recovered later as an applied PURE annotation.

### Complexity

The standalone path must visit every AST node to assign dense `NodeId`s. The semantic path reuses the traversal it already performs. In both modes, expensive comment classification should be limited to subtrees intersecting comments.

- Sort and deduplicate comment anchors once if the lexer did not already produce monotonic order.
- Maintain an interval query over comment spans and anchors.
- Skip comment-classification work for a subtree when neither a comment span nor an anchor intersects its span; the standalone path still visits its nodes to assign IDs.
- Record ancestry and sibling-gap data only for nodes required to resolve a comment.
- Group assignments by `NodeId` with counting sort and prefix sums.

The standalone mode is `O(V + C log C)`, where `V` is the number of AST nodes and `C` is the number of comments. The semantic mode adds `O(C log C + Vc)` work to its existing traversal, where `Vc` is the number of nodes in subtrees intersecting comments. Comment-free files skip attachment entirely.

## Why the sidecar lives on Program

The sidecar describes ownership within one concrete AST and must travel with that AST through transforms and codegen. `ParserReturn` is only a construction result and is commonly destructured or discarded before transformation.

Arena ownership on `Program` also keeps lifetimes simple and avoids a separate object that callers must manually keep synchronized.

The sidecar is not serialized to ESTree and is ignored by structural AST equality.

## Transform contract

Most comments follow transformed code without transform-specific logic because they follow the preserved `NodeId`.

Transforms need only handle identity-changing operations deliberately:

```rust,ignore
comments.rehome(from_id, to_id);
comments.rehome_before(from_id, next_id);
comments.orphan(from_id);
```

These operations move ownership; they never copy it.

If a transform removes a host and does nothing, its comments remain unclaimed. That is valid and is handled by safe fallback. High-value transforms can rehome comments for better placement without changing the fundamental model.

## Codegen integration

### Generic boundaries

`Gen::print` and `GenExpr::print_expr` are the central owners of ordinary boundary emission:

```rust,ignore
print_before(node_id);
node.gen(...);
print_after(node_id);
```

Calls which intentionally bypass these wrappers with direct `r#gen` must be audited. A bypass either delegates ownership to its enclosing enum or claims the concrete node explicitly; it must not silently skip comments.

### Composite printers

Composite printers only need explicit handling for `Inside` comments belonging to the composite node they are already printing. Important initial sites are:

- function headers;
- parameter and argument lists;
- array, object, class, TypeScript, and JSX delimiters;
- binary/logical operand boundaries;
- separators and closing delimiters.

Comments around non-empty children are emitted by those children's ordinary `Before` and `After` boundaries. Separators, operators, function-header punctuation, and closing delimiters do not perform independent comment lookup.

No composite printer scans arbitrary source ranges. The attachment pass has already answered which comments belong there.

### Annotation printing

PURE and NO_SIDE_EFFECTS recovery uses the same sidecar as ordinary comments:

```rust,ignore
comments.take_matching(node_id, CommentContent::Pure)
```

Only a matching annotation kind is consumed. Sibling comments at the same host remain available. Canonical annotation text is emitted only for a synthesized annotated node with no recoverable source comment.

### CJS lexer fast paths

CJS module lexer fast paths have output-shape constraints. They must either claim only comments proven safe for that fast path or deliberately leave the comments to their enclosing owner. Adding generic comment emission inside those paths is not assumed safe.

## Fallback

Fallback exists for removed hosts. It is not a normal placement mechanism.

1. After a statement reaches a syntactically safe terminator, codegen may emit still-unclaimed comments owned by removed descendants of that statement.
2. At `Program` completion, remaining comments are emitted in original comment order.
3. Relocated leftovers are normalized onto separate lines. This keeps reparsing from converting a leading JSDoc or annotation into a trailing comment.

Fallback must not depend on a global source cursor or on current output order.

Annex B HTML-like line comments are a deliberate exception. A trailing `<!--` or `-->` cannot safely be relocated outside its original line context. If its host disappears and it cannot be emitted at that host's boundary, codegen consumes it without relocation.

## Comment options

Filtering happens before attachment storage is built for codegen:

- disabled comments: build no attachment store;
- normal/JSDoc/legal/annotation options retain only enabled categories;
- legal-comment EOF/linked/external modes continue to use their configured destination;
- minified comments-disabled paths pay no per-node comment cost.

The attachment collector may still be reused by another consumer, but codegen's claim table contains only comments it can print.

## Performance requirements

The feature is not ready if comment preservation dominates codegen time.

Required properties:

- no hash lookup at every node;
- no binary search at every node;
- no repeated scan over source anchors;
- one presence-bit test for the common comment-free node;
- attachment disabled when codegen is absent;
- semantic and React Compiler-only pipelines do not pay attachment cost;
- no duplicated comment buffers proportional to the source comment count.

Performance validation must include `kitchen-sink.tsx`, `App.tsx`, `binder.ts`, and `react.development.js`. A change which fixes conformance by reintroducing a double-digit codegen regression is rejected.

## Testing strategy

### Exact ownership

Every test comment receives a unique marker. Assert that each enabled marker appears exactly once, not merely that output contains it.

### Placement coverage

- repeated comments on one host;
- mixed leading and trailing comments on one boundary;
- function keyword/generator/name/parameter gaps;
- empty containers and comments before closing delimiters;
- parameters, arguments, bindings, expressions, and binary operands;
- object/class members;
- TypeScript syntax and JSX;
- PURE, NO_SIDE_EFFECTS, property-key, legal, and coverage annotations;
- CJS fast paths;
- Annex B HTML comments.

### Transform coverage

- moved and reordered nodes;
- replaced nodes which preserve identity;
- cloned nodes which must not duplicate comments;
- removed hosts and deterministic fallback;
- stale spans;
- synthesized nodes with and without canonical annotations.

### Stability gates

- parse/codegen idempotency;
- transformer idempotency;
- minifier idempotency;
- comments-disabled minified output;
- source maps;
- `oxc_codegen`, `oxc_minifier`, `oxc_isolated_declarations`, transformer fixtures, and full coverage snapshots;
- CodSpeed comparison against current `main`.

Coverage snapshots must not be updated merely to encode new mismatches. Each changed mismatch needs an explanation and an intentional expected behavior.

## Implementation sequence

1. Establish a clean coverage and performance baseline against `main`.
2. Implement standalone dense `NodeId` assignment and verify its IDs match semantic construction.
3. Finalize the sidecar and shared attachment collector without changing codegen output.
4. Integrate the collector with the standalone ID walk and semantic traversal; benchmark both modes independently.
5. Switch generic codegen boundaries to the single sidecar owner.
6. Add `Inside` handling to composite delimiters, deleting the corresponding offset-based emitter each time.
7. Integrate annotations with the same claim table.
8. Add safe statement and Program fallback.
9. Remove the remaining generic source-range store and dual-ownership synchronization.
10. Run all focused suites, full coverage, allocation snapshots required by parser/semantic changes, and CodSpeed.

Each step must leave tests green. Broad ownership-policy changes and snapshot updates must not be combined in one iteration.

## Current branch audit

The current implementation is transitional. It contains both NodeId-based ownership and source-offset recovery paths, plus synchronization between them. That violates the intended single-owner invariant and makes correctness depend on which emitter reaches a comment first.

Before further feature work, the implementation should be brought back to a clean coverage baseline and then migrated according to the sequence above. New special cases should use the same three placements or explicit transform rehoming, not another independent lookup path in codegen.
