use oxc_allocator::Allocator;
use oxc_ast::{
    CommentPlacement,
    ast::{ArrayExpressionElement, Expression, Program, Statement},
};
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions, LegalComment};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::{SourceType, Span};
use oxc_syntax::node::NodeId;

fn comments(program: &Program<'_>) -> Vec<String> {
    let mut comments: Vec<_> = program
        .comments
        .iter()
        .map(|comment| {
            comment
                .content_span()
                .source_text(program.source_text)
                .lines()
                .map(str::trim)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect();
    comments.sort_unstable();
    comments
}

/// Verify retention, syntax, unchanged program meaning, and stable formatting.
fn check(source: &str, source_type: SourceType, minify: bool) -> String {
    check_with_parse_options(source, source_type, minify, ParseOptions::default())
}

fn check_with_parse_options(
    source: &str,
    source_type: SourceType,
    minify: bool,
    parse_options: ParseOptions,
) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).with_options(parse_options).parse();
    assert!(parsed.diagnostics.is_empty(), "{source}: {:?}", parsed.diagnostics);
    let program = parsed.program;
    let without_comments = CodegenOptions {
        minify: true,
        comments: CommentOptions::disabled(),
        ..CodegenOptions::default()
    };
    let expected = Codegen::new().with_options(without_comments.clone()).build(&program).code;
    let expected_comments = comments(&program);

    let options = CodegenOptions { minify, ..CodegenOptions::default() };
    let code = Codegen::new().with_options(options.clone()).build(&program).code;
    let reparsed = Parser::new(&allocator, &code, source_type).with_options(parse_options).parse();
    assert!(
        reparsed.diagnostics.is_empty(),
        "{source}\nGenerated:\n{code}\n{:?}",
        reparsed.diagnostics
    );
    let reparsed = reparsed.program;
    assert_eq!(comments(&reparsed), expected_comments, "{source}\nGenerated:\n{code}");
    assert_eq!(
        Codegen::new().with_options(without_comments).build(&reparsed).code,
        expected,
        "{source}\nGenerated:\n{code}"
    );

    assert_eq!(
        Codegen::new().with_options(options).build(&reparsed).code,
        code,
        "Unstable formatting for {source}"
    );
    code
}

#[test]
fn statements() {
    for source in [
        "// leading\nfirst(); // trailing\n/* between */ second(); /* eof */",
        "{ // open\n first(); // first\n // end\n}",
        "if (a) { /* inside */ } else if (b) { /* other */ } else { /* final */ }",
        "try { /* try */ } catch (/* parameter */ error) { /* catch */ } finally { /* finally */ }",
        "function f() { /* only */ } class C { /* class */ }",
        "#!/usr/bin/env node\n/* file */\nrun(); // eof",
        "/* first */ ('string'); // trailing",
        "(/* inside string */ 'string' /* after string */);",
        "/* empty */",
        "/* 1 */ 'use strict' /* 2 */; /* 3 */",
        "(/* 1 */ function /* 2 */ f(/* 3 */ a /* 4 */) { /* 5 */ } /* 6 */);",
        "/* 1 */ (/* 2 */ (/* 3 */ a /* 4 */, /* 5 */ b /* 6 */) /* 7 */) /* 8 */; /* 9 */",
        "const [/* one */ x /* two */, /* three */] /* four */ = [];",
    ] {
        for minify in [false, true] {
            check(source, SourceType::mjs(), minify);
        }
    }
}

