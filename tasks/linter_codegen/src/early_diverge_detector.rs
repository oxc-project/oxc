use syn::{Arm, Expr, Pat, Stmt};

use crate::{
    CollectionResult, NodeTypeSet,
    utils::{astkind_variant_from_path, is_node_kind_call},
};

/// Detects various kinds of diverging statements that narrow by more than one AST node type.
pub struct EarlyDivergeDetector {
    node_types: NodeTypeSet,
}

impl EarlyDivergeDetector {
    pub fn from_run_func(run_func: &syn::ImplItemFn) -> Option<NodeTypeSet> {
        // Only look at cases where the body has more than one top-level statement.
        let block = &run_func.block;
        if block.stmts.len() <= 1 {
            return None;
        }

        // Look at the first statement in the function body.
        let stmt = block.stmts.first()?;

        // Check if it's `let something = match node.kind() { ... }` that diverges
        if let Stmt::Local(local) = stmt
            && let Some(init) = &local.init
            && let Expr::Match(match_expr) = &*init.expr
            && is_node_kind_call(&match_expr.expr)
        {
            let mut detector = Self { node_types: NodeTypeSet::new() };
            let result = detector.extract_variants_from_diverging_match_expr(match_expr);
            if result == CollectionResult::Incomplete || detector.node_types.is_empty() {
                return None;
            }
            return Some(detector.node_types);
        }
        // Stand-alone `match node.kind() { ... }` that diverges
        else if let Stmt::Expr(Expr::Match(match_expr), _) = stmt
            && is_node_kind_call(&match_expr.expr)
        {
            let mut detector = Self { node_types: NodeTypeSet::new() };
            let result = detector.extract_variants_from_diverging_match_expr(match_expr);
            if result == CollectionResult::Incomplete || detector.node_types.is_empty() {
                return None;
            }
            return Some(detector.node_types);
        }

        None
    }

    fn extract_variants_from_diverging_match_expr(
        &mut self,
        match_expr: &syn::ExprMatch,
    ) -> CollectionResult {
        let mut overall_result = CollectionResult::Complete;
        for arm in &match_expr.arms {
            let result = self.extract_variants_from_diverging_match_arm(arm);
            if result == CollectionResult::Incomplete {
                overall_result = CollectionResult::Incomplete;
            }
        }
        overall_result
    }

    fn extract_variants_from_diverging_match_arm(&mut self, arm: &Arm) -> CollectionResult {
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
                // Body must be diverging (i.e., returning from function)
                if let Expr::Return(_) = *arm.body {
                    return CollectionResult::Complete;
                }
                CollectionResult::Incomplete
            }
            Pat::Or(or) => {
                let mut overall_result = CollectionResult::Complete;
                for case in &or.cases {
                    let result = match case {
                        Pat::TupleStruct(ts) => {
                            if let Some(variant) = astkind_variant_from_path(&ts.path) {
                                self.node_types.insert(variant);
                                CollectionResult::Complete
                            } else {
                                CollectionResult::Incomplete
                            }
                        }
                        _ => CollectionResult::Incomplete,
                    };
                    if result == CollectionResult::Incomplete {
                        overall_result = CollectionResult::Incomplete;
                    }
                }
                overall_result
            }
            _ => CollectionResult::Incomplete,
        }
    }
}
