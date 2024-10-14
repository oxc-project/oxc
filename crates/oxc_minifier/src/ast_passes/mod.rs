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
use oxc_traverse::{Traverse, TraverseCtx};

pub trait CompressorPass<'a>: Traverse<'a> {
    fn changed(&self) -> bool;

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>);
}