#[test]
fn expressions() {
    for source in [
        "const a = [/* empty */], b = { /* object */ }; f(/* args */); new C(/* new */);",
        "const a = [one, /* between */ two, /* end */]; const b = { /* key */ key: /* value */ value, /* end obj */ };",
        "a /* left */ + /* right */ b * /* nested */ c; a / /* division */ b;",
        "foo ? bar ? /* inner */ x : y : z;",
        "[/* hole */ , /* hole two */ , value, /* tail */];",
        "const { /* binding */ a = /* value */ 1 } = obj;",
        "(a /* middle */ + b) /* outer */ + c; a && /* and */ b || /* or */ c;",
        "const { /* empty pattern */ } = obj; const [/* array pattern */] = arr;",
        "const o = { /* shorthand */ a, b: /* value same */ b }; const { c: /* binding */ c } = o;",
        "const f = (/* params */) => 1; const g = async (/* param */ value /* after */) => value;",
        "function f() { return (// return\n value); } function* g() { yield (// yield\n value); throw (// throw\n value); }",
        "obj[/* index */ key /* index end */]; fn(value, /* args end */); import('mod' /* import end */);",
        "value /* postfix */ ++; void /* unary */ value; await /* awaited */ value;",
        "const t = tag`head${/* before */ value /* after */}tail${// line\n value}end`;",
        "const o = { m(/* param */) { /* method */ }, get p() { /* getter */ } };",
        "const o = { [/* key */ value /* key end */](/* param */ x /* params end */) { /* body */ } };",
        "exports[/* key */ 'name' /* end */] /* target */ = value; const mod = require(/* source */ 'mod' /* end */);",
        "Object.defineProperty(exports, /* key */ 'name' /* end */, {/* descriptor */});",
    ] {
        for minify in [false, true] {
            check(source, SourceType::mjs(), minify);
        }
    }
}

#[test]
fn leading_comments_inside_generated_grouping() {
    for source in [
        "x = (// c\n a = 1, a);",
        "x = (/* c */ a = 1, a);",
        "a && (// c\n b || c);",
        "a && (/* c */ b || c);",
        "(a + (// c\n b + c)) * d;",
        "a + (// c\n b ? c : d);",
        "a && (/* c */ b ? c : d);",
        "a || (// c\n b = c);",
        "f = () => (/* c */ { x: a });",
        "(/* c */ { x: a });",
        "(/* c */ class {});",
    ] {
        for minify in [false, true] {
            let code = check_with_parse_options(
                source,
                SourceType::mjs(),
                minify,
                ParseOptions { preserve_parens: false, ..ParseOptions::default() },
            );
            assert!(code.contains("(// c") || code.contains("(/* c */"), "{code}");
        }
    }
}

#[test]
fn trailing_comments_inside_generated_grouping() {
    for source in [
        "a && (b || c // c\n);",
        "a + (b ? c : d // c\n);",
        "a || (b = c // c\n);",
        "x = (a = 1, a // c\n);",
        "x = (a || b // c\n) && d;",
        "function f() { return a && (b || c && (d || e // c\n)); }",
    ] {
        for minify in [false, true] {
            check_with_parse_options(
                source,
                SourceType::mjs(),
                minify,
                ParseOptions { preserve_parens: false, ..ParseOptions::default() },
            );
        }
    }
}

#[test]
fn adjacent_comments_do_not_leave_trailing_spaces() {
    for minify in [false, true] {
        let code = check_with_parse_options(
            "x =\n/*a*/ (\n/*b*/ a\n).b;",
            SourceType::mjs(),
            minify,
            ParseOptions { preserve_parens: false, ..ParseOptions::default() },
        );
        assert!(!code.contains("/*a*/ \n"), "{code}");
    }
}

#[test]
fn import_attribute_keys_keep_comments() {
    for minify in [false, true] {
        check(
            "import value from 'mod' with { /* key */ type /* end */: 'json' };",
            SourceType::mjs(),
            minify,
        );
        check(
            "export { value } from 'mod' with { /* key */ 'type' /* end */: 'json' };",
            SourceType::mjs(),
            minify,
        );
    }
}

#[test]
fn generated_annotations_after_operators() {
    for source in ["x = +new Date;", "x = [...new Set([])];"] {
        let allocator = Allocator::default();
        let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
        let Statement::ExpressionStatement(statement) = &mut program.body[0] else {
            unreachable!()
        };
        let Expression::AssignmentExpression(assignment) = &mut statement.expression else {
            unreachable!()
        };
        let argument = match &mut assignment.right {
            Expression::UnaryExpression(unary) => &mut unary.argument,
            Expression::ArrayExpression(array) => {
                let ArrayExpressionElement::SpreadElement(spread) = &mut array.elements[0] else {
                    unreachable!()
                };
                &mut spread.argument
            }
            _ => unreachable!(),
        };
        let Expression::NewExpression(new) = argument else { unreachable!() };
        // Compression can mark a known constructor pure without a source comment.
        new.pure = true;
        for minify in [false, true] {
            let options = CodegenOptions { minify, ..CodegenOptions::default() };
            let first = Codegen::new().with_options(options.clone()).build(&program).code;
            assert_eq!(first.matches("/* @__PURE__ */").count(), 1, "{first}");
            let parsed = Parser::new(&allocator, &first, SourceType::mjs()).parse();
            assert!(parsed.diagnostics.is_empty(), "{first}: {:?}", parsed.diagnostics);
            let reparsed = parsed.program;

            assert_eq!(Codegen::new().with_options(options).build(&reparsed).code, first);
        }
    }
}

