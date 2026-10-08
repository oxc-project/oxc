use std::cell::Cell;

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::{
    AstKind, AstType, CommentAttachment, CommentContent, CommentPlacement, ast::Program,
};
use oxc_ast_visit::Visit;
use oxc_comment_assignment::CommentAssignment;
use oxc_parser::{ParseOptions, Parser, config::RuntimeParserConfig};
use oxc_span::{ContentEq, GetSpan, SourceType, Span};
use oxc_syntax::node::NodeId;

fn parse<'a>(allocator: &'a Allocator, source: &'a str, source_type: SourceType) -> Program<'a> {
    let ret = Parser::new(allocator, source, source_type).parse();
    assert!(ret.diagnostics.is_empty(), "Parse errors: {:?}", ret.diagnostics);
    let program = ret.program;
    let nodes = nodes(&program);
    for comment in &program.comments {
        let owner = comment
            .attachment
            .as_ref()
            .expect("Parser returned an unassigned comment")
            .node_id
            .get();
        assert!(nodes.iter().any(|node| node.id == owner));
    }
    program
}

struct Node {
    id: NodeId,
    kind: AstType,
    span: Span,
    pure: bool,
}

#[derive(Default)]
struct Nodes(Vec<Node>);

impl<'a> Visit<'a> for Nodes {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let pure = match kind {
            AstKind::CallExpression(node) => node.pure,
            AstKind::NewExpression(node) => node.pure,
            AstKind::Function(node) => node.pure,
            AstKind::ArrowFunctionExpression(node) => node.pure,
            _ => false,
        };
        self.0.push(Node { id: kind.node_id(), kind: kind.ty(), span: kind.span(), pure });
    }
}

fn nodes(program: &Program<'_>) -> Vec<Node> {
    let mut nodes = Nodes::default();
    nodes.visit_program(program);
    nodes.0
}

#[track_caller]
fn assign(program: &mut Program<'_>) {
    let before: Vec<_> = nodes(program).iter().map(|node| node.id).collect();
    CommentAssignment::new().assign(program);
    let nodes = nodes(program);
    assert_eq!(nodes.iter().map(|node| node.id).collect::<Vec<_>>(), before);
    for comment in &program.comments {
        let entry = comment.attachment.as_ref().expect("Unassigned comment");
        assert!(nodes.iter().any(|node| node.id == entry.node_id.get()));
    }
}

fn attachments(program: &Program<'_>) -> Vec<CommentAttachment> {
    program.comments.iter().map(|comment| comment.attachment.as_ref().unwrap().clone()).collect()
}

#[track_caller]
fn attachment(program: &Program<'_>, source: &str) -> CommentAttachment {
    program
        .comments
        .iter()
        .find(|comment| {
            &program.source_text[comment.span.start as usize..comment.span.end as usize] == source
        })
        .unwrap_or_else(|| panic!("Comment not found: {source}"))
        .attachment
        .as_ref()
        .unwrap()
        .clone()
}

#[track_caller]
fn node_id(program: &Program<'_>, kind: AstType, source: &str) -> NodeId {
    nodes(program)
        .into_iter()
        .find(|node| {
            node.kind == kind
                && &program.source_text[node.span.start as usize..node.span.end as usize] == source
        })
        .unwrap_or_else(|| panic!("{kind:?} not found: {source}"))
        .id
}

#[test]
fn comment_free_programs_preserve_node_ids() {
    for source in ["", "const value = input;"] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        assert!(program.comments.is_empty());
    }
}

#[test]
fn comment_only_program_owns_all_comments() {
    let allocator = Allocator::default();
    let mut program =
        parse(&allocator, "// first\n/* second */\n/*! license */", SourceType::mjs());
    assign(&mut program);

    assert_eq!(program.comments.len(), 3);
    assert!(attachments(&program).iter().all(|entry| entry.node_id.get() == program.node_id.get()));
}

#[test]
fn hashbang_and_directive_nodes_preserve_ids_with_comments() {
    let allocator = Allocator::default();
    let mut program = parse(
        &allocator,
        "#!/usr/bin/env node\n\"use strict\";\n/* leading */ value; // trailing",
        SourceType::mjs(),
    );
    assign(&mut program);

    let statement = node_id(&program, AstType::ExpressionStatement, "value;");
    assert_eq!(
        attachments(&program),
        [
            CommentAttachment {
                node_id: Cell::new(statement),
                placement: CommentPlacement::Leading
            },
            CommentAttachment {
                node_id: Cell::new(statement),
                placement: CommentPlacement::Trailing
            },
        ],
    );
}

