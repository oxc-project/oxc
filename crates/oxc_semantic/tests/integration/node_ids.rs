use oxc_allocator::Allocator;
use oxc_ast::{AstKind, AstType};
use oxc_ast_visit::{AstNodeIdAssigner, Visit};
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