#[test]
fn parenthesized_functions_without_preserved_parentheses() {
    for source in [
        "consume(/*#__PURE__*/ new Set(), (function() {}));",
        "consume(/*#__PURE__*/ new Set(), (() => 0));",
        "/* unrelated */ consume((function() { return (function() {}); }));",
        "/* unrelated */ consume((() => (() => 0)));",
        "/* unrelated */ (function() {});",
        "/* unrelated */ ((() => 0));",
        "consume((/* inner */ function() {}));",
        "consume((/* inner */ () => 0));",
    ] {
        for preserve_parens in [false, true] {
            for minify in [false, true] {
                check_with_parse_options(
                    source,
                    SourceType::mjs(),
                    minify,
                    ParseOptions { preserve_parens, ..ParseOptions::default() },
                );
            }
        }
    }
}

#[test]
fn jsx_arrows_with_comments_without_preserved_parentheses() {
    for source in [
        "/* component */ const i = <C handler={(val) => {}} />;",
        "const i = <C handler={/* handler */ (val) => { /* body */ }} />;",
        "const i = <C value={a = /* assigned */ b} sequence={(a, /* comma */ b)} />;",
    ] {
        for minify in [false, true] {
            let code = check_with_parse_options(
                source,
                SourceType::jsx(),
                minify,
                ParseOptions { preserve_parens: false, ..ParseOptions::default() },
            );
            assert!(!code.contains("{((val)"), "{code}");
        }
    }
}

#[test]
fn trailing_comments_before_flat_closing_braces() {
    for source in [
        "function f() { const { cache, flags = 'r', mode = 438, // =0666\n } = opts; }",
        "function f() { const { force = false, noSpaceAfterFlags = false, // flags\n } = params; }",
        "const { value, /* last */\n } = opts;",
        "const { value /* last */\n } = opts;",
        "const { value, // first\n // last\n } = opts;",
        "function f({ value = 1, // default\n }: { value: number }) {}",
        "type Replace<P> = (...args: { [K in keyof P]: P[K]; // tuple\n }) => P; // handle objects",
        "type Match<T> = Readonly<{ [K in keyof T]?: any; // narrowed\n } & Document>;",
        "type Match<T> = { [K in keyof T]: T[K] /* last */\n };",
    ] {
        for minify in [false, true] {
            check(source, SourceType::ts(), minify);
        }
    }
}

#[test]
fn pure_annotation_inside_generated_parentheses() {
    for source in [
        "class C extends /*#__PURE__*/ factory().annotations({}) {}",
        "class C extends (/*#__PURE__*/ factory().annotations({})) {}",
        "class C extends // #__PURE__\nfactory() {}",
        "const x = /*#__PURE__*/ factory().value;",
        "const x = /*#__PURE__*/ factory()();",
        "const x = /*#__PURE__*/ new Factory().value;",
        "const x = /*#__PURE__*/ new Factory(arg).value;",
        "const x = new (/*#__PURE__*/ factory())();",
        "function f() { return /*#__PURE__*/ factory().value; }",
    ] {
        for minify in [false, true] {
            check(source, SourceType::mjs(), minify);
        }
    }
}

#[test]
fn pure_annotation_group_keeps_source_order() {
    let source = "const x = /* before */ /*#__PURE__*/ /* after */ factory().value;";
    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    // A transform can move neighboring comments onto the annotated call.
    let attachment = &program.comments[1];
    let owner = attachment.node_id.get();
    let placement = attachment.placement;
    for comment in &mut program.comments {
        let attachment = comment;
        attachment.node_id.set(owner);
        attachment.placement = placement;
    }
    for minify in [false, true] {
        let code = Codegen::new()
            .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
            .build(&program)
            .code;
        assert_eq!(code.matches("/*#__PURE__*/").count(), 1);
        assert!(code.find('(').unwrap() < code.find("/* before */").unwrap());
        assert!(code.find("/* before */").unwrap() < code.find("/*#__PURE__*/").unwrap());
        assert!(code.find("/*#__PURE__*/").unwrap() < code.find("/* after */").unwrap());
    }
}