#[test]
fn program_frame_includes_comments_outside_a_narrow_program_span() {
    let allocator = Allocator::default();
    let source = "/* leading */ value; // trailing";
    let mut program = parse(&allocator, source, SourceType::mjs());
    let start = u32::try_from(source.find("value").unwrap()).unwrap();
    program.span = Span::new(start, start + 6);
    assign(&mut program);

    assert_eq!(
        attachment(&program, "/* leading */"),
        CommentAttachment {
            node_id: Cell::new(NodeId::ROOT),
            placement: CommentPlacement::Leading
        },
    );
    assert_eq!(
        attachment(&program, "// trailing"),
        CommentAttachment {
            node_id: Cell::new(node_id(&program, AstType::ExpressionStatement, "value;")),
            placement: CommentPlacement::Trailing,
        },
    );
}

#[test]
fn comment_free_function_bodies_preserve_neighbor_attachments() {
    let allocator = Allocator::default();
    let body = "value += left * right;\n".repeat(256);
    let source = format!(
        "/* first */\nfunction first() {{ {body} }} // first tail\n\
         /* second */\nfunction second() {{ {body} }} // second tail"
    );
    let mut program = parse(&allocator, &source, SourceType::mjs());
    let first_id = program.body[0].node_id();
    let second_id = program.body[1].node_id();
    assign(&mut program);

    assert_eq!(
        attachments(&program),
        [
            CommentAttachment {
                node_id: Cell::new(first_id),
                placement: CommentPlacement::Leading
            },
            CommentAttachment {
                node_id: Cell::new(first_id),
                placement: CommentPlacement::Trailing
            },
            CommentAttachment {
                node_id: Cell::new(second_id),
                placement: CommentPlacement::Leading
            },
            CommentAttachment {
                node_id: Cell::new(second_id),
                placement: CommentPlacement::Trailing
            },
        ],
    );
}

#[test]
fn sparse_comment_survives_frame_stack_growth() {
    let allocator = Allocator::default();
    let source = format!("{}/* leaf */ value;{}", "{\n".repeat(128), "}\n".repeat(128));
    let mut program = parse(&allocator, &source, SourceType::mjs());
    assign(&mut program);

    assert_eq!(
        attachment(&program, "/* leaf */"),
        CommentAttachment {
            node_id: Cell::new(node_id(&program, AstType::ExpressionStatement, "value;")),
            placement: CommentPlacement::Leading,
        },
    );
}

#[test]
fn sparse_nested_ternary_keeps_the_comment_in_the_inner_branch() {
    let allocator = Allocator::default();
    let mut program = parse(
        &allocator,
        "console.log(1);\nconsole.log(1);\nconsole.log(foo ? bar ? /* inner */ 0 : 1 : 2);",
        SourceType::mjs(),
    );
    assign(&mut program);
    let entry = attachment(&program, "/* inner */");
    assert_eq!(entry.node_id.get(), node_id(&program, AstType::NumericLiteral, "0"));
    assert_eq!(entry.placement, CommentPlacement::Leading);
}

#[test]
fn ancestors_resolve_their_gaps_after_children_claim_nested_comments() {
    let allocator = Allocator::default();
    let statement = "console.log(foo ? bar ? /* inner */ 0 : 1 : /* outer */ 2);";
    let source = format!("// leading\n{statement} // trailing");
    let mut program = parse(&allocator, &source, SourceType::mjs());
    assign(&mut program);
    let statement_id = node_id(&program, AstType::ExpressionStatement, statement);

    for (comment, owner, placement) in [
        ("// leading", statement_id, CommentPlacement::Leading),
        ("/* inner */", node_id(&program, AstType::NumericLiteral, "0"), CommentPlacement::Leading),
        ("/* outer */", node_id(&program, AstType::NumericLiteral, "2"), CommentPlacement::Leading),
        ("// trailing", statement_id, CommentPlacement::Trailing),
    ] {
        let entry = attachment(&program, comment);
        assert_eq!((entry.node_id.get(), entry.placement), (owner, placement));
    }
}

