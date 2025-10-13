use syn::{Arm, Expr, Pat, Stmt};

use crate::{
    CollectionResult,
    node_type_set::NodeTypeSet,
    utils::{astkind_variant_from_path, is_node_kind_call},
};

/// Detects top-level `match node.kind() { ... }` patterns in the `run` method.
pub struct MatchDetector {
    node_types: NodeTypeSet,
}

impl MatchDetector {
    pub fn from_run_func(run_func: &syn::ImplItemFn) -> Option<NodeTypeSet> {
        // Only consider when the body's only statement is `match node.kind() { ... }`
        let block = &run_func.block;
        if block.stmts.len() != 1 {
            return None;
        }
        let first_stmt = &block.stmts[0];
        // Must be an expression statement
        let Stmt::Expr(expr, _) = first_stmt else { return None };
        // Must be a match expression
        let Expr::Match(match_expr) = expr else { return None };
        // The match expression's must be matching on a `node.kind()` call
        if !is_node_kind_call(&match_expr.expr) {
            return None;
        }

        let mut detector = Self { node_types: NodeTypeSet::new() };
        let result = detector.extract_variants_from_match_expr(match_expr);
        if detector.node_types.is_empty() || result == CollectionResult::Incomplete {
            return None;
        }

        Some(detector.node_types)
    }

    fn extract_variants_from_match_expr(
        &mut self,
        match_expr: &syn::ExprMatch,
    ) -> CollectionResult {
        let mut overall_result = CollectionResult::Complete;
        for arm in &match_expr.arms {
            let result = self.extract_variants_from_match_arm(arm);
            if result == CollectionResult::Incomplete {
                overall_result = CollectionResult::Incomplete;
            }
        }
        overall_result
    }

    fn extract_variants_from_match_arm(&mut self, arm: &Arm) -> CollectionResult {
        let pat = &arm.pat;
        match pat {
            Pat::TupleStruct(ts) => {
                if let Some(variant) = astkind_variant_from_path(&ts.path) {
                    // NOTE: If there is a guard, we assume that it may or may not be taken and collect all AST kinds
                    // regardless of the guard condition.
                    self.node_types.insert(variant);
                    CollectionResult::Complete
                } else {
                    CollectionResult::Incomplete
                }
            }
            Pat::Wild(_) => {
                // Body must be completely empty.
                if let Expr::Block(block) = &*arm.body {
                    if block.block.stmts.is_empty() {
                        CollectionResult::Complete
                    } else {
                        CollectionResult::Incomplete
                    }
                } else {
                    CollectionResult::Incomplete
                }
            }
            _ => CollectionResult::Incomplete,
        }
    }
}
