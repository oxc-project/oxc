# Oxc Comment Assignment

AST traversal for assigning comments to nodes, used automatically by the parser.

Parser-returned comments already have owners. For an AST that still matches its source text,
the standalone pass can recompute ownership using the existing node IDs:

```rust
use oxc_comment_assignment::CommentAssignment;

CommentAssignment::new().assign(&mut program);

for comment in &program.comments {
    let attachment = comment.attachment.unwrap();
    // Use attachment.node_id and attachment.placement to find the owner.
}
```

The pass accepts a mutable Program reference and stores attachments directly on its comments. The AST must have unique node IDs, with ID `0` reserved for Program, as provided by the parser or semantic analysis. Each source comment stores its owner's ID and leading, trailing, or dangling placement in `Comment::attachment`. The parser runs this pass before returning, so every parser-returned comment already has an attachment. Semantic analysis preserves ownership when replacing node IDs.

Running the pass preserves node IDs and existing semantic data. Programs without comments require no traversal.

The standalone pass can recompute ownership on an AST that still matches its source text. After transformations, use ordinary semantic analysis to remap existing owners rather than reassignment from source positions. Standalone lexer comments, manually constructed comments, and clones without node IDs can remain unassigned.