#[test]
fn leading_and_exact_trailing_comments_attach_to_outer_statements() {
    let allocator = Allocator::default();
    let mut program = parse(
        &allocator,
        "// leading\nfirst(); // trailing\n// next\nsecond();",
        SourceType::mjs(),
    );
    assign(&mut program);
    let first = node_id(&program, AstType::ExpressionStatement, "first();");
    let second = node_id(&program, AstType::ExpressionStatement, "second();");

    for (comment, expected_node, expected_placement) in [
        ("// leading", first, CommentPlacement::Leading),
        ("// trailing", first, CommentPlacement::Trailing),
        ("// next", second, CommentPlacement::Leading),
    ] {
        let entry = attachment(&program, comment);
        assert_eq!((entry.node_id.get(), entry.placement), (expected_node, expected_placement));
    }
}

#[test]
fn empty_delimiters_and_closing_gaps_keep_dangling_comments() {
    let cases = [
        ("const value = [/* inside */];", AstType::ArrayExpression, "[/* inside */]"),
        ("const value = {/* inside */};", AstType::ObjectExpression, "{/* inside */}"),
        ("function f(/* inside */) {}", AstType::FormalParameters, "(/* inside */)"),
        ("function f() {/* inside */}", AstType::FunctionBody, "{/* inside */}"),
        ("f(/* inside */);", AstType::CallExpression, "f(/* inside */)"),
        ("const value = [item, /* inside */];", AstType::ArrayExpression, "[item, /* inside */]"),
    ];

    for (source, kind, container) in cases {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        let entry = attachment(&program, "/* inside */");
        assert_eq!(entry.node_id.get(), node_id(&program, kind, container), "{source}");
        assert_eq!(entry.placement, CommentPlacement::Dangling, "{source}");
    }
}

#[test]
fn jsx_comments_belong_to_the_empty_expression() {
    for (source, source_type) in [
        ("const value = <>{/* inside */}</>;", SourceType::jsx()),
        ("const value: JSX.Element = <div>{/* inside */}</div>;", SourceType::tsx()),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, source_type);
        assign(&mut program);
        let empty = nodes(&program)
            .into_iter()
            .find(|node| node.kind == AstType::JSXEmptyExpression)
            .unwrap();
        let entry = attachment(&program, "/* inside */");
        assert_eq!(entry.node_id.get(), empty.id);
        assert_eq!(entry.placement, CommentPlacement::Dangling);
    }
}

#[test]
fn template_children_claim_comments_after_quasis_were_visited() {
    for (source, source_type, owner_kind) in [
        (
            "const value = `a${First /* one */ + X}b${Second /* two */ + Y}c`;",
            SourceType::mjs(),
            AstType::IdentifierReference,
        ),
        (
            "type Value = `a${First /* one */ | X}b${Second /* two */ | Y}c`;",
            SourceType::ts(),
            AstType::TSTypeReference,
        ),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, source_type);
        assign(&mut program);
        for (comment, owner) in [("/* one */", "First"), ("/* two */", "Second")] {
            let entry = attachment(&program, comment);
            assert_eq!(entry.node_id.get(), node_id(&program, owner_kind, owner), "{source}");
            assert_eq!(entry.placement, CommentPlacement::Trailing);
        }
    }
}

#[test]
fn overlapping_this_parameter_and_parameter_list_do_not_reassign_comments() {
    let allocator = Allocator::default();
    let mut program = parse(
        &allocator,
        "function f(this: /* context */ Context, /* value */ value: number) {}",
        SourceType::ts(),
    );
    assign(&mut program);

    let entry = attachment(&program, "/* context */");
    assert_eq!(entry.placement, CommentPlacement::Leading);
    assert_eq!(entry.node_id.get(), node_id(&program, AstType::TSTypeReference, "Context"),);
    let entry = attachment(&program, "/* value */");
    assert_eq!(entry.placement, CommentPlacement::Leading);
    assert_eq!(entry.node_id.get(), node_id(&program, AstType::FormalParameter, "value: number"),);
}

