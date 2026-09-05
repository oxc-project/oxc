use oxc_allocator::Allocator;
use oxc_ast::{AstKind, AstType};
use oxc_ast_visit::{AstNodeIdAssigner, CommentAttachmentBuilder, Visit};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_syntax::node::NodeId;

#[test]
fn standalone_node_ids_match_semantic_node_ids() {
    let allocator = Allocator::default();
    let source = r#"
        import value from "module";

        @decorator
        export class C<T> extends Base<T> {
            #value = value;

            method<U>(argument: U): T {
                return this.#value satisfies T;
            }
        }

        const element = <C<string> />;
    "#;
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    assert!(parsed.diagnostics.is_empty());

    let standalone_count = AstNodeIdAssigner::assign(&parsed.program);
    let standalone_ids = collect_node_ids(&parsed.program);

    let semantic = SemanticBuilder::new_compiler().build(&parsed.program);
    assert!(semantic.diagnostics.is_empty());
    let semantic_ids = collect_node_ids(&parsed.program);

    assert_eq!(standalone_count as usize, standalone_ids.len());
    assert_eq!(semantic.semantic.stats().nodes, standalone_count);
    assert_eq!(semantic_ids, standalone_ids);
}

#[test]
fn semantic_comment_attachments_match_standalone_attachments() {
    let allocator = Allocator::default();
    let source = concat!(
        "/* before */ const value = [/* first */ 1 /* after */, /* close */];\n",
        "function /* name */ f(/* empty */) { /* body */ }\n",
    );
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());

    let standalone = CommentAttachmentBuilder::build(&parsed.program);
    let semantic =
        SemanticBuilder::new_compiler().with_comment_attachments(true).build(&parsed.program);

    assert!(semantic.diagnostics.is_empty());
    assert_eq!(semantic.comment_attachments.as_ref(), Some(&standalone));
}

#[test]
fn semantic_comment_attachments_are_opt_in() {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, "/* comment */ value;", SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());

    let semantic = SemanticBuilder::new_compiler().build(&parsed.program);

    assert!(semantic.comment_attachments.is_none());
}

#[test]
fn semantic_rebuild_rehomes_surviving_comments_and_discards_removed_comments() {
    let allocator = Allocator::default();
    let source = "/* first */ first();\n/* second */ second();\n";
    let mut parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());

    let mut attachments = CommentAttachmentBuilder::build(&parsed.program);
    parsed.program.body.swap(0, 1);

    let rebuilt = SemanticBuilder::new_compiler()
        .build_and_rehome_comments(&parsed.program, &mut attachments);
    assert!(rebuilt.diagnostics.is_empty());
    assert_eq!(comment_indices(attachments.comments_for(parsed.program.body[1].node_id())), [0]);
    assert_eq!(comment_indices(attachments.comments_for(parsed.program.body[0].node_id())), [1]);
    drop(rebuilt);

    parsed.program.body.remove(1);
    let rebuilt = SemanticBuilder::new_compiler()
        .build_and_rehome_comments(&parsed.program, &mut attachments);
    assert!(rebuilt.diagnostics.is_empty());
    assert_eq!(comment_indices(attachments.comments_for(parsed.program.body[0].node_id())), [1]);
    assert_eq!(attachments.len(), 1);
}

#[test]
fn semantic_rebuild_does_not_rehome_program_comments_to_dummy_nodes() {
    let allocator = Allocator::default();
    let mut parsed = Parser::new(&allocator, "/* program */", SourceType::mjs()).parse();
    let mut generated = Parser::new(&allocator, "generated();", SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());
    assert!(generated.diagnostics.is_empty());

    let mut attachments = CommentAttachmentBuilder::build(&parsed.program);
    parsed.program.body.push(generated.program.body.pop().unwrap());

    let rebuilt = SemanticBuilder::new_compiler()
        .build_and_rehome_comments(&parsed.program, &mut attachments);
    assert!(rebuilt.diagnostics.is_empty());
    assert_eq!(comment_indices(attachments.comments_for(NodeId::ROOT)), [0]);
    assert!(attachments.comments_for(parsed.program.body[0].node_id()).is_empty());
}

fn comment_indices(comments: &[oxc_ast_visit::AttachedComment]) -> Vec<u32> {
    comments.iter().map(|comment| comment.comment_index).collect()
}

fn collect_node_ids<'a>(program: &'a oxc_ast::ast::Program<'a>) -> Vec<NodeRecord> {
    let mut collector = NodeIdCollector::default();
    collector.visit_program(program);
    collector.nodes
}

#[derive(Debug, PartialEq, Eq)]
struct NodeRecord {
    id: NodeId,
    ty: AstType,
    span: Span,
}

#[derive(Default)]
struct NodeIdCollector {
    nodes: Vec<NodeRecord>,
}

impl<'a> Visit<'a> for NodeIdCollector {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.nodes.push(NodeRecord { id: kind.node_id(), ty: kind.ty(), span: kind.span() });
    }
}
