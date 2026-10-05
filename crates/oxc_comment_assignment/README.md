# Oxc Comment Assignment

AST traversal for assigning comments to nodes.

The current implementation assigns dense node IDs in preparation for comment ownership. Run it after parsing:

```rust
use oxc_comment_assignment::CommentAssignment;

CommentAssignment::new().assign(&program);
```

The pass accepts a shared Program reference and updates its node ID cells. Program receives ID `0`; other IDs follow AST visitor preorder.

Running the pass renumbers nodes. Rebuild existing semantic data keyed by node IDs afterward.
