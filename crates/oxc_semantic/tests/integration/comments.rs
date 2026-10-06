use oxc_allocator::Allocator;
use oxc_ast::{AstKind, AstType, CommentPlacement, ast::Program};
use oxc_ast_visit::Visit;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_comment_assignment::CommentAssignment;
use oxc_parser::Parser;
use oxc_semantic::{NodeId, SemanticBuilder};
use oxc_span::{GetSpan, SourceType, Span};

#[derive(Default)]
struct Nodes(Vec<(NodeId, AstType, Span)>);

impl<'a> Visit<'a> for Nodes {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.0.push((kind.node_id(), kind.ty(), kind.span()));
    }
}

fn owners(program: &Program<'_>) -> Vec<(AstType, Span, CommentPlacement)> {
    let mut nodes = Nodes::default();
    nodes.visit_program(program);
    let mut ids: Vec<_> = nodes.0.iter().map(|node| node.0).collect();
    ids.sort_unstable();
    assert_eq!(ids.first(), Some(&NodeId::ROOT));
    assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    program
        .comments
        .iter()
        .map(|comment| {
            let attachment = comment.attachment.expect("Unassigned comment");
            let (_, kind, span) = nodes.0.iter().find(|node| node.0 == attachment.node_id).unwrap();
            (*kind, *span, attachment.placement)
        })
        .collect()
}

#[track_caller]
fn assert_matches_standalone(source: &str, source_type: SourceType) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{source}: {:?}", parsed.diagnostics);
    let mut standalone = parsed.program;
    CommentAssignment::new().assign(&mut standalone);
    let expected = owners(&standalone);

    for full_nodes in [false, true] {
        let mut program = Parser::new(&allocator, source, source_type).parse().program;
        let original_comments = program.comments.to_vec();
        let result =
            SemanticBuilder::new().with_build_nodes(full_nodes).build_with_comments(&mut program);
        assert!(result.diagnostics.is_empty(), "{source}: {:?}", result.diagnostics);
        assert_eq!(result.semantic.nodes().is_empty(), !full_nodes);
        assert_eq!(result.semantic.comments().len(), original_comments.len());
        for (assigned, original) in result.semantic.comments().iter().zip(original_comments) {
            let mut assigned = *assigned;
            assert!(assigned.attachment.is_some());
            assigned.attachment = original.attachment;
            assert_eq!(assigned, original, "Assignment changed source comment metadata");
        }
        if full_nodes {
            // Both views remain usable after writing attachments. In particular,
            // the stored root references the original arena vector header.
            let root = result.semantic.nodes().program();
            assert_eq!(owners(root), expected, "{source}");
            assert_eq!(root.comments.as_ptr(), result.semantic.comments().as_ptr());
            for comment in result.semantic.comments() {
                let attachment = comment.attachment.unwrap();
                let kind = result.semantic.nodes().kind(attachment.node_id);
                assert_eq!(kind.node_id(), attachment.node_id);
            }
        }
        drop(result);
        assert_eq!(owners(&program), expected, "{source}");
    }
}

#[test]
fn comments_match_standalone_in_both_node_storage_modes() {
    for source in [
        "",
        "const value = input;",
        "// first\n/* second */\n/*! license */",
        "#!/usr/bin/env node\n'use strict';\n/* before */ value; // after",
        "first(); /* between */ second(); // end",
        "outer(1, inner(/* argument */ value));",
        "console.log(1); console.log(foo ? bar ? /* branch */ one : two : three);",
        "const empty = [/* inside */]; const values = [item, /* after last */];",
        "function fn(/* params */) { /* body */ }",
        "const object = { /* key */ method(/* param */ value) { return /* return */ value; } };",
        "/*#__PURE__*/ call(/* arg */ value);",
        "/*#__NO_SIDE_EFFECTS__*/ export const fn = () => value;",
        "const key = /* @__KEY__ */ 'name';",
        "/* istanbul ignore file */ import(/* webpackChunkName: 'chunk' */ 'module');",
        "tag`head${/* before */ left /* after */}tail${/* next */ right}end`;",
        "class Example { /* member */ #value = 1; method(/* param */ value) {} }",
    ] {
        assert_matches_standalone(source, SourceType::mjs());
    }
    for source in [
        "type Template<T> = `head${/* before */ T /* after */}tail`;",
        "type Value = { /* member */ readonly value?: /* type */ string };",
        "@decorator(/* arg */ value) export class Example { /* member */ method() {} }",
        "const fn = /* before */ <T,>(value: T): /* return */ T => value;",
        "const element = <div>{/* jsx */ value}<span /* prop */ title='text' /></div>;",
        "enum E { /* member */ A = 1, B = /* init */ 2 } namespace N { /* body */ type T = E; }",
    ] {
        assert_matches_standalone(source, SourceType::tsx());
    }
}

