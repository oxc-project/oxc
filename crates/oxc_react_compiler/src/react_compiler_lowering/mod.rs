pub mod build_hir;
pub mod find_context_identifiers;
pub mod hir_builder;
pub mod identifier_loc_index;
pub mod source_loc;

use oxc_ast::ast as oxc;

use crate::react_compiler_hir::BindingKind;

/// Convert AST binding kind to HIR binding kind.
pub fn convert_binding_kind(kind: &crate::scope::BindingKind) -> BindingKind {
    match kind {
        crate::scope::BindingKind::Var => BindingKind::Var,
        crate::scope::BindingKind::Let => BindingKind::Let,
        crate::scope::BindingKind::Const => BindingKind::Const,
        crate::scope::BindingKind::Param => BindingKind::Param,
        crate::scope::BindingKind::Module => BindingKind::Module,
        crate::scope::BindingKind::Hoisted => BindingKind::Hoisted,
        crate::scope::BindingKind::Local => BindingKind::Local,
        crate::scope::BindingKind::Unknown => BindingKind::Unknown,
    }
}

/// Represents a reference to a function AST node for lowering.
/// Analogous to TS's `NodePath<t.Function>` / `BabelFn`.
///
/// oxc collapses Babel's `FunctionDeclaration`/`FunctionExpression` into one
/// [`oxc::Function`] (discriminated by `r#type`); arrows are separate.
#[derive(Clone, Copy)]
pub enum FunctionNode<'a> {
    Function(&'a oxc::Function<'a>),
    Arrow(&'a oxc::ArrowFunctionExpression<'a>),
}

impl<'a> FunctionNode<'a> {
    /// The node_id of the function node, equal to its `span.start`.
    pub fn node_id(&self) -> Option<u32> {
        Some(match self {
            FunctionNode::Function(f) => f.span.start,
            FunctionNode::Arrow(a) => a.span.start,
        })
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
