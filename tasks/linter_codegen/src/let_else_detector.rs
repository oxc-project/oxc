use syn::{Expr, Pat, Stmt};

use crate::{
    CollectionResult,
    node_type_set::NodeTypeSet,
    utils::{astkind_variant_from_path, is_node_kind_call},
};

/// Detects top-level `let AstKind::... = node.kind() else { return; }` patterns in the `run` method.
pub struct LetElseDetector {
    node_types: NodeTypeSet,
}

impl LetElseDetector {
    pub fn from_run_func(run_func: &syn::ImplItemFn) -> Option<NodeTypeSet> {
        // Only consider when the body's first statement is `let AstKind::... = node.kind() else { ... }`,
        // and the body of the `else` is just `return`.
        let block = &run_func.block;
        let first_stmt = block.stmts.first()?;
        // Must be a local `let` statement
        let Stmt::Local(local) = first_stmt else { return None };
        // Must have an initializer that is a `node.kind()` call
        let Some(init) = &local.init else { return None };
        if !is_node_kind_call(&init.expr) {
            return None;
        }
        // Must have a diverging `else` block
        let Some(diverge) = &init.diverge else { return None };
        let diverge_expr = &*diverge.1;
        let Expr::Block(else_block) = diverge_expr else { return None };
        // The else block must have exactly one statement and it must be a `return`
        if else_block.block.stmts.len() != 1 {
            return None;
        }
        let Stmt::Expr(expr, _) = &else_block.block.stmts[0] else { return None };
        if !matches!(expr, Expr::Return(_)) {
            return None;
        }

        let mut detector = Self { node_types: NodeTypeSet::new() };
        let result = detector.extract_variants_from_pat(&local.pat);
        if detector.node_types.is_empty() || result == CollectionResult::Incomplete {
            return None;
        }

        Some(detector.node_types)
    }

    fn extract_variants_from_pat(&mut self, pat: &Pat) -> CollectionResult {
        match pat {
            Pat::TupleStruct(ts) => {
                if let Some(variant) = astkind_variant_from_path(&ts.path) {
                    self.node_types.insert(variant);
                    CollectionResult::Complete
                } else {
                    CollectionResult::Incomplete
                }
            }
            _ => CollectionResult::Incomplete,
        }
    }
}