#[test]
fn decorators_before_export_keep_comments_in_the_decorator() {
    for source in [
        "@decorate(/* inside */ value)\nexport class C {}",
        "@decorate(/* inside */ value)\nexport default class C {}",
        "export @decorate(/* inside */ value) class C {}",
        "export default @decorate(/* inside */ value) class C {}",
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::ts());
        assign(&mut program);

        let entry = attachment(&program, "/* inside */");

        assert_eq!(
            entry.node_id.get(),
            node_id(&program, AstType::IdentifierReference, "value"),
            "{source}",
        );
        assert_eq!(entry.placement, CommentPlacement::Leading);
    }
}

#[test]
fn applied_annotations_attach_to_the_flagged_nodes() {
    for (source, owner_kind) in [
        ("/* #__PURE__ */ factory();", AstType::CallExpression),
        ("/* #__PURE__ */ new Factory();", AstType::NewExpression),
        ("/* #__NO_SIDE_EFFECTS__ */ function f() {}", AstType::Function),
        ("const f = /* #__NO_SIDE_EFFECTS__ */ () => 1;", AstType::ArrowFunctionExpression),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        let owner = nodes(&program)
            .into_iter()
            .find(|node| node.kind == owner_kind && node.pure)
            .unwrap_or_else(|| panic!("Annotation was not applied in {source}"));

        assert_eq!(attachments(&program)[0].node_id.get(), owner.id, "{source}");
        assert_eq!(attachments(&program)[0].placement, CommentPlacement::Leading);
    }
}

#[test]
fn property_key_annotations_attach_to_literals() {
    for (source, kind, literal) in [
        ("const value = /* @__KEY__ */ 'key';", AstType::StringLiteral, "'key'"),
        ("const value = /* @__KEY__ */ `key`;", AstType::TemplateLiteral, "`key`"),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        assert_eq!(attachments(&program)[0].node_id.get(), node_id(&program, kind, literal));
        assert_eq!(attachments(&program)[0].placement, CommentPlacement::Leading);
    }
}

#[test]
fn direct_import_magic_comments_are_dangling_on_the_import() {
    for source in [
        "import(/* webpackChunkName: 'chunk' */ 'module');",
        "import(/* @vite-ignore */ path);",
        "import(/* turbopackOptional: true */ 'module');",
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        let import = nodes(&program)
            .into_iter()
            .find(|node| node.kind == AstType::ImportExpression)
            .unwrap();
        assert_eq!(attachments(&program)[0].node_id.get(), import.id, "{source}");
        assert_eq!(attachments(&program)[0].placement, CommentPlacement::Dangling);
    }
}

#[test]
fn nested_magic_comments_are_not_captured_by_an_outer_import() {
    let allocator = Allocator::default();
    let mut program = parse(
        &allocator,
        "import(wrap(/* webpackChunkName: 'nested' */ 'module'));",
        SourceType::mjs(),
    );
    assign(&mut program);
    let owner = nodes(&program)
        .into_iter()
        .find(|node| node.id == attachments(&program)[0].node_id.get())
        .unwrap();
    assert_ne!(owner.kind, AstType::ImportExpression);
}

#[test]
fn file_coverage_comments_belong_to_program() {
    for source in [
        "/* v8 ignore file */ function f() {}",
        "/* istanbul ignore file */ const value = 1;",
        "f(); /* istanbul ignore file */\ng();",
        "f(); /* v8 ignore file */",
        "function f() { g(); /* istanbul ignore file */\nh(); }",
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        assert_eq!(attachments(&program)[0].node_id.get(), program.node_id.get());
        assert_eq!(attachments(&program)[0].placement, CommentPlacement::Leading);
    }
}

#[test]
fn source_comments_are_unchanged_and_repeated_assignments_are_identical() {
    let allocator = Allocator::default();
    let mut program = parse(
        &allocator,
        "// header\n/*#__PURE__*/ factory(); // tail\nconst key = /* @__KEY__ */ 'key';\n/*! license */",
        SourceType::mjs(),
    );
    let source_comments = program.comments.to_vec();

    let first = attachments(&program);
    assign(&mut program);
    assign(&mut program);

    assert_eq!(program.comments.as_slice(), source_comments.as_slice());
    assert_eq!(first, attachments(&program));
}

