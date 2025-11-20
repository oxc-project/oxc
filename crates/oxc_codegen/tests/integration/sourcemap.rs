use std::{env, path::PathBuf};

use cow_utils::CowUtils;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::tester::default_options;

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
#[cfg(not(target_endian = "big"))] // we run big endian tests on docker that does not have node installed
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
