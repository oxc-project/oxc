use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_mangler::{MangleOptions, Mangler};
use oxc_minifier::{CompressOptions, Compressor, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

#[test]
fn compressed_comments_are_idempotent() {
    for dce in [false, true] {
        for minify in [false, true] {
            for source in [
                "consume(new Set(), (function() {}));",
                "consume(new Set(), (() => {}));",
                "consume(new Set(), (val => val));",
                "consume(new Set(), (function() { return (function() {}); }));",
                "consume(new Set(), (function(/* parameter */) {}));",
                "x = +new Date;",
                "x = [...new Set([])];",
                "x = (// c\n a = 1, a);",
                "a && (// c\n b || c);",
                "a && (/* c */ b || c);",
                "var a; var x = (a = {}, // c\n a.k = function(){return this;}, a); use(x);",
                "function f(){consume(new Set());do {} while(test());}",
                "x = a || (// c\n b ? c : d);",
                "a || (// c\n b = c);",
                "f = () => (/*c*/ {a: b, c: d});",
                "x = (a || b // c\n) && d;",
                "x = a && (b || c && (d || (e && f) // c\n));",
                "x =\n/*a*/ (\n/*b*/ a\n).b;",
                "var x; /* istanbul ignore file */ if (a) x=b; else x=c; use(x);",
                "f(); /* istanbul ignore file */\n g();",
                "var s=require('s'); var x; /* istanbul ignore file */ if(s.a)x=b;else if(s.c)x=d;else x=s.e;exports.default=(0,s.wrap)(x);module.exports=exports.default;",
            ] {
                let print = |source: &str| {
                    let allocator = Allocator::default();
                    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
                    assert!(parsed.diagnostics.is_empty(), "{source}: {:?}", parsed.diagnostics);
                    let mut program = parsed.program;
                    SemanticBuilder::new().build(&program);
                    let compressor = Compressor::new(&allocator);
                    if dce {
                        compressor.dead_code_elimination(&mut program, CompressOptions::dce());
                    } else {
                        compressor.build(&mut program, CompressOptions::default());
                    }
                    Codegen::new()
                        .with_options(CodegenOptions {
                            minify,
                            // Match the monitor: compression retains annotations,
                            // while DCE also retains ordinary comments.
                            comments: CommentOptions {
                                normal: dce,
                                jsdoc: dce,
                                ..CommentOptions::default()
                            },
                            ..CodegenOptions::default()
                        })
                        .build(&program)
                        .code
                };
                let first = print(source);
                let second = print(&first);
                assert_eq!(second, first, "{source}, dce={dce}, minify={minify}");
                assert_eq!(print(&second), second, "{source}, dce={dce}, minify={minify}");
            }
        }
    }
}

#[test]
fn pure_class_heritage_is_idempotent() {
    let source = "class ArrayFormatterIssue extends (/*#__PURE__*/Struct({_tag:propertySignature(Literal(`Pointer`,`Unexpected`,`Missing`,`Composite`,`Refinement`,`Transformation`,`Type`,`Forbidden`)).annotations({description:`The tag identifying the type of parse issue`}),path:propertySignature(Array$(PropertyKey$)).annotations({description:`The path to the property where the issue occurred`}),message:propertySignature(String$).annotations({description:`A descriptive message explaining the issue`})}).annotations({identifier:`ArrayFormatterIssue`,description:`Represents an issue returned by the ArrayFormatter formatter`})){}exports.ArrayFormatterIssue=ArrayFormatterIssue;";
    for mangle in [false, true] {
        for minify in [false, true] {
            let print = |source: &str| {
                let allocator = Allocator::default();
                let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
                assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
                let mut program = parsed.program;

                let result = Minifier::new(MinifierOptions {
                    compress: Some(CompressOptions::default()),
                    mangle: mangle.then(MangleOptions::default),
                    mangle_properties: None,
                })
                .minify(&allocator, &mut program);
                Codegen::new()
                    .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
                    .with_scoping(result.scoping)
                    .build(&program)
                    .code
            };
            let first = print(source);
            assert_eq!(first.matches("/*#__PURE__*/").count(), 1, "{first}");
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &first, SourceType::mjs()).parse();
            assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
            let program = parsed.program;

            let regenerated = Codegen::new()
                .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
                .build(&program)
                .code;
            assert_eq!(regenerated, first, "mangle={mangle}, minify={minify}");
            assert_eq!(print(&first), first, "mangle={mangle}, minify={minify}");
        }
    }
}

#[test]
fn mangler_rebuild_preserves_comments_after_reordering() {
    let allocator = Allocator::default();
    let source = "/* first */ first();\n/* second */ second();";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    program.body.swap(0, 1);
    let result = Mangler::new().build(&program);
    assert_eq!(program.comments[0].node_id.get(), program.body[1].node_id());
    assert_eq!(program.comments[1].node_id.get(), program.body[0].node_id());
    let printed = Codegen::new().with_scoping(Some(result.scoping)).build(&program).code;
    assert!(printed.find("/* second */").unwrap() < printed.find("second()").unwrap());
    assert!(printed.find("second()").unwrap() < printed.find("/* first */").unwrap());
}

#[test]
fn compressor_entry_points_preserve_assigned_owners() {
    for dce in [false, true] {
        let allocator = Allocator::default();
        let source = "/* first */ first();\n/* second */ second();";
        let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

        // Compression can merge expression statements. Attach to the calls,
        // which survive that transform, so this checks identity preservation.
        for (comment, statement) in program.comments.iter().zip(&program.body) {
            let Statement::ExpressionStatement(statement) = statement else { unreachable!() };
            comment.node_id.set(statement.expression.node_id());
        }
        program.body.swap(0, 1);
        let compressor = Compressor::new(&allocator);
        if dce {
            compressor.dead_code_elimination(&mut program, CompressOptions::dce());
        } else {
            compressor.build(&mut program, CompressOptions::default());
        }
        let printed = Codegen::new().build(&program).code;
        assert!(printed.find("/* second */").unwrap() < printed.find("second()").unwrap());
        assert!(printed.find("second()").unwrap() < printed.find("/* first */").unwrap());
    }
}

#[test]
fn minifier_rebuilds_preserve_comments_through_dce_and_mangling() {
    let allocator = Allocator::default();
    let source = "/* removed */ if (false) gone();\n/* live */ consume(/* argument */ 1);";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    let result = Minifier::new(MinifierOptions {
        compress: Some(CompressOptions::default()),
        mangle: Some(MangleOptions::default()),
        mangle_properties: None,
    })
    .minify(&allocator, &mut program);
    assert!(
        program
            .comments
            .iter()
            .all(|comment| comment.node_id.get() != oxc_ast::Comment::UNASSIGNED_NODE_ID)
    );
    assert_eq!(program.comments[0].node_id.get(), oxc_semantic::NodeId::ORPHANED);
    let printed = Codegen::new()
        .with_options(CodegenOptions::default())
        .with_scoping(result.scoping)
        .build(&program)
        .code;
    assert!(!printed.contains("gone()"));
    assert!(printed.find("/* live */").unwrap() < printed.find("consume(").unwrap());
    assert!(printed.find("consume(").unwrap() < printed.find("/* argument */").unwrap());
    assert!(!printed.contains("/* removed */"), "{printed}");
    let reparsed = Parser::new(&allocator, &printed, SourceType::mjs()).parse();
    assert!(reparsed.diagnostics.is_empty(), "{printed}: {:?}", reparsed.diagnostics);
}
