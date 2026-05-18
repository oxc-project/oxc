use std::{env, path::PathBuf};

use cow_utils::CowUtils;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::tester::default_options;

#[derive(Clone, Copy)]
struct Position {
    line: u32,
    col: u32,
}

#[derive(Clone, Copy)]
struct Mapping {
    dst: Position,
    src: Position,
}

fn pos(line: u32, col: u32) -> Position {
    Position { line, col }
}

fn sourcemap_tokens(source_text: &str, source_type: SourceType) -> Vec<Mapping> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty(), "parse errors: {:?}", ret.errors);

    Codegen::new()
        .with_options(default_options())
        .build(&ret.program)
        .map
        .expect("sourcemap should be generated")
        .get_tokens()
        .map(|token| Mapping {
            dst: pos(token.get_dst_line(), token.get_dst_col()),
            src: pos(token.get_src_line(), token.get_src_col()),
        })
        .collect()
}

fn has_mapping(tokens: &[Mapping], src: Position, dst: Position) -> bool {
    tokens.iter().any(|token| {
        token.src.line == src.line
            && token.src.col == src.col
            && token.dst.line == dst.line
            && token.dst.col == dst.col
    })
}

fn first_generated_position_for_source(tokens: &[Mapping], src: Position) -> Option<Position> {
    tokens
        .iter()
        .filter(|token| token.src.line == src.line && token.src.col == src.col)
        .map(|token| token.dst)
        .min_by_key(|position| (position.line, position.col))
}

fn assert_source_maps_after_indent(
    tokens: &[Mapping],
    src: Position,
    wrong_dst: Position,
    correct_dst: Position,
) {
    assert!(!has_mapping(tokens, src, wrong_dst));
    assert!(has_mapping(tokens, src, correct_dst));
}

fn assert_member_start_maps_before_key(
    tokens: &[Mapping],
    member_start: Position,
    key_start: Position,
) {
    let member_start = first_generated_position_for_source(tokens, member_start)
        .expect("member start should be mapped");
    let key_start = first_generated_position_for_source(tokens, key_start)
        .expect("member key should be mapped");

    assert_eq!(member_start.line, key_start.line);
    assert!(member_start.col < key_start.col);
}

/// Upstream may have modified the AST to include incorrect spans.
/// e.g. <https://github.com/rolldown/rolldown/blob/v1.0.0-beta.19/crates/rolldown/src/utils/ecma_visitors/mod.rs>
#[test]
fn incorrect_ast() {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let source_text = "foo\nvar bar = '测试'";
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty());

    let mut program = ret.program;
    program.span = Span::new(0, 0);
    if let Statement::ExpressionStatement(s) = &mut program.body[0] {
        s.span = Span::new(17, 17);
        if let Expression::Identifier(ident) = &mut s.expression {
            ident.span = Span::new(17, 17);
        }
    }

    let ret = Codegen::new().with_options(default_options()).build(&program);
    assert!(ret.map.is_some(), "sourcemap exists");
}

/// Test that sourcemaps don't contain invalid tokens for positions beyond source content.
/// This addresses the issue where oxc_codegen adds semicolons/newlines and creates tokens
/// for positions that don't exist in the original source.
/// See: https://github.com/rolldown/rolldown/pull/6750
#[test]
fn no_invalid_tokens_beyond_source() {
    let test_cases = vec![
        // Export statement without trailing semicolon
        "export default { foo }",
        // Variable declaration without trailing semicolon
        "const a = 1",
        // Function without trailing semicolon
        "function foo() { return 42 }",
        // Object with shorthand property
        "const obj = { foo }",
    ];

    for source_text in test_cases {
        let allocator = Allocator::default();
        let source_type = SourceType::mjs();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(ret.errors.is_empty());

        let result = Codegen::new()
            .with_options(CodegenOptions {
                source_map_path: Some(PathBuf::from("test.js")),
                ..Default::default()
            })
            .build(&ret.program);

        let map = result.map.unwrap();
        // Verify all tokens have source positions within bounds
        for token in map.get_tokens() {
            if let Some(source_id) = token.get_source_id()
                && let Some(content) = map.get_source_content(source_id)
            {
                let src_line = token.get_src_line() as usize;
                let src_col = token.get_src_col() as usize;

                let lines: Vec<&str> = content.split('\n').collect();
                assert!(
                    src_line < lines.len(),
                    "Invalid token: line {src_line} is beyond source line count {} for source '{source_text}'",
                    lines.len(),
                );

                let line_content = lines[src_line];
                let line_len_utf16: usize = line_content.chars().map(char::len_utf16).sum();
                assert!(
                    src_col < line_len_utf16,
                    "Invalid token: column {src_col} is beyond line length {line_len_utf16} for line '{line_content}' in source '{source_text}'",
                );
            }
        }
    }
}

