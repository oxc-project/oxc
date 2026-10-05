use oxc_allocator::Allocator;
use oxc_ast::{AstKind, AstType, ast::Program};
use oxc_ast_visit::Visit;
use oxc_comment_assignment::CommentAssignment;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::node::NodeId;

fn parse<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Program<'a> {
    let ret = Parser::new(allocator, source_text, source_type).parse();
    assert!(ret.diagnostics.is_empty(), "Parse errors: {:?}", ret.diagnostics);
    ret.program
}

#[derive(Default)]
struct NodeCollector {
    nodes: Vec<(AstType, NodeId)>,
}

impl<'a> Visit<'a> for NodeCollector {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.nodes.push((kind.ty(), kind.node_id()));
    }
}

fn collect_nodes(program: &Program<'_>) -> Vec<(AstType, NodeId)> {
    let mut collector = NodeCollector::default();
    collector.visit_program(program);
    collector.nodes
}

#[track_caller]
fn assert_dense_ids(program: &Program<'_>) -> Vec<(AstType, NodeId)> {
    let nodes = collect_nodes(program);
    assert_eq!(nodes.first(), Some(&(AstType::Program, NodeId::ROOT)));
    for (index, (kind, node_id)) in nodes.iter().enumerate() {
        assert_eq!(*node_id, NodeId::new(index), "Unexpected ID for {kind:?}");
    }
    nodes
}

#[test]
fn empty_program_has_root_id() {
    let allocator = Allocator::default();
    let program = parse(&allocator, "", SourceType::mjs());

    CommentAssignment::new().assign(&program);

    assert_eq!(collect_nodes(&program), [(AstType::Program, NodeId::ROOT)]);
}

#[test]
fn assigns_dense_ids_to_js_ts_and_jsx_nodes() {
    let cases: &[(&str, SourceType, &[AstType])] = &[
        (
            "function f([first, ...rest]) { return [first, , `${rest.length}`]; }",
            SourceType::mjs(),
            &[AstType::Function, AstType::Elision, AstType::TemplateElement],
        ),
        (
            "type Pair<T> = [left: T, right: T]; const pair: Pair<number> = [1, 2];",
            SourceType::ts(),
            &[AstType::TSTypeParameter, AstType::TSNamedTupleMember, AstType::TSTypeAnnotation],
        ),
        (
            "const element = <Component prop={value}><>{/* comment */}{value}</></Component>;",
            SourceType::jsx(),
            &[AstType::JSXElement, AstType::JSXFragment, AstType::JSXEmptyExpression],
        ),
        (
            "const element: JSX.Element = <Component<string> prop={value} />;",
            SourceType::tsx(),
            &[
                AstType::TSTypeAnnotation,
                AstType::TSTypeParameterInstantiation,
                AstType::JSXElement,
            ],
        ),
    ];

    for &(source_text, source_type, expected_kinds) in cases {
        let allocator = Allocator::default();
        let program = parse(&allocator, source_text, source_type);

        CommentAssignment::new().assign(&program);

        let nodes = assert_dense_ids(&program);
        for expected_kind in expected_kinds {
            assert!(
                nodes.iter().any(|(kind, _)| kind == expected_kind),
                "Missing {expected_kind:?} in {source_text}",
            );
        }
    }
}

#[test]
fn includes_hashbang_and_directive_nodes_without_comments() {
    let allocator = Allocator::default();
    let program =
        parse(&allocator, "#!/usr/bin/env node\n\"use strict\"; foo();", SourceType::mjs());
    assert!(program.comments.is_empty());

    CommentAssignment::new().assign(&program);

    let nodes = assert_dense_ids(&program);
    let kinds: Vec<_> = nodes.iter().map(|(kind, _)| *kind).collect();
    assert_eq!(
        kinds,
        [
            AstType::Program,
            AstType::Hashbang,
            AstType::Directive,
            AstType::StringLiteral,
            AstType::ExpressionStatement,
            AstType::CallExpression,
            AstType::IdentifierReference,
        ],
    );
}

#[test]
fn repeated_assignment_starts_from_root() {
    let allocator = Allocator::default();
    let program = parse(&allocator, "const value = input + 1;", SourceType::mjs());

    CommentAssignment::new().assign(&program);
    let first = assert_dense_ids(&program);

    program.node_id.set(NodeId::new(99));
    CommentAssignment::new().assign(&program);

    assert_eq!(assert_dense_ids(&program), first);
}

#[test]
fn assign_resets_ids_after_visiting_another_program() {
    let allocator = Allocator::default();
    let first = parse(&allocator, "function f() { return 1; }", SourceType::mjs());
    let second = parse(&allocator, "value;", SourceType::mjs());

    let mut assignment = CommentAssignment::new();
    assignment.visit_program(&first);
    let first_nodes = assert_dense_ids(&first);
    assignment.assign(&second);

    assert_eq!(assert_dense_ids(&second).len(), 3);
    assert_eq!(collect_nodes(&first), first_nodes);
}
