use oxc::{
    ast::{Comment as AstComment, CommentPlacement},
    span::Span,
};
use oxc_estree::{
    CompactSerializer, ESTree, JsonSafeString, NodeCommentPlacement, NodeComments, Serializer,
    StructSerializer,
};
use oxc_napi::Comment;

/// Group comments by the node which owns them, for the `experimentalNodeComments` option.
///
/// `comments` are `ast_comments` converted to NAPI comments, in the same order.
pub fn build_node_comments(
    ast_comments: &[AstComment],
    comments: &[Comment],
    ranges: bool,
) -> NodeComments {
    let mut node_comments = NodeComments::new();
    for (ast_comment, comment) in ast_comments.iter().zip(comments) {
        let placement = match ast_comment.placement {
            CommentPlacement::Leading => NodeCommentPlacement::Leading,
            CommentPlacement::Trailing => NodeCommentPlacement::Trailing,
            CommentPlacement::Dangling => NodeCommentPlacement::Dangling,
        };
        let mut serializer = CompactSerializer::new(false, ranges);
        CommentJson(comment).serialize(&mut serializer);
        node_comments.push(ast_comment.node_id.get().index(), placement, serializer.into_string());
    }
    node_comments
}

struct CommentJson<'c>(&'c Comment);

impl ESTree for CommentJson<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let comment = self.0;
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString(&comment.r#type));
        state.serialize_field("value", &comment.value);
        state.serialize_span(Span::new(comment.start, comment.end));
        state.end();
    }
}
