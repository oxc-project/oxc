use syn::{Arm, Expr, Pat, Stmt};

use crate::{
    CollectionResult, RuleRunnerData,
    node_type_set::NodeTypeSet,
    utils::{astkind_variant_from_path, is_node_kind_call},
};

/// Detects top-level `match node.kind() { ... }` patterns in the `run` method.
pub struct MatchDetector<'a> {
    node_types: NodeTypeSet,
    rule_runner_data: &'a RuleRunnerData,
}

impl<'a> MatchDetector<'a> {
    pub fn from_run_func(
        run_func: &syn::ImplItemFn,
        rule_runner_data: &'a RuleRunnerData,
    ) -> Option<NodeTypeSet> {
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

        let mut detector = Self { node_types: NodeTypeSet::new(), rule_runner_data };
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
                // Body must be completely empty, or return empty type only `()`
                if let Expr::Block(block) = &*arm.body {
                    if block.block.stmts.is_empty() {
                        CollectionResult::Complete
                    } else {
                        CollectionResult::Incomplete
                    }
                } else if let Expr::Tuple(tuple) = &*arm.body {
                    if tuple.elems.is_empty() {
                        CollectionResult::Complete
                    } else {
                        CollectionResult::Incomplete
                    }
                } else {
                    CollectionResult::Incomplete
                }
            }
            Pat::Ident(ident) => {
                if ident.subpat.is_some() {
                    return CollectionResult::Incomplete;
                }
                // Look for a `member_expr if member_expr.is_member_expression_kind() => {` arm
                if let Some((_, guard_expr)) = &arm.guard
                    && let Expr::MethodCall(method_call) = &**guard_expr
                    && method_call.method == "is_member_expression_kind"
                    && method_call.args.is_empty()
                {
                    // We have a match, so we can add MemberExpression to the set of node types
                    self.node_types.extend(self.rule_runner_data.member_expression_kinds.clone());
                    CollectionResult::Complete
                } else {
                    CollectionResult::Incomplete
                }
            }
            _ => CollectionResult::Incomplete,
        }
    }
}