#[test]
fn indented_statement_mappings_start_after_generated_indent() {
    let tokens = sourcemap_tokens(
        r"if (foo) {
  bar();
}",
        SourceType::mjs(),
    );

    assert_source_maps_after_indent(&tokens, pos(1, 2), pos(1, 0), pos(1, 1));
}

// `Directive`, `ImportDeclaration`, `ExportNamedDeclaration`,
// `ExportAllDeclaration`, and `ExportDefaultDeclaration` previously called
// `add_source_mapping` *before* `print_indent`, anchoring the mapping at
// gen col 0 (whitespace) instead of the start of the keyword.
#[test]
fn top_level_decl_mappings_start_after_generated_indent() {
    // Wrap the imports/exports in `if (true) { ... }` so the body is
    // indented, exposing the order of `add_source_mapping` vs `print_indent`.
    let tokens = sourcemap_tokens(
        r#"if (true) {
"use strict";
import { x } from "x";
export { x } from "x";
export * from "x";
export default 1;
}"#,
        SourceType::mjs(),
    );

    // Directive `"use strict"` source col 0 of line 1 → gen col 1 (after tab),
    // not gen col 0.
    assert_source_maps_after_indent(&tokens, pos(1, 0), pos(1, 0), pos(1, 1));
    // ImportDeclaration
    assert_source_maps_after_indent(&tokens, pos(2, 0), pos(2, 0), pos(2, 1));
    // ExportNamedDeclaration
    assert_source_maps_after_indent(&tokens, pos(3, 0), pos(3, 0), pos(3, 1));
    // ExportAllDeclaration
    assert_source_maps_after_indent(&tokens, pos(4, 0), pos(4, 0), pos(4, 1));
    // ExportDefaultDeclaration
    assert_source_maps_after_indent(&tokens, pos(5, 0), pos(5, 0), pos(5, 1));
}

#[test]
fn class_member_mappings_start_before_member_keys() {
    let tokens = sourcemap_tokens(
        r"class Foo {
  get value() {
    return 1;
  }

  static load() {
    return 1;
  }
}",
        SourceType::ts(),
    );

    assert_member_start_maps_before_key(&tokens, pos(1, 2), pos(1, 6));
    assert_member_start_maps_before_key(&tokens, pos(5, 2), pos(5, 9));
}

// `print_block_end` wraps a non-block consequent when codegen must preserve
// `else` binding. For parser-produced source this shape is not naturally
// reachable - an outer `else` either binds to the nearest inner `if`, or the
// source uses an explicit block and goes through `print_block_statement`.
// Consumers can still hit this path when printing transformed or hand-built ASTs.
#[test]
fn synthesized_block_closing_braces_are_mapped() {
    let source_text = "if (foo) {\n  if (bar)\n    baz();\n} else\n  qux();";
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(ret.errors.is_empty(), "parse errors: {:?}", ret.errors);

    let mut program = ret.program;
    let Statement::IfStatement(outer_if) = &mut program.body[0] else {
        panic!("expected outer if statement");
    };
    let consequent = {
        let Statement::BlockStatement(block) = &mut outer_if.consequent else {
            panic!("expected block statement");
        };
        assert_eq!(block.body.len(), 1);
        block.body.remove(0)
    };
    outer_if.consequent = consequent;

    let ret = Codegen::new().with_options(default_options()).build(&program);
    assert_eq!(ret.code, "if (foo) {\n\tif (bar) baz();\n} else qux();\n");

    let tokens: Vec<Mapping> = ret
        .map
        .expect("sourcemap should be generated")
        .get_tokens()
        .map(|token| Mapping {
            dst: pos(token.get_dst_line(), token.get_dst_col()),
            src: pos(token.get_src_line(), token.get_src_col()),
        })
        .collect();

    // The emitted `}` after `if (bar) baz();` maps back to the end of `baz();`.
    assert!(
        has_mapping(&tokens, pos(2, 9), pos(2, 0)),
        "expected the synthesized block closing brace to map back to the wrapped if statement",
    );
}

// Both `)` of `factory()()` should map to their own source position
// at the gen position of the `)`, not one past it.
#[test]
fn call_end_mapping_lands_at_close_paren() {
    let tokens = sourcemap_tokens("factory()()", SourceType::mjs());
    assert!(has_mapping(&tokens, pos(0, 8), pos(0, 8)), "inner `)`");
    assert!(has_mapping(&tokens, pos(0, 10), pos(0, 10)), "outer `)`");
}

