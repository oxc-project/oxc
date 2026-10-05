# Oxc Comment Assignment

AST traversal for assigning comments to nodes, used automatically by the parser.

Parser-returned comments already have owners. For an AST that still matches its source text,
the standalone pass can assign unattached comments using the existing node IDs:

```rust
use oxc_comment_assignment::assign_remaining;

assign_remaining(&mut program);

for comment in &program.comments {
    // Use comment.node_id and comment.placement to find the owner.
}
```

The pass accepts a mutable Program reference and stores attachments directly on its comments. The AST must have unique node IDs, with ID `0` reserved for Program, as provided by the parser or semantic analysis. Each source comment stores its owner's ID and leading, trailing, or dangling placement in `Comment::node_id` and `Comment::placement`. The parser runs this pass before returning, so every parser-returned comment already has an attachment. Semantic analysis preserves ownership when replacing node IDs.

Running the pass preserves existing attachments, node IDs, and semantic data. Programs without unattached comments require no traversal.

After transformations, use ordinary semantic analysis to remap existing owners. Standalone lexer comments, manually constructed comments, and clones without node IDs use `Comment::UNASSIGNED_NODE_ID` until ownership is assigned.
