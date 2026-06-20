//! Dump the React Compiler's internal IR after each pipeline pass.
//!
//! Enables `options.debug`, runs the pipeline, and prints every debug log entry
//! (the compiler emits an HIR/reactive-function dump after most passes).
//!
//! Usage:
//!   cargo run -p oxc_react_compiler --example react_compiler_debug -- <FILE> [pass_name]
//!
//! With no `pass_name`, prints every pass. With one, prints only passes whose
//! name exactly matches (e.g. `HIR`, `SSA`, `InferTypes`).

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use oxc_ast::AstBuilder;
use oxc_ast::AstKind;

use oxc_react_compiler::convert_ast::convert_program;
use oxc_react_compiler::convert_scope::convert_scope_info;
use oxc_react_compiler::default_plugin_options;
use oxc_react_compiler::react_compiler::entrypoint::compile_result::{
    CompileResult, OrderedLogItem,
};
use oxc_react_compiler::react_compiler::entrypoint::program::compile_program;
use oxc_react_compiler::react_compiler_lowering::FunctionNode;

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| {
        eprintln!("usage: react_compiler_debug <FILE> [pass_name]");
        std::process::exit(1);
    });
    let filter = args.next();

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{name}: {e}"));
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::tsx());

    let allocator = Allocator::default();
    let program = Parser::new(&allocator, &source_text, source_type).parse().program;
    let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;

    let file = convert_program(&program, &source_text);
    let scope_info = convert_scope_info(&semantic, &program);

    // Map each function's node_id (== span.start) to its oxc node (as in `transform`).
    let mut fn_map = rustc_hash::FxHashMap::default();
    for node in semantic.nodes() {
        match node.kind() {
            AstKind::Function(func) => {
                fn_map.insert(func.span.start, FunctionNode::Function(func));
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                fn_map.insert(arrow.span.start, FunctionNode::Arrow(arrow));
            }
            _ => {}
        }
    }

    let mut options = default_plugin_options();
    options.debug = true;

    let ast_builder = AstBuilder::new(&allocator);
    let result = compile_program(&ast_builder, &program, file, scope_info, options, &fn_map);
    let ordered_log = match &result {
        CompileResult::Success { ordered_log, .. } | CompileResult::Error { ordered_log, .. } => {
            ordered_log
        }
    };

    let mut printed = 0;
    for item in ordered_log {
        if let OrderedLogItem::Debug { entry } = item {
            if let Some(f) = &filter {
                if &entry.name != f {
                    continue;
                }
            }
            println!("\x1b[1;36m===== after pass: {} =====\x1b[0m", entry.name);
            println!("{}", entry.value);
            printed += 1;
        }
    }
    if printed == 0 {
        eprintln!("(no debug entries — no React component/hook found, or pass name didn't match)");
    }
}