#[test]
fn leading_comments_keep_their_statement_owners_across_repeated_assignments() {
    for count in [1, 8, 9, 32] {
        let allocator = Allocator::default();
        let source = "/* leading */\nvalue;\n".repeat(count);
        let mut program = parse(&allocator, &source, SourceType::mjs());
        let source_comments = program.comments.to_vec();
        assert_eq!(program.comments.len(), count);

        assign(&mut program);
        let statements: Vec<_> = nodes(&program)
            .into_iter()
            .filter(|node| node.kind == AstType::ExpressionStatement)
            .map(|node| node.id)
            .collect();
        for (comment, node_id) in program.comments.iter().zip(statements) {
            assert_eq!(
                comment.attachment,
                Some(CommentAttachment {
                    node_id: Cell::new(node_id),
                    placement: CommentPlacement::Leading
                })
            );
        }
        let first = attachments(&program);
        assign(&mut program);

        assert_eq!(first, attachments(&program));
        assert!(program.comments.iter().zip(&source_comments).all(|(a, b)| a.content_eq(b)));
    }
}

#[test]
fn normal_comments_keep_their_ownership_before_annotation_comments() {
    for (source, annotation, normal_kind, normal_source, annotation_kind, annotation_source) in [
        (
            "// normal\n/*#__PURE__*/foo();",
            "/*#__PURE__*/",
            AstType::ExpressionStatement,
            "foo();",
            AstType::CallExpression,
            "foo()",
        ),
        (
            "const key =\n// normal\n/* @__KEY__ */ 'key';",
            "/* @__KEY__ */",
            AstType::StringLiteral,
            "'key'",
            AstType::StringLiteral,
            "'key'",
        ),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        let normal = attachment(&program, "// normal");
        let annotation = attachment(&program, annotation);

        assert_eq!(normal.node_id.get(), node_id(&program, normal_kind, normal_source));
        assert_eq!(normal.placement, CommentPlacement::Leading);
        assert_eq!(annotation.node_id.get(), node_id(&program, annotation_kind, annotation_source));
        assert_eq!(annotation.placement, CommentPlacement::Leading);
    }
}

#[test]
fn template_substitution_edges_attach_to_the_substitution_root() {
    for (source, source_type, root_kind, root_source) in [
        (
            "tag`a${/*before*/ x\n/*after*/}b${y}`;",
            SourceType::mjs(),
            AstType::IdentifierReference,
            "x",
        ),
        (
            "tag`a${/*before*/ x + z\n/*after*/}b${y}`;",
            SourceType::mjs(),
            AstType::BinaryExpression,
            "x + z",
        ),
        (
            "type Value = `a${/*before*/ X\n/*after*/}b${Y}`;",
            SourceType::ts(),
            AstType::TSTypeReference,
            "X",
        ),
        (
            "type Value = `a${/*before*/ X | Z\n/*after*/}b${Y}`;",
            SourceType::ts(),
            AstType::TSUnionType,
            "X | Z",
        ),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, source_type);
        assign(&mut program);
        let root = node_id(&program, root_kind, root_source);
        for (comment, placement) in
            [("/*before*/", CommentPlacement::Leading), ("/*after*/", CommentPlacement::Trailing)]
        {
            let entry = attachment(&program, comment);
            assert_eq!((entry.node_id.get(), entry.placement), (root, placement), "{source}");
        }
    }
}

#[test]
fn this_parameter_edges_survive_the_overlapping_parameter_list() {
    let allocator = Allocator::default();
    let mut program = parse(
        &allocator,
        "function f(/* this doc */ this: Context // this tail\n, value: number) {}",
        SourceType::ts(),
    );
    assign(&mut program);
    let this_parameter = node_id(&program, AstType::TSThisParameter, "this: Context");

    for (comment, placement) in [
        ("/* this doc */", CommentPlacement::Leading),
        ("// this tail", CommentPlacement::Trailing),
    ] {
        let entry = attachment(&program, comment);
        assert_eq!((entry.node_id.get(), entry.placement), (this_parameter, placement));
    }
}