#[test]
fn modules_and_annotations() {
    for source in [
        "import { /* imported */ foo as /* local */ bar, /* next */ baz } from /* source */ 'mod';",
        "import { /* empty import */ } from 'mod'; export { /* exported */ value /* export end */ };",
        "import value from 'mod' with { /* attribute */ type: /* kind */ 'json' };",
        "/* @__PURE__ custom */ f(); /* #__PURE__ */ new C();",
        "/* @__NO_SIDE_EFFECTS__ */ export function f() {}",
        "const f = /* #__NO_SIDE_EFFECTS__ */ () => 1; const o = { [/* @__KEY__ */ 'key']: value };",
        "import(/* webpackChunkName: 'chunk' */ 'mod'); import(/* @vite-ignore */ name);",
        "/* istanbul ignore file */\n// normal\nrun(); /*! license */",
    ] {
        for minify in [false, true] {
            check(source, SourceType::mjs(), minify);
        }
    }
}

#[test]
fn typescript_and_jsx() {
    for source in [
        "const x = value /* assertion */ !; const z = (/* type open */ value /* type end */) as T;",
        "type T = { /* type */ }; type U = [/* tuple */]; type M = { [K in /* keyof */ keyof T]: /* mapped */ T[K] };",
        "interface Empty { /* inside */ } interface I { m(/* in */ p: number) /* out */: void; (p: number) // signature\n: void; }",
        "type T = { m(p: number) // signature\n: void; }; type F = (this: T, /* p */ p: T /* end */) /* after */ => T;",
        "type T = U extends Cell<infer V/**, infer _ or unknown, or any valid type **/> ? V : never;",
        "class C { /** s1 */ static s1: number; /** s2 */ constructor() {} }",
        "class C { [/* key */ value /* key end */](/* param */ x /* params end */) { /* body */ } }",
        "class C { constructor(/** @internal */ public x: string, /** @internal */ public y: string) {} }",
        "@(/* outer */ dec) class C { @dec\n/* between */ accessor z = 1; }",
        "const fragment = < /* start */ ></ /* end */>;// tail",
        "const element = <C /* self closing */ />; const line = <C // close\n/>;",
        "type T = `head${/* before */ string /* after */}tail`; enum E { /* enum */ } namespace N { /* namespace */ }",
        "const view = <C {.../* spread */ attrs /* spread end */}>{.../* child spread */ children /* child end */}</C>;",
        "const view = <C /* attribute */ value={/* expression */ x}>{/* empty */}{/* child */ x}</C>;",
        "const view = <C value={(/* first */ a, /* second */ b)}>{(/* child */ a, b)}</C>;",
        "const view = <C value={(a, b)}>{(a, b)}</C>;",
        "@/* decorator */ dec class C { /* member */ m() { /* body */ } }",
    ] {
        for minify in [false, true] {
            check(source, SourceType::tsx(), minify);
        }
    }
}

#[test]
fn moved_and_removed_owners() {
    let allocator = Allocator::default();
    let source = "/* first */ first(); // first tail\n/* second */ second(); // second tail\n/* third */ third();";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    program.body.swap(0, 2);
    let code = Codegen::new().build(&program).code;
    assert!(code.find("/* third */").unwrap() < code.find("third()").unwrap(), "{code}");
    assert!(code.find("/* first */").unwrap() > code.find("second()").unwrap(), "{code}");
    assert!(code.find("// first tail").unwrap() > code.find("first()").unwrap(), "{code}");
    assert_eq!(
        comments(&Parser::new(&allocator, &code, SourceType::mjs()).parse().program),
        comments(&program)
    );
    program.body.clear();
    let code = Codegen::new().build(&program).code;
    assert!(code.is_empty(), "{code}");
}

