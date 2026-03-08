use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::GetAddress;
use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, Reference, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unmodified_loop_condition_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is not modified in this loop.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnmodifiedLoopCondition;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow references in loop conditions that are never modified within the loop.
    ///
    /// ### Why is this bad?
    ///
    /// A loop condition that depends on values that never change within the loop body
    /// can cause infinite loops or logic bugs.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let done = false;
    /// while (!done) {
    ///   work();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let done = false;
    /// while (!done) {
    ///   done = checkDone();
    /// }
    /// ```
    NoUnmodifiedLoopCondition,
    eslint,
    suspicious
);

impl Rule for NoUnmodifiedLoopCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::WhileStatement(statement) => {
                let loop_info =
                    LoopInfo { span: statement.span, node_id: node.id(), kind: LoopKind::While };
                Self::check_loop_condition(&statement.test, &loop_info, ctx);
            }
            AstKind::DoWhileStatement(statement) => {
                let loop_info =
                    LoopInfo { span: statement.span, node_id: node.id(), kind: LoopKind::DoWhile };
                Self::check_loop_condition(&statement.test, &loop_info, ctx);
            }
            AstKind::ForStatement(statement) => {
                let Some(test) = &statement.test else {
                    return;
                };
                let loop_info = LoopInfo {
                    span: statement.span,
                    node_id: node.id(),
                    kind: LoopKind::For { init_span: statement.init.as_ref().map(GetSpan::span) },
                };
                Self::check_loop_condition(test, &loop_info, ctx);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LoopInfo {
    span: Span,
    node_id: NodeId,
    kind: LoopKind,
}

#[derive(Debug, Clone, Copy)]
enum LoopKind {
    While,
    DoWhile,
    For { init_span: Option<Span> },
}

impl LoopInfo {
    fn is_in_loop(&self, reference: &Reference, ctx: &LintContext<'_>) -> bool {
        let reference_span = ctx.semantic().reference_span(reference);
        if !self.span.contains_inclusive(reference_span) {
            return false;
        }
        let is_in_loop_range = match self.kind {
            LoopKind::For { init_span: Some(init_span) } => {
                !init_span.contains_inclusive(reference_span)
            }
            _ => true,
        };

        is_in_loop_range && !self.is_in_nested_function_scope(reference, ctx)
    }

    fn is_in_nested_function_scope(&self, reference: &Reference, ctx: &LintContext<'_>) -> bool {
        for (ancestor_id, ancestor) in ctx.nodes().ancestors_enumerated(reference.node_id()) {
            if ancestor_id == self.node_id {
                return false;
            }
            if matches!(ancestor.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
            {
                return true;
            }
        }

        false
    }
}

impl NoUnmodifiedLoopCondition {
    fn check_loop_condition<'a>(
        condition: &Expression<'a>,
        loop_info: &LoopInfo,
        ctx: &LintContext<'a>,
    ) {
        let mut collector = ConditionSymbolsCollector::new(ctx);
        collector.visit_expression(condition);

        let mut standalone_symbols: Vec<(SymbolId, NodeId)> = vec![];
        let mut standalone_seen: FxHashSet<SymbolId> = FxHashSet::default();
        let mut grouped_symbols: FxHashMap<Span, Vec<(SymbolId, NodeId)>> = FxHashMap::default();
        let mut grouped_seen: FxHashMap<Span, FxHashSet<SymbolId>> = FxHashMap::default();
        let mut group_order: Vec<Span> = vec![];
        for symbol in collector.symbols {
            if let Some(group_span) = symbol.group_span {
                if let std::collections::hash_map::Entry::Vacant(entry) =
                    grouped_symbols.entry(group_span)
                {
                    entry.insert(vec![]);
                    group_order.push(group_span);
                }
                let seen = grouped_seen.entry(group_span).or_default();
                if seen.insert(symbol.symbol_id) {
                    grouped_symbols
                        .entry(group_span)
                        .or_default()
                        .push((symbol.symbol_id, symbol.reference_node_id));
                }
            } else if standalone_seen.insert(symbol.symbol_id) {
                standalone_symbols.push((symbol.symbol_id, symbol.reference_node_id));
            }
        }

        let mut modified_cache: FxHashMap<SymbolId, bool> = FxHashMap::default();
        let mut diagnostics: Vec<NodeId> = vec![];
        let mut reported_symbols: FxHashSet<SymbolId> = FxHashSet::default();

        for (symbol_id, reference_node_id) in standalone_symbols {
            if Self::is_symbol_modified_cached(symbol_id, &mut modified_cache, loop_info, ctx) {
                continue;
            }
            if reported_symbols.insert(symbol_id) {
                diagnostics.push(reference_node_id);
            }
        }

        for group_span in group_order {
            if collector.dynamic_groups.contains(&group_span) {
                continue;
            }
            let Some(symbols) = grouped_symbols.get(&group_span) else {
                continue;
            };
            let has_modified_symbol = symbols.iter().any(|(symbol_id, _)| {
                Self::is_symbol_modified_cached(*symbol_id, &mut modified_cache, loop_info, ctx)
            });
            if has_modified_symbol {
                continue;
            }
            for (symbol_id, reference_node_id) in symbols {
                if reported_symbols.insert(*symbol_id) {
                    diagnostics.push(*reference_node_id);
                }
            }
        }

        for reference_node_id in diagnostics {
            Self::report_condition(reference_node_id, ctx);
        }
    }

    fn report_condition(reference_node_id: NodeId, ctx: &LintContext<'_>) {
        let node = ctx.nodes().get_node(reference_node_id);
        let AstKind::IdentifierReference(ident) = node.kind() else {
            return;
        };
        ctx.diagnostic(no_unmodified_loop_condition_diagnostic(ident.name.as_str(), ident.span));
    }

    fn is_symbol_modified_in_loop(
        symbol_id: SymbolId,
        loop_info: &LoopInfo,
        ctx: &LintContext<'_>,
    ) -> bool {
        for reference in ctx.scoping().get_resolved_references(symbol_id) {
            if !reference.is_write() {
                continue;
            }

            if loop_info.is_in_loop(reference, ctx)
                || Self::is_modified_via_called_function_declaration(loop_info, reference, ctx)
            {
                return true;
            }
        }
        false
    }

    fn is_symbol_modified_cached(
        symbol_id: SymbolId,
        modified_cache: &mut FxHashMap<SymbolId, bool>,
        loop_info: &LoopInfo,
        ctx: &LintContext<'_>,
    ) -> bool {
        if let Some(is_modified) = modified_cache.get(&symbol_id) {
            return *is_modified;
        }
        let is_modified = Self::is_symbol_modified_in_loop(symbol_id, loop_info, ctx);
        modified_cache.insert(symbol_id, is_modified);
        is_modified
    }

    fn is_modified_via_called_function_declaration(
        loop_info: &LoopInfo,
        modifier: &Reference,
        ctx: &LintContext<'_>,
    ) -> bool {
        let Some(function_symbol_id) =
            Self::get_enclosing_function_declaration_symbol_id(modifier.node_id(), ctx)
        else {
            return false;
        };

        for function_reference in ctx.scoping().get_resolved_references(function_symbol_id) {
            if loop_info.is_in_loop(function_reference, ctx)
                && Self::is_function_invocation_reference(function_reference, ctx)
            {
                return true;
            }
        }

        false
    }

    fn get_enclosing_function_declaration_symbol_id(
        node_id: NodeId,
        ctx: &LintContext<'_>,
    ) -> Option<SymbolId> {
        let nodes = ctx.nodes();
        let mut current_id = node_id;

        while current_id != NodeId::ROOT {
            let current_node = nodes.get_node(current_id);
            if let AstKind::Function(function) = current_node.kind()
                && function.is_declaration()
            {
                return function.id.as_ref().map(oxc_ast::ast::BindingIdentifier::symbol_id);
            }
            current_id = nodes.parent_id(current_id);
        }

        None
    }

    fn is_function_invocation_reference(reference: &Reference, ctx: &LintContext<'_>) -> bool {
        let reference_node_id = reference.node_id();
        let reference_node = ctx.nodes().get_node(reference_node_id);

        match ctx.nodes().parent_kind(reference_node_id) {
            AstKind::CallExpression(call_expression) => {
                call_expression.callee.address() == reference_node.address()
            }
            AstKind::NewExpression(new_expression) => {
                new_expression.callee.address() == reference_node.address()
            }
            AstKind::TaggedTemplateExpression(tagged_template_expression) => {
                tagged_template_expression.tag.address() == reference_node.address()
            }
            _ => false,
        }
    }
}

struct ConditionSymbolsCollector<'a, 'ctx> {
    ctx: &'ctx LintContext<'a>,
    symbols: Vec<ConditionSymbolInfo>,
    group_stack: Vec<Span>,
    dynamic_groups: FxHashSet<Span>,
}

impl<'a, 'ctx> ConditionSymbolsCollector<'a, 'ctx> {
    fn new(ctx: &'ctx LintContext<'a>) -> Self {
        Self { ctx, symbols: vec![], group_stack: vec![], dynamic_groups: FxHashSet::default() }
    }
}

impl<'a> Visit<'a> for ConditionSymbolsCollector<'a, '_> {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        let is_group_expression = matches!(
            expression,
            Expression::BinaryExpression(_) | Expression::ConditionalExpression(_)
        );
        if is_group_expression {
            self.group_stack.push(expression.span());
        }

        if matches!(
            expression,
            Expression::CallExpression(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::PrivateFieldExpression(_)
                | Expression::NewExpression(_)
                | Expression::TaggedTemplateExpression(_)
                | Expression::YieldExpression(_)
        ) {
            if let Some(group_span) = self.group_stack.first().copied() {
                self.dynamic_groups.insert(group_span);
            }
            if is_group_expression {
                self.group_stack.pop();
            }
            return;
        }

        if matches!(
            expression,
            Expression::FunctionExpression(_)
                | Expression::ArrowFunctionExpression(_)
                | Expression::ClassExpression(_)
        ) {
            if is_group_expression {
                self.group_stack.pop();
            }
            return;
        }

        walk::walk_expression(self, expression);
        if is_group_expression {
            self.group_stack.pop();
        }
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        let reference_id = identifier.reference_id();
        let reference = self.ctx.scoping().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else {
            return;
        };
        self.symbols.push(ConditionSymbolInfo {
            symbol_id,
            reference_node_id: reference.node_id(),
            group_span: self.group_stack.first().copied(),
        });
    }
}

struct ConditionSymbolInfo {
    symbol_id: SymbolId,
    reference_node_id: NodeId,
    group_span: Option<Span>,
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var foo = 0; while (foo) { ++foo; }",
        "let foo = 0; while (foo) { ++foo; }", // { "ecmaVersion": 6 },
        "var foo = 0; while (foo) { foo += 1; }",
        "var foo = 0; while (foo++) { }",
        "var foo = 0; while (foo = next()) { }",
        "var foo = 0; while (ok(foo)) { }",
        "var foo = 0, bar = 0; while (++foo < bar) { }",
        "var foo = 0, obj = {}; while (foo === obj.bar) { }",
        "var foo = 0, f = {}, bar = {}; while (foo === f(bar)) { }",
        "var foo = 0, f = {}; while (foo === f()) { }",
        "var foo = 0, tag = 0; while (foo === tag`abc`) { }", // { "ecmaVersion": 6 },
        "function* foo() { var foo = 0; while (yield foo) { } }", // { "ecmaVersion": 6 },
        "function* foo() { var foo = 0; while (foo === (yield)) { } }", // { "ecmaVersion": 6 },
        "var foo = 0; while (foo.ok) { }",
        "var foo = 0; while (foo) { update(); } function update() { ++foo; }",
        "var foo = 0, bar = 9; while (foo < bar) { foo += 1; }",
        "var foo = 0, bar = 1, baz = 2; while (foo ? bar : baz) { foo += 1; }",
        "var foo = 0, bar = 0; while (foo && bar) { ++foo; ++bar; }",
        "var foo = 0, bar = 0; while (foo || bar) { ++foo; ++bar; }",
        "var foo = 0; do { ++foo; } while (foo);",
        "var foo = 0; do { } while (foo++);",
        "for (var foo = 0; foo; ++foo) { }",
        "for (var foo = 0; foo;) { ++foo }",
        "var foo = 0, bar = 0; for (bar; foo;) { ++foo }",
        "var foo; if (foo) { }",
        "var a = [1, 2, 3]; var len = a.length; for (var i = 0; i < len - 1; i++) {}",
    ];

    let fail = vec![
        "var foo = 0; while (foo) { } foo = 1;",
        "var foo = 0; while (!foo) { } foo = 1;",
        "var foo = 0; while (foo != null) { } foo = 1;",
        "var foo = 0, bar = 9; while (foo < bar) { } foo = 1;",
        "var foo = 0, bar = 0; while (foo && bar) { ++bar; } foo = 1;",
        "var foo = 0, bar = 0; while (foo && bar) { ++foo; } foo = 1;",
        "var a, b, c; while (a < c && b < c) { ++a; } foo = 1;",
        "var foo = 0; while (foo ? 1 : 0) { } foo = 1;",
        "var foo = 0; while (foo) { update(); } function update(foo) { ++foo; }",
        "var foo = 0; while (foo < 10) { function update() { foo++; } }",
        "var foo = 0; while (foo < 10) { const fn = update; } function update() { foo++; }",
        "var foo; do { } while (foo);",
        "for (var foo = 0; foo < 10; ) { } foo = 1;",
    ];

    Tester::new(NoUnmodifiedLoopCondition::NAME, NoUnmodifiedLoopCondition::PLUGIN, pass, fail)
        .test_and_snapshot();
}