#[test]
fn pure_annotations_follow_flagged_calls_through_wrappers() {
    for (source, source_type) in [
        ("/*#__PURE__*/ ((factory()));", SourceType::mjs()),
        ("/*#__PURE__*/ factory().value;", SourceType::mjs()),
        ("/*#__PURE__*/ factory?.().value;", SourceType::mjs()),
        ("/*#__PURE__*/ (new Factory()).value;", SourceType::mjs()),
        ("const value = /*#__PURE__*/ (factory() as Result)!;", SourceType::ts()),
        ("const value = /*#__PURE__*/ factory() satisfies Result;", SourceType::ts()),
        ("const value = /*#__PURE__*/ <Result>factory()!;", SourceType::ts()),
        ("/*#__PURE__*/ (factory()) + other();", SourceType::mjs()),
        ("/*#__PURE__*/ (factory()) && other();", SourceType::mjs()),
        ("/*#__PURE__*/ (factory()) ? left() : right();", SourceType::mjs()),
        ("/*#__PURE__*/ (factory()), other();", SourceType::mjs()),
        ("/*#__PURE__*/ (factory()).value = input;", SourceType::mjs()),
    ] {
        for preserve_parens in [true, false] {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, source, source_type)
                .with_options(ParseOptions { preserve_parens, ..ParseOptions::default() })
                .parse();
            assert!(parsed.diagnostics.is_empty(), "{source}: {:?}", parsed.diagnostics);
            let mut program = parsed.program;
            assert_eq!(program.comments[0].content, CommentContent::Pure, "{source}");
            assign(&mut program);
            let flagged: Vec<_> = nodes(&program).into_iter().filter(|node| node.pure).collect();
            assert_eq!(flagged.len(), 1, "{source}");

            assert_eq!(attachments(&program)[0].node_id.get(), flagged[0].id, "{source}");
            assert_eq!(attachments(&program)[0].placement, CommentPlacement::Leading);
        }
    }
}

#[test]
fn no_side_effects_annotations_follow_export_and_const_prefixes() {
    for (source, function_count) in [
        ("/*#__NO_SIDE_EFFECTS__*/ export function first() {}", 1),
        ("export default /*#__NO_SIDE_EFFECTS__*/ function first() {}", 1),
        ("/*#__NO_SIDE_EFFECTS__*/ export const first = () => 1, second = () => 2;", 2),
        ("/*#__NO_SIDE_EFFECTS__*/ const first = function() {}, second = function() {};", 2),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assert_eq!(program.comments[0].content, CommentContent::NoSideEffects, "{source}");
        assign(&mut program);
        let functions: Vec<_> = nodes(&program)
            .into_iter()
            .filter(|node| {
                matches!(node.kind, AstType::Function | AstType::ArrowFunctionExpression)
            })
            .collect();
        assert_eq!(functions.len(), function_count);
        assert!(functions[0].pure, "{source}");
        assert!(functions[1..].iter().all(|node| !node.pure), "{source}");

        assert_eq!(attachments(&program)[0].node_id.get(), functions[0].id, "{source}");
        assert_eq!(attachments(&program)[0].placement, CommentPlacement::Leading);
    }
}

#[test]
fn const_prefix_annotations_skip_functions_in_binding_patterns() {
    for declaration in ["const", "export const"] {
        let source = format!(
            "/*#__NO_SIDE_EFFECTS__*/ {declaration} {{ a = /*@__NO_SIDE_EFFECTS__*/ () => 1 }} = () => 2;"
        );
        let allocator = Allocator::default();
        let mut program = parse(&allocator, &source, SourceType::mjs());
        assert!(
            program
                .comments
                .iter()
                .all(|comment| { comment.content == CommentContent::NoSideEffects })
        );
        assign(&mut program);
        for (comment, function) in
            [("/*#__NO_SIDE_EFFECTS__*/", "() => 2"), ("/*@__NO_SIDE_EFFECTS__*/", "() => 1")]
        {
            let entry = attachment(&program, comment);
            assert_eq!(
                entry.node_id.get(),
                node_id(&program, AstType::ArrowFunctionExpression, function)
            );
            assert_eq!(entry.placement, CommentPlacement::Leading);
        }
    }
}

