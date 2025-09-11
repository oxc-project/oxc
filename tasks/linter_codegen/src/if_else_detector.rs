use syn::{Expr, ExprIf, Pat, Stmt};

use crate::{
    CollectionResult, NodeTypeSet,
    utils::{astkind_variant_from_path, is_node_kind_call},
};

/// Detects top-level `if let AstKind::... = node.kind()` patterns in the `run` method.
pub struct IfElseKindDetector {
    node_types: NodeTypeSet,
}

impl IfElseKindDetector {
    pub fn from_run_func(run_func: &syn::ImplItemFn) -> Option<NodeTypeSet> {
        // Only consider when the body has exactly one top-level statement and it's an `if`.
        let block = &run_func.block;
        if block.stmts.len() != 1 {
            return None;
        }
        let stmt = &block.stmts[0];
        let Stmt::Expr(Expr::If(ifexpr), _) = stmt else { return None };
        let mut detector = Self { node_types: NodeTypeSet::new() };
        let result = detector.collect_if_chain_variants(ifexpr);
        if result == CollectionResult::Incomplete || detector.node_types.is_empty() {
            return None;
        }
        Some(detector.node_types)
    }

    /// Collects AstKind variants from an if-else chain of `if let AstKind::Xxx(..) = node.kind()`.
    /// Returns `true` if all syntax was recognized as supported, otherwise `false`, indicating that
    /// the variants collected may be incomplete and should not be treated as valid.
    fn collect_if_chain_variants(&mut self, ifexpr: &ExprIf) -> CollectionResult {
        // Extract variants from condition like `if let AstKind::Xxx(..) = node.kind()`.
        if self.extract_variants_from_if_let_condition(&ifexpr.cond) == CollectionResult::Incomplete
        {
            // If syntax is not recognized, return Incomplete.
            return CollectionResult::Incomplete;
        }
        // Walk else-if chain.
        if let Some((_, else_branch)) = &ifexpr.else_branch {
            match &**else_branch {
                Expr::If(nested) => self.collect_if_chain_variants(nested),
                // plain `else { ... }` should default to any node type
                _ => CollectionResult::Incomplete,
            }
        } else {
            CollectionResult::Complete
        }
    }

    /// Extracts AstKind variants from an `if let` condition like `if let AstKind::Xxx(..) = node.kind()`.
    fn extract_variants_from_if_let_condition(&mut self, cond: &Expr) -> CollectionResult {
        let Expr::Let(let_expr) = cond else { return CollectionResult::Incomplete };
        // RHS must be `node.kind()`
        if is_node_kind_call(&let_expr.expr) {
            self.extract_variants_from_pat(&let_expr.pat)
        } else {
            CollectionResult::Incomplete
        }
    }

    fn extract_variants_from_pat(&mut self, pat: &Pat) -> CollectionResult {
        match pat {
            Pat::Or(orpat) => {
                for p in &orpat.cases {
                    if self.extract_variants_from_pat(p) == CollectionResult::Incomplete {
                        return CollectionResult::Incomplete;
                    }
                }
                CollectionResult::Complete
            }
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
