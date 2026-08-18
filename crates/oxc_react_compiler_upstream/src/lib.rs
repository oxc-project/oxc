use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::Program;
use oxc_diagnostics::Diagnostics;
use oxc_semantic::SemanticBuilder;
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

extern crate react_compiler_ast_upstream as react_compiler_ast;
extern crate react_compiler_hir_upstream as react_compiler_hir;
extern crate react_compiler_upstream as react_compiler;

mod convert_ast;
mod convert_ast_reverse;
mod convert_scope;
mod diagnostics;
mod prefilter;

use crate::convert_ast::convert_program;
use crate::convert_ast_reverse::convert_program_to_oxc_with_source;
use crate::convert_scope::convert_scope_info;
use crate::diagnostics::compile_result_to_diagnostics;
use crate::prefilter::{has_react_like_functions, has_resource_management_declarations};
use react_compiler::entrypoint::compile_result::CompileResult;
use react_compiler::entrypoint::plugin_options::PluginOptions;
use react_compiler::entrypoint::program::compile_program;

#[derive(Default)]
pub struct TransformResult {
    pub changed: bool,
    pub fatal: bool,
    pub diagnostics: Diagnostics,
}

/// Run the pinned upstream React Compiler on an Oxc program.
///
/// This benchmark-only adapter intentionally pays the Oxc-to-Babel-AST and
/// Babel-AST-to-Oxc conversion costs used by upstream native integrations.
pub fn transform<'a>(
    program: &mut Program<'a>,
    allocator: &'a Allocator,
    filename: &str,
) -> TransformResult {
    if !has_react_like_functions(program) || has_resource_management_declarations(program) {
        return TransformResult::default();
    }

    let semantic = SemanticBuilder::new().with_build_nodes(true).build(program).semantic;
    let file = convert_program(program, program.source_text);
    let scope_info = convert_scope_info(&semantic, program);
    let options = plugin_options(filename);
    let result = compile_program(file, scope_info, options);
    let diagnostics = compile_result_to_diagnostics(&result);

    match result {
        CompileResult::Success { ast, .. } => {
            let Some(file) = ast else {
                return TransformResult { diagnostics, ..TransformResult::default() };
            };
            let mut compiled =
                convert_program_to_oxc_with_source(&file, allocator, program.source_text);
            compiled.source_type = program.source_type;
            preserve_comments(&mut compiled, program, allocator);
            *program = compiled;
            TransformResult { changed: true, fatal: false, diagnostics }
        }
        CompileResult::Error { .. } => TransformResult { changed: false, fatal: true, diagnostics },
    }
}

fn plugin_options(filename: &str) -> PluginOptions {
    serde_json::from_value(serde_json::json!({
        "shouldCompile": true,
        "enableReanimated": false,
        "isDev": false,
        "filename": filename,
        "compilationMode": "infer",
        "panicThreshold": "none",
        "target": "19",
        "gating": null,
        "dynamicGating": null,
        "noEmit": false,
        "outputMode": null,
        "eslintSuppressionRules": null,
        "flowSuppressions": true,
        "ignoreUseNoForget": false,
        "customOptOutDirectives": null,
        "environment": {},
        "sourceCode": null,
        "profiling": false,
        "debug": false
    }))
    .expect("fixed upstream React Compiler options must be valid")
}

fn preserve_comments<'a>(
    compiled: &mut Program<'a>,
    source: &Program<'a>,
    allocator: &'a Allocator,
) {
    let mut top_level_starts = FxHashSet::default();
    top_level_starts.insert(0u32);
    for statement in &compiled.body {
        let start = statement.span().start;
        if start > 0 {
            top_level_starts.insert(start);
        }
    }

    let mut comments = ArenaVec::with_capacity_in(source.comments.len(), &allocator);
    for comment in &source.comments {
        if top_level_starts.contains(&comment.attached_to) {
            comments.push(*comment);
        }
    }
    compiled.comments = comments;
    compiled.source_text = source.source_text;
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_codegen::Codegen;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    #[test]
    fn compiles_component() {
        let source = "function Component(props) { return <div>{props.text}</div>; }";
        let allocator = Allocator::default();
        let mut program = Parser::new(&allocator, source, SourceType::tsx()).parse().program;
        let result = super::transform(&mut program, &allocator, "Component.tsx");

        assert!(!result.fatal, "unexpected diagnostics: {:?}", result.diagnostics);
        assert!(result.changed);
        let code = Codegen::new().build(&program).code;
        assert!(code.contains("react/compiler-runtime"));
        assert!(code.contains("_c("));
    }
}