#[test]
fn nested_annotations_stay_with_their_own_targets() {
    for (source, kind, outer, inner) in [
        (
            "/*#__PURE__*/ outer(/*@__PURE__*/ inner());",
            AstType::CallExpression,
            "outer(/*@__PURE__*/ inner())",
            "inner()",
        ),
        (
            "/*#__NO_SIDE_EFFECTS__*/ function outer() { /*@__NO_SIDE_EFFECTS__*/ function inner() {} }",
            AstType::Function,
            "function outer() { /*@__NO_SIDE_EFFECTS__*/ function inner() {} }",
            "function inner() {}",
        ),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assign(&mut program);
        for (index, target) in [outer, inner].into_iter().enumerate() {
            let entry = program.comments[index].attachment.as_ref().unwrap();
            assert_eq!(entry.node_id.get(), node_id(&program, kind, target), "{source}");
            assert_eq!(entry.placement, CommentPlacement::Leading);
        }
    }
}

#[test]
fn no_side_effects_annotations_follow_parenthesized_functions() {
    for source in [
        "/*#__NO_SIDE_EFFECTS__*/ (() => 1)();",
        "const value = /*#__NO_SIDE_EFFECTS__*/ (() => 1).value;",
        "const value = /*#__NO_SIDE_EFFECTS__*/ (() => 1) || fallback;",
    ] {
        for preserve_parens in [true, false] {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, source, SourceType::mjs())
                .with_options(ParseOptions { preserve_parens, ..ParseOptions::default() })
                .parse();
            assert!(parsed.diagnostics.is_empty(), "{source}: {:?}", parsed.diagnostics);
            let mut program = parsed.program;
            assert_eq!(program.comments[0].content, CommentContent::NoSideEffects, "{source}");
            assign(&mut program);
            assert_eq!(
                attachments(&program)[0].node_id.get(),
                node_id(&program, AstType::ArrowFunctionExpression, "() => 1"),
                "{source}",
            );
            assert_eq!(attachments(&program)[0].placement, CommentPlacement::Leading);
        }
    }
}

#[test]
fn unapplied_annotations_do_not_move_to_later_flagged_nodes() {
    for (source, unapplied, applied) in [
        (
            "/*#__PURE__*/ value;\n/*#__PURE__*/ factory();",
            CommentContent::PureNotApplied,
            CommentContent::Pure,
        ),
        (
            "/*#__NO_SIDE_EFFECTS__*/ value;\n/*#__NO_SIDE_EFFECTS__*/ function later() {}",
            CommentContent::NoSideEffectsNotApplied,
            CommentContent::NoSideEffects,
        ),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, SourceType::mjs());
        assert_eq!(program.comments[0].content, unapplied);
        assert_eq!(program.comments[1].content, applied);
        assign(&mut program);
        let flagged = nodes(&program).into_iter().find(|node| node.pure).unwrap();
        let before = program.comments[0].attachment.as_ref().unwrap();
        let after = program.comments[1].attachment.as_ref().unwrap();

        assert_eq!(before.node_id.get(), node_id(&program, AstType::ExpressionStatement, "value;"));
        assert_ne!(before.node_id.get(), flagged.id);
        assert_eq!(after.node_id.get(), flagged.id);
    }
}

#[test]
fn varied_syntax_assigns_every_comment_once() {
    for (source, source_type) in [
        (
            "const values = [/* first */, /* item */ item, , ...rest /* last */]; // done",
            SourceType::mjs(),
        ),
        (
            "if (/* test */ ready) { /* start */ work(); } else /* alternate */ { stop(); }\n\
             for (let i = /* init */ 0; i < 3; /* update */ i++) { continue; }\n\
             try { run(); } catch (/* catch */ error) { throw /* throw */ error; }\n\
             finally { /* finally */ done(); }\n\
             switch (value) { case /* case */ 1: break; default: /* default */ stop(); }",
            SourceType::mjs(),
        ),
        (
            "const node = <Widget {.../* spread */ props} value={/* attribute */ value}>\
             {.../* child */ children}{/* empty */}</Widget>;",
            SourceType::jsx(),
        ),
        (
            "@decorate(/* decorator */ name)\n\
             class Box<T> extends Base</* argument */ T> {\n\
                 constructor(public /* parameter */ value: T) {}\n\
                 field /* annotation */: T;\n\
                 method(/* input */ input: T): /* return */ T { return input; }\n\
             }\n\
             type Mutable<T> = { -readonly [K /* key */ in keyof T as /* name */ K]-?: /* value */ T[K] };",
            SourceType::ts(),
        ),
        (
            "const view = <Widget</* type */ Value> {.../* props */ props}\n\
             render={(value: /* annotation */ Value) => <>{/* child */}{value}</>} />;",
            SourceType::tsx(),
        ),
    ] {
        let allocator = Allocator::default();
        let mut program = parse(&allocator, source, source_type);
        assert!(program.comments.len() >= 3);
        assign(&mut program);
    }
}

