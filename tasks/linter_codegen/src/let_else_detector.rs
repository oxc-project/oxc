use syn::{Expr, Pat, Stmt};

use crate::{
    CollectionResult, RuleRunnerData,
    node_type_set::NodeTypeSet,
    utils::{astkind_variant_from_path, is_node_kind_call},
};

/// Detects top-level `let AstKind::... = node.kind() else { return; }` patterns in the `run` method.
pub struct LetElseDetector<'a> {
    node_types: NodeTypeSet,
    rule_runner_data: &'a RuleRunnerData,
}

impl<'a> LetElseDetector<'a> {
    pub fn from_run_func(
        run_func: &syn::ImplItemFn,
        rule_runner_data: &'a RuleRunnerData,
    ) -> Option<NodeTypeSet> {
        // Only consider when the body's first statement is `let AstKind::... = node.kind() else { ... }`,
        // and the body of the `else` is just `return`.
        let block = &run_func.block;
        let first_stmt = block.stmts.first()?;
        // Must be a local `let` statement
        let Stmt::Local(local) = first_stmt else { return None };
        // Must have an initializer that is a `node.kind()` call
        let Some(init) = &local.init else { return None };
        if !(is_node_kind_call(&init.expr) || is_node_kind_as_call(&init.expr)) {
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

        let mut detector = Self { node_types: NodeTypeSet::new(), rule_runner_data };

        if is_node_kind_as_call(&init.expr) {
            // If the initializer is `node.kind().as_<variant>()`, extract that variant.
            if let Expr::MethodCall(mc) = &*init.expr
                && let Some(variants) = detector.extract_variants_from_as_call(mc)
            {
                detector.node_types.extend(variants);
            }
        } else {
            // Otherwise, the initializer is `node.kind()`, so extract from the pattern.
            // Expecting `AstKind::Variant` pattern
            let result = detector.extract_variants_from_pat(&local.pat);
            if result == CollectionResult::Incomplete {
                return None;
            }
        }
        if detector.node_types.is_empty() {
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

    fn extract_variants_from_as_call(&self, mc: &syn::ExprMethodCall) -> Option<NodeTypeSet> {
        // Looking for `node.kind().as_<snake_case_variant>()`
        let method_ident = mc.method.to_string();
        if !method_ident.starts_with("as_") || !mc.args.is_empty() {
            return None;
        }
        // Receiver must be `node.kind()`
        if !is_node_kind_call(&mc.receiver) {
            return None;
        }
        let snake_variant = &method_ident[3..]; // strip `as_`
        if snake_variant == "member_expression_kind" {
            return Some(self.rule_runner_data.member_expression_kinds.clone());
        }
        let mut node_type_set = NodeTypeSet::new();
        node_type_set.insert(snake_to_pascal_case(snake_variant));
        Some(node_type_set)
    }
}

/// Checks if is `node.kind().as_some_ast_kind()`
pub fn is_node_kind_as_call(expr: &Expr) -> bool {
    if let Expr::MethodCall(mc) = expr
        && mc.method.to_string().starts_with("as_")
        && mc.args.is_empty()
        && is_node_kind_call(&mc.receiver)
    {
        return true;
    }
    false
}

fn snake_to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|seg| !seg.is_empty())
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
