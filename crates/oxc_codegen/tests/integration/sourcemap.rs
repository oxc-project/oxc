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

    insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
        insta::assert_snapshot!(
            "stacktrace_is_correct",
            cases.iter().map(|s| {
                let (output, sourcemap_url) = codegen(s);
                format!("## Input\n{}\n\n## Output\n{}\n\n## Stderr\n{}", s, output, execute_with_node(&output, &sourcemap_url))
            }).collect::<Vec<_>>().join("\n------------------------------------------------------\n")
        );
    });
}

fn codegen(code: &str) -> (String, String) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, code, SourceType::mjs()).parse();
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