#[test]
fn ownership_moves_with_comments_when_the_vector_changes() {
    let allocator = Allocator::default();
    let mut program =
        parse(&allocator, "// first\nfirst(); // tail\n// second\nsecond();", SourceType::mjs());
    assign(&mut program);
    let first = attachment(&program, "// first");
    let second = attachment(&program, "// second");
    let comment = &program.comments[0];
    assert_eq!(comment.attachment.as_ref(), Some(&first));
    let removed = program.comments.remove(1);
    program.comments.reverse();

    assert_eq!(attachment(&program, "// first"), first);
    assert_eq!(attachment(&program, "// second"), second);
    assert_eq!(removed.attachment.as_ref().unwrap().node_id.get(), first.node_id.get());
}

#[test]
fn ast_cloning_keeps_attachments_only_with_semantic_ids() {
    let allocator = Allocator::default();
    let mut program = parse(&allocator, "// leading\nfirst();", SourceType::mjs());
    assign(&mut program);
    let owner = attachment(&program, "// leading");
    assert_ne!(owner.node_id.get(), NodeId::ROOT);

    let cloned = program.clone_in(&allocator);
    assert!(cloned.comments[0].attachment.is_none());
    let mut cloned = program.clone_in_with_semantic_ids(&allocator);
    assert_eq!(attachment(&cloned, "// leading"), owner);
    assert_eq!(node_id(&cloned, AstType::ExpressionStatement, "first();"), owner.node_id.get());
    cloned.comments[0].attachment = None;
    assert_eq!(attachment(&program, "// leading"), owner);
}

#[test]
fn parser_exit_assignment_matches_the_complete_pass() {
    for (source, source_type) in [
        ("/* directive */ 'use strict'; /* statement */ first();", SourceType::mjs()),
        ("switch (value) { default: /* leading */ break; /* trailing */ }", SourceType::mjs()),
        (
            "switch (value) { case 0: first(); // statement\n\
             if (ready) second(); // case\n default: last(); } // switch",
            SourceType::mjs(),
        ),
        ("@dec /* export */ export class C {}", SourceType::ts()),
        ("/* export */ export default @dec /* class */ class C {}", SourceType::ts()),
        ("const f = (a = () => { /* body */ work(); }) => a;", SourceType::ts()),
        ("const f = async (a = () => { /* body */ work(); }) => a;", SourceType::ts()),
        (
            "/* before */ before(); /* await */ await /x/u; export {}; /* after */ after();",
            SourceType::unambiguous(),
        ),
        ("/* before */ before(); function invalid( {", SourceType::mjs()),
    ] {
        let allocator = Allocator::default();
        for tokens in [false, true] {
            for preserve_parens in [false, true] {
                let mut program = Parser::new(&allocator, source, source_type)
                    .with_config(RuntimeParserConfig::new(tokens))
                    .with_options(ParseOptions { preserve_parens, ..ParseOptions::default() })
                    .parse()
                    .program;
                let actual = attachments(&program);
                assign(&mut program);
                assert_eq!(actual, attachments(&program), "{source}");
            }
        }
    }
}

#[test]
fn remaining_assignment_handles_partially_attached_windows() {
    let allocator = Allocator::default();
    let source = "/* first */ const a = call(/* argument */ value);\n\
                  /* second */ const b = { /* property */ key: value };\n\
                  /* third */ last(); // tail";
    let mut program = parse(&allocator, source, SourceType::mjs());
    assign(&mut program);
    let expected = attachments(&program);
    for (index, comment) in program.comments.iter_mut().enumerate() {
        if index % 2 == 0 {
            comment.attachment = None;
        }
    }
    CommentAssignment::new().assign_remaining(&mut program);
    assert_eq!(expected, attachments(&program));
}