#[test]
fn removed_owners_preserve_only_legal_and_file_comments() {
    let allocator = Allocator::default();
    let source = "/* normal */ first();\n/** jsdoc */ second();\n/*! license */ third();\n/* istanbul ignore file */ fourth();\n/* @__PURE__ */ fifth();";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
    program.body.clear();

    // Cover both missing owners and owners explicitly orphaned by a semantic rebuild.
    for orphaned in [false, true] {
        if orphaned {
            for comment in &program.comments {
                comment.node_id.set(NodeId::ORPHANED);
            }
        }
        for minify in [false, true] {
            let code = Codegen::new()
                .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
                .build(&program)
                .code;
            assert_eq!(
                comments(&Parser::new(&allocator, &code, SourceType::mjs()).parse().program),
                vec!["! license", "istanbul ignore file"],
                "{code}"
            );
        }
    }
}

#[test]
fn inside_comments_follow_ids_when_container_spans_change() {
    let allocator = Allocator::default();
    let source = "[[], /* outer */];";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    for minify in [false, true] {
        let options = CodegenOptions { minify, ..CodegenOptions::default() };
        let expected = Codegen::new().with_options(options.clone()).build(&program).code;
        for discard_spans in [false, true] {
            let Statement::ExpressionStatement(statement) = &mut program.body[0] else {
                unreachable!();
            };
            let Expression::ArrayExpression(outer) = &mut statement.expression else {
                unreachable!();
            };
            let attachment = &program.comments[0];
            assert_eq!(attachment.node_id.get(), outer.node_id());
            assert_eq!(attachment.placement, CommentPlacement::Dangling);
            let span = if discard_spans { Span::default() } else { outer.span };
            let ArrayExpressionElement::ArrayExpression(inner) = &mut outer.elements[0] else {
                unreachable!();
            };
            // A transform can reuse or discard source spans without changing ownership.
            inner.span = span;
            assert_eq!(Codegen::new().with_options(options.clone()).build(&program).code, expected);
        }
    }
}

#[test]
fn generated_container_does_not_claim_program_comments() {
    let allocator = Allocator::default();
    let source = "[];\n/* eof */";
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    assert_eq!(program.comments[0].node_id.get(), NodeId::ROOT);
    let Statement::ExpressionStatement(statement) = &mut program.body[0] else {
        unreachable!();
    };
    let Expression::ArrayExpression(array) = &mut statement.expression else {
        unreachable!();
    };
    array.node_id.set(NodeId::DUMMY);
    for minify in [false, true] {
        let code = Codegen::new()
            .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
            .build(&program)
            .code;
        assert!(code.starts_with("[]"), "{code}");
        assert_eq!(code.matches("/* eof */").count(), 1, "{code}");
    }
}

#[test]
fn leading_comments_keep_indentation() {
    assert_eq!(
        check("class C {\n/** method */\nm() {}\n}", SourceType::mjs(), false),
        "class C {\n\t/** method */\n\tm() {}\n}\n"
    );
}

#[test]
fn binary_chain_comments() {
    // Binary chains use an iterative printer. Exercise comment frames for many
    // successive owners without changing that traversal into recursive printing.
    let source = format!("{}value;", "value /* left */ + /* right */ ".repeat(256));
    for minify in [false, true] {
        check(&source, SourceType::mjs(), minify);
    }
}

