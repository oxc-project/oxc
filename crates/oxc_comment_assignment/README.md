# Oxc Comment Assignment

AST traversal for assigning comments to nodes.

Run the pass after parsing to attach every source comment using the existing node IDs:

```rust
use oxc_comment_assignment::CommentAssignment;

CommentAssignment::new().assign(&mut program);

for comment in &program.comments {
    let attachment = comment.attachment.unwrap();
    // Use attachment.node_id and attachment.placement to find the owner.
}
```

The pass accepts a mutable Program reference and stores attachments directly on its comments. The AST must have unique node IDs, with ID `0` reserved for Program, as provided by the parser or semantic analysis. Each source comment stores its owner's ID and leading, trailing, or dangling placement in `Comment::attachment`. Parsing leaves attachments unset.

Running the pass preserves node IDs and existing semantic data. Programs without comments require no traversal.