// `print_block_end`: source `}` → gen `}`, not the `;` that follows.
#[test]
fn block_end_mapping_lands_at_close_brace() {
    let source = "const fn = () => { return 1 }";
    let src_brace = u32::try_from(source.rfind('}').unwrap()).unwrap();
    let tokens = sourcemap_tokens(source, SourceType::mjs());
    assert!(has_mapping(&tokens, pos(0, src_brace), pos(2, 0)));
}

// `print_curly_braces` (shared by class body, switch, TS enum/interface/
// typeliteral/module): pin one to cover the shared helper.
#[test]
fn class_body_close_brace_lands_at_close_brace() {
    let source = "class C { a; }";
    let src_brace = u32::try_from(source.rfind('}').unwrap()).unwrap();
    let tokens = sourcemap_tokens(source, SourceType::mjs());
    assert!(has_mapping(&tokens, pos(0, src_brace), pos(2, 0)));
}

#[test]
fn array_close_bracket_lands_at_close_bracket() {
    let source = "const a = [1, 2]";
    let src_bracket = u32::try_from(source.rfind(']').unwrap()).unwrap();
    let tokens = sourcemap_tokens(source, SourceType::mjs());
    assert!(has_mapping(&tokens, pos(0, src_bracket), pos(0, src_bracket)));
}

#[test]
fn object_close_brace_lands_at_close_brace() {
    let source = "const o = { a: 1 }";
    let src_brace = u32::try_from(source.rfind('}').unwrap()).unwrap();
    let tokens = sourcemap_tokens(source, SourceType::mjs());
    assert!(has_mapping(&tokens, pos(0, src_brace), pos(0, src_brace)));
}

#[test]
#[cfg(all(not(target_endian = "big"), target_pointer_width = "64"))] // we run big endian tests on docker that does not have node installed; skip 32-bit as well
fn stacktrace_is_correct() {
    let cases = &[
        "\
const fn = () => {
    Error.stackTraceLimit = 2;
    throw new Error()
};
fn()",
        "\
const obj = {
    fn() {
        Error.stackTraceLimit = 2;
        throw new Error()
    }
}
obj.fn()",
        "\
const obj = {
    obj2: {
        fn() {
            Error.stackTraceLimit = 2;
            throw new Error()
        }
    }
}
obj.obj2.fn()",
        "\
const obj = {
    fn() {
        return function fn2() {
            Error.stackTraceLimit = 2;
            throw new Error()
        }
    }
}
obj.fn()()",
        "\
const obj = {
    fn() {
        return () => {
            Error.stackTraceLimit = 2;
            throw new Error()
        }
    }
}
obj.fn([1])()",
        "\
const factory = () => {
    return () => {
        Error.stackTraceLimit = 2;
        throw new Error()
    }
}
factory()()",
        "\
var a
const obj = {
    fn() {
        return () => {
            Error.stackTraceLimit = 2;
            throw new Error()
        }
    }
}
obj.fn({a})()",
        "\
const fn = (name, cb) => {
    cb()
}
fn('name', () => {
    Error.stackTraceLimit = 2;
    throw new Error()
})",
    ];

    let node_version = std::process::Command::new("node").arg("--version").output().map_or_else(
        |_| "unknown".to_string(),
        |output| String::from_utf8_lossy(&output.stdout).trim().to_string(),
    );

    insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
        insta::assert_snapshot!(
            "stacktrace_is_correct",
            format!(
                "Node.js version: {}\n\n{}",
                node_version,
                cases.iter().map(|s| {
                    let (output, sourcemap_url) = codegen(s);
                    format!("## Input\n{}\n\n## Output\n{}\n\n## Stderr\n{}", s, output, execute_with_node(&output, &sourcemap_url))
                }).collect::<Vec<_>>().join("\n------------------------------------------------------\n")
            )
        );
    });
}

fn codegen(code: &str) -> (String, String) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, code, SourceType::mjs()).parse();
    assert!(ret.errors.is_empty());
    let ret = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: Some(PathBuf::from("input.js")),
            ..Default::default()
        })
        .build(&ret.program);
    (ret.code, ret.map.unwrap().to_data_url())
}

fn execute_with_node(code: &str, sourcemap_url: &str) -> String {
    let cwd = env::current_dir().unwrap().join("input.js");
    let cwd = cwd.to_str().unwrap();

    let code = format!("{code}\n//# sourceMappingURL={sourcemap_url}\n");

    let output = std::process::Command::new("node")
        .arg("--enable-source-maps")
        .args(["--input-type", "module"])
        .args(["--eval", &code])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stderr)
        .cow_replace(cwd, "/project/input.js")
        .lines()
        .filter(|line| !line.starts_with("Node.js v"))
        .collect::<Vec<_>>()
        .join("\n")
}
