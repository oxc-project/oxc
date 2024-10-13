mod collapse_variable_declarations;
mod exploit_assigns;
mod peephole_fold_constants;
mod peephole_minimize_conditions;
mod peephole_remove_dead_code;
mod peephole_replace_known_methods;
mod peephole_substitute_alternate_syntax;
mod remove_syntax;
mod statement_fusion;

pub use collapse_variable_declarations::CollapseVariableDeclarations;
pub use exploit_assigns::ExploitAssigns;
pub use peephole_fold_constants::PeepholeFoldConstants;
pub use peephole_minimize_conditions::PeepholeMinimizeConditions;
pub use peephole_remove_dead_code::PeepholeRemoveDeadCode;
pub use peephole_replace_known_methods::PeepholeReplaceKnownMethods;
pub use peephole_substitute_alternate_syntax::PeepholeSubstituteAlternateSyntax;
pub use remove_syntax::RemoveSyntax;
pub use statement_fusion::StatementFusion;

use oxc_ast::ast::Program;
use oxc_semantic::{ScopeTree, SymbolTable};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::node_util::NodeUtil;

impl<'a> NodeUtil<'a> for TraverseCtx<'a> {
    fn symbols(&self) -> &SymbolTable {
        self.scoping.symbols()
    }

    fn scopes(&self) -> &ScopeTree {
        self.scoping.scopes()
    }
}

pub trait CompressorPass<'a>: Traverse<'a> {
    fn changed(&self) -> bool;

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>);
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Statement;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    #[derive(Default)]
    pub struct Tester;

    fn build<'a>(allocator: &'a Allocator, source_text: &'a str) -> (TraverseCtx<'a>, Program<'a>) {
        let source_type = SourceType::mjs();

        let program = Parser::new(allocator, source_text, source_type).parse().program;
        let (symbols, scopes) =
            SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();

        (TraverseCtx::new(scopes, symbols, allocator), program)
    }

    fn get_string_literal(source_text: &str) -> Option<String> {
        let allocator = Allocator::default();
        let (ctx, program) = build(&allocator, source_text);

        let Some(Statement::ExpressionStatement(expr_stmt)) = program.body.first() else {
            return None;
        };

        ctx.get_string_literal(&expr_stmt.expression).map(Into::into)
    }

    #[test]
    fn test_get_string_literal() {
        assert_eq!(get_string_literal("`abc`"), Some("abc".to_string()));
        assert_ne!(get_string_literal("`a${b}`"), Some("ab".to_string()));
        assert_eq!(get_string_literal("`${null}`"), Some("null".to_string()));
        assert_eq!(get_string_literal("`${undefined}`"), Some("undefined".to_string()));
        assert_eq!(get_string_literal("`${{}}123`"), Some("[object Object]123".to_string()));
        assert_eq!(get_string_literal("`a${1}${true}${NaN}0`"), Some("a1trueNaN0".to_string()));

        // assert_eq!(get_string_literal("`${1,2}`"), Some("2".to_string()));
        // assert_eq!(get_string_literal("`${[]}${[1,2]}`"), Some("1,2".to_string()));
        // assert_eq!(get_string_literal("`${new Set()}`"), Some("[object Set]".to_string()));
    }
}
