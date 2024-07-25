use oxc_ast::AstKind;
use oxc_semantic::SymbolFlags;

use crate::{
    semantic::{SymbolTestCase, TestName},
    suite::TestResult,
};

#[derive(Debug, Default, Clone)]
pub struct FunctionDecls;

impl TestName for FunctionDecls {
    fn name(&self) -> &'static str {
        "symbols/function_decls"
    }
}

impl SymbolTestCase for FunctionDecls {
    fn run_on_symbol(
        &self,
        symbol_id: oxc_semantic::SymbolId,
        semantic: &oxc_semantic::Semantic<'_>,
    ) -> crate::suite::TestResult {
        let flags = semantic.symbols().get_flag(symbol_id);

        // check function declarations. Function expressions are not BlockScopedVariables
        if flags.is_function() && flags.contains(SymbolFlags::BlockScopedVariable) {
            let decl = semantic.nodes().get_node(semantic.symbols().get_declaration(symbol_id));
            if matches!(decl.kind(), AstKind::Function(_)) {
                TestResult::Passed
            } else {
                TestResult::SemanticError("Declaration for symbol flagged as a function does not point to a Function AST node.".to_string())
            }
        } else {
            TestResult::Passed
        }
    }
}