#[test]
fn comments_survive_inline_storage_and_frame_stack_growth() {
    let dense = "/* before */ value = /* left */ left + /* right */ right; // after\n".repeat(64);
    let nested = format!("{}/* leaf */ value;{}", "{\n".repeat(128), "}\n".repeat(128));
    for source in [dense, nested] {
        assert_matches_standalone(&source, SourceType::mjs());
    }
}

#[test]
fn assignment_reuses_semantic_ids_and_preserves_scoping() {
    let allocator = Allocator::default();
    let source = "/* declaration */ const value = input; function fn(arg) { return /* use */ value + arg; } fn(value);";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
    let baseline = SemanticBuilder::new().with_build_nodes(true).build(&program);
    let ids: Vec<_> = baseline
        .semantic
        .nodes()
        .iter()
        .map(|node| (node.id(), node.kind().ty(), node.kind().span()))
        .collect();
    let stats = baseline.semantic.stats();
    drop(baseline);

    let assigned = SemanticBuilder::new().with_build_nodes(true).build_with_comments(&mut program);
    let assigned_stats = assigned.semantic.stats();
    assert_eq!(
        (
            assigned_stats.nodes,
            assigned_stats.scopes,
            assigned_stats.symbols,
            assigned_stats.references
        ),
        (stats.nodes, stats.scopes, stats.symbols, stats.references),
    );
    assert_eq!(
        assigned
            .semantic
            .nodes()
            .iter()
            .map(|node| (node.id(), node.kind().ty(), node.kind().span()))
            .collect::<Vec<_>>(),
        ids,
    );
    let scoping = assigned.semantic.into_scoping();
    assert_eq!(scoping.symbols_len(), 3);
    assert!(program.comments.iter().all(|comment| comment.attachment.is_some()));
}

#[test]
fn shared_build_leaves_comments_unassigned() {
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, "/* before */ value;", SourceType::mjs()).parse().program;
    let result = SemanticBuilder::new().build(&program);
    assert!(result.semantic.comments().iter().all(|comment| comment.attachment.is_none()));
}

#[test]
fn semantic_comments_print_and_reparse() {
    let allocator = Allocator::default();
    let source = "/* before */ const value = [/* empty */]; call(/* arg */ value); // after";
    for minify in [false, true] {
        let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
        let scoping = SemanticBuilder::new_compiler()
            .build_with_comments(&mut program)
            .semantic
            .into_scoping();
        let printed = Codegen::new()
            .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
            .with_scoping(Some(scoping))
            .build(&program)
            .code;
        let reparsed = Parser::new(&allocator, &printed, SourceType::mjs()).parse();
        assert!(reparsed.diagnostics.is_empty(), "{printed}: {:?}", reparsed.diagnostics);
        assert_eq!(reparsed.program.comments.len(), program.comments.len());
        for text in ["/* before */", "/* empty */", "/* arg */", "// after"] {
            assert!(printed.contains(text), "{printed}");
        }
    }
}

#[cfg(feature = "jsdoc")]
#[test]
fn jsdoc_and_cfg_can_share_comment_assignment() {
    let allocator = Allocator::default();
    let source = "/** Documentation. */ function fn(value) { return /* result */ value; }";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
    let result = SemanticBuilder::new_linter().build_with_comments(&mut program);
    assert!(result.diagnostics.is_empty());
    assert_eq!(owners(result.semantic.nodes().program()).len(), 2);
    assert!(result.semantic.nodes().iter().any(|node| {
        result.semantic.nodes().flags(node.id()).contains(oxc_semantic::NodeFlags::JSDoc)
    }));
    #[cfg(feature = "cfg")]
    assert!(result.semantic.cfg().is_some());
}
