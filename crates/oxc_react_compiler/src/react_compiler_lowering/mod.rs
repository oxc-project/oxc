pub mod build_hir;
pub mod find_context_identifiers;
pub mod hir_builder;
pub mod identifier_loc_index;

use crate::react_compiler_ast::expressions::ArrowFunctionExpression;
use crate::react_compiler_ast::expressions::FunctionExpression;
use crate::react_compiler_ast::scope::BindingKind as AstBindingKind;
use crate::react_compiler_ast::statements::FunctionDeclaration;
use crate::react_compiler_hir::BindingKind;

/// Convert AST binding kind to HIR binding kind.
pub fn convert_binding_kind(kind: &AstBindingKind) -> BindingKind {
    match kind {
        AstBindingKind::Var => BindingKind::Var,
        AstBindingKind::Let => BindingKind::Let,
        AstBindingKind::Const => BindingKind::Const,
        AstBindingKind::Param => BindingKind::Param,
        AstBindingKind::Module => BindingKind::Module,
        AstBindingKind::Hoisted => BindingKind::Hoisted,
        AstBindingKind::Local => BindingKind::Local,
        AstBindingKind::Unknown => BindingKind::Unknown,
    }
}

/// Represents a reference to a function AST node for lowering.
/// Analogous to TS's `NodePath<t.Function>` / `BabelFn`.
pub enum FunctionNode<'a> {
    FunctionDeclaration(&'a FunctionDeclaration),
    FunctionExpression(&'a FunctionExpression),
    ArrowFunctionExpression(&'a ArrowFunctionExpression),
}

impl<'a> FunctionNode<'a> {
    /// Get the node_id of the function node. Panics if not set.
    pub fn node_id(&self) -> Option<u32> {
        match self {
            FunctionNode::FunctionDeclaration(d) => d.base.node_id,
            FunctionNode::FunctionExpression(e) => e.base.node_id,
            FunctionNode::ArrowFunctionExpression(a) => a.base.node_id,
        }
    }
}

// The main lower() function - delegates to build_hir
pub use build_hir::lower;
// Re-export post-build helper functions used by optimization passes
pub use crate::react_compiler_hir::visitors::each_terminal_successor;
pub use crate::react_compiler_hir::visitors::terminal_fallthrough;
pub use hir_builder::{
    create_temporary_place, get_reverse_postordered_blocks, mark_instruction_ids,
    mark_predecessors, remove_dead_do_while_statements, remove_unnecessary_try_catch,
    remove_unreachable_for_updates,
};