#[test]
fn return_comment_without_preserved_parentheses() {
    let allocator = Allocator::default();
    let source = "function f() { return (// value\n x + y); }";
    let program = Parser::new(&allocator, source, SourceType::mjs())
        .with_options(oxc_parser::ParseOptions {
            preserve_parens: false,
            ..oxc_parser::ParseOptions::default()
        })
        .parse()
        .program;

    let code = Codegen::new().build(&program).code;
    let parsed = Parser::new(&allocator, &code, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{code}: {:?}", parsed.diagnostics);
    let options = CodegenOptions::minify();
    assert_eq!(
        Codegen::new().with_options(options.clone()).build(&parsed.program).code,
        Codegen::new().with_options(options).build(&program).code,
        "{code}"
    );
}

#[test]
fn string_statement_without_preserved_parentheses() {
    let allocator = Allocator::default();
    let source = "(/* string */ 'use strict');";
    let program = Parser::new(&allocator, source, SourceType::mjs())
        .with_options(oxc_parser::ParseOptions {
            preserve_parens: false,
            ..oxc_parser::ParseOptions::default()
        })
        .parse()
        .program;

    for minify in [false, true] {
        let code = Codegen::new()
            .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
            .build(&program)
            .code;
        let parsed = Parser::new(&allocator, &code, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty(), "{code}: {:?}", parsed.diagnostics);
        assert!(parsed.program.directives.is_empty(), "{code}");
        assert_eq!(comments(&parsed.program), comments(&program), "{code}");
    }
}

#[test]
fn options() {
    let allocator = Allocator::default();
    let source =
        "/* normal */ f(); // tail\n/** jsdoc */ g(); h(); /*! license */\n/* @__PURE__ */ i();";
    let program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    for legal in
        [LegalComment::None, LegalComment::Inline, LegalComment::Eof, LegalComment::External]
    {
        let options = CodegenOptions {
            comments: CommentOptions {
                normal: false,
                jsdoc: false,
                annotation: false,
                legal: legal.clone(),
            },
            ..CodegenOptions::default()
        };
        let result = Codegen::new().with_options(options).build(&program);
        assert!(
            !result.code.contains("normal")
                && !result.code.contains("tail")
                && !result.code.contains("jsdoc")
                && !result.code.contains("PURE"),
            "{}",
            result.code
        );
        assert_eq!(
            result.code.contains("license"),
            matches!(legal, LegalComment::Inline | LegalComment::Eof)
        );
        assert_eq!(result.legal_comments.len(), usize::from(legal == LegalComment::External));
    }
}

#[test]
fn legal_jsdoc_is_printed_once() {
    let allocator = Allocator::default();
    let source = "f(); /** @license legal */";
    let program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    for legal in [
        LegalComment::None,
        LegalComment::Inline,
        LegalComment::Eof,
        LegalComment::Linked("input.LEGAL.txt".into()),
        LegalComment::External,
    ] {
        let result = Codegen::new()
            .with_options(CodegenOptions {
                comments: CommentOptions { legal: legal.clone(), ..CommentOptions::default() },
                ..CodegenOptions::default()
            })
            .build(&program);
        assert_eq!(
            result.code.matches("@license legal").count(),
            usize::from(matches!(legal, LegalComment::Inline | LegalComment::Eof)),
            "{}",
            result.code
        );
        assert_eq!(
            result.legal_comments.len(),
            usize::from(matches!(legal, LegalComment::Linked(_) | LegalComment::External))
        );
    }
}

#[cfg(feature = "sourcemap")]
#[test]
fn source_mappings_follow_comments() {
    let allocator = Allocator::default();
    let source = "/* 💬 */\nconst array = [value, /* close */]; // tail\nnext();";
    let program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    let result = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: Some("input.js".into()),
            ..CodegenOptions::default()
        })
        .build(&program);
    let map = result.map.unwrap();
    let position = |text: &str, needle: &str| {
        let prefix = &text[..text.find(needle).unwrap()];
        (
            u32::try_from(prefix.bytes().filter(|&byte| byte == b'\n').count()).unwrap(),
            u32::try_from(prefix.rsplit('\n').next().unwrap().encode_utf16().count()).unwrap(),
        )
    };
    for token in ["value", "]", "next"] {
        let (src_line, src_col) = position(source, token);
        let (dst_line, dst_col) = position(&result.code, token);
        assert!(
            map.get_tokens().any(|mapping| {
                mapping.get_src_line() == src_line
                    && mapping.get_src_col() == src_col
                    && mapping.get_dst_line() == dst_line
                    && mapping.get_dst_col() == dst_col
            }),
            "Missing mapping for {token}: {}",
            result.code
        );
    }
}

#[test]
fn unapplied_annotations_do_not_become_applied_when_printed() {
    for source in [
        "const foo /* #__PURE__ */ = pureOperation();",
        "const foo /* @__NO_SIDE_EFFECTS__ */ = () => effect();",
    ] {
        let allocator = Allocator::default();
        let program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
        let printed = Codegen::new().build(&program).code;
        assert!(!printed.contains("__PURE__"), "{printed}");
        assert!(!printed.contains("__NO_SIDE_EFFECTS__"), "{printed}");
    }
}

#[test]
fn comments_preserve_expression_start_markers() {
    for source in [
        "const f = () => /* object */ ({ answer: 42 });",
        "/* expression */ ({ answer: 42 });",
        "export default /* function */ (function () {});",
    ] {
        for minify in [false, true] {
            check(source, SourceType::mjs(), minify);
        }
    }
}
