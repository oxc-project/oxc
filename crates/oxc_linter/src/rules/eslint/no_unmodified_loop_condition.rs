use rustc_hash::FxHashMap;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, Reference, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

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
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut grouped_conditions: FxHashMap<NodeId, Vec<LoopConditionInfo>> =
            FxHashMap::default();
        let scoping = ctx.scoping();

        for symbol_id in scoping.symbol_ids() {
            Self::check_symbol(symbol_id, ctx, &mut grouped_conditions);
        }

        for conditions in grouped_conditions.values() {
            if conditions.iter().all(|condition| !condition.modified) {
                for condition in conditions {
                    Self::report_condition(condition, ctx);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct LoopConditionInfo {
    reference_node_id: NodeId,
    group_node_id: Option<NodeId>,
    loop_span: Span,
    loop_kind: LoopKind,
    modified: bool,
}

#[derive(Debug, Clone, Copy)]
enum LoopKind {
    While,
    DoWhile,
    For { init_span: Option<Span> },
}

impl LoopConditionInfo {
    fn is_in_loop(&self, reference: &Reference, ctx: &LintContext<'_>) -> bool {
        let reference_span = ctx.semantic().reference_span(reference);
        if !self.loop_span.contains_inclusive(reference_span) {
            return false;
        }
        match self.loop_kind {
            LoopKind::For { init_span: Some(init_span) } => {
                !init_span.contains_inclusive(reference_span)
            }
            _ => true,
        }
    }
}

impl NoUnmodifiedLoopCondition {
    fn check_symbol(
        symbol_id: SymbolId,
        ctx: &LintContext<'_>,
        grouped_conditions: &mut FxHashMap<NodeId, Vec<LoopConditionInfo>>,
    ) {
        let scoping = ctx.scoping();
        let mut conditions = vec![];
        let mut modifiers = vec![];

        for reference in scoping.get_resolved_references(symbol_id) {
            if let Some(condition) = Self::to_loop_condition(reference, ctx) {
                conditions.push(condition);
            }
            if reference.is_write() {
                modifiers.push(reference);
            }
        }

        if conditions.is_empty() {
            return;
        }

        if !modifiers.is_empty() {
            Self::update_modified_flags(&mut conditions, &modifiers, ctx);
        }

        for condition in conditions {
            if let Some(group_node_id) = condition.group_node_id {
                grouped_conditions.entry(group_node_id).or_default().push(condition);
            } else if !condition.modified {
                Self::report_condition(&condition, ctx);
            }
        }
    }

    fn report_condition(condition: &LoopConditionInfo, ctx: &LintContext<'_>) {
        let node = ctx.nodes().get_node(condition.reference_node_id);
        let AstKind::IdentifierReference(ident) = node.kind() else {
            return;
        };
        ctx.diagnostic(no_unmodified_loop_condition_diagnostic(ident.name.as_str(), ident.span));
    }

    fn update_modified_flags(
        conditions: &mut [LoopConditionInfo],
        modifiers: &[&Reference],
        ctx: &LintContext<'_>,
    ) {
        for condition in conditions.iter_mut() {
            for modifier in modifiers {
                if condition.modified {
                    break;
                }

                let in_loop = condition.is_in_loop(modifier, ctx)
                    || Self::is_modified_via_called_function_declaration(condition, modifier, ctx);
                condition.modified = in_loop;
            }
        }
    }

    fn is_modified_via_called_function_declaration(
        condition: &LoopConditionInfo,
        modifier: &Reference,
        ctx: &LintContext<'_>,
    ) -> bool {
        let Some(function_symbol_id) =
            Self::get_enclosing_function_declaration_symbol_id(modifier.node_id(), ctx)
        else {
            return false;
        };

        for function_reference in ctx.scoping().get_resolved_references(function_symbol_id) {
            if condition.is_in_loop(function_reference, ctx) {
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

    fn to_loop_condition(
        reference: &Reference,
        ctx: &LintContext<'_>,
    ) -> Option<LoopConditionInfo> {
        let nodes = ctx.nodes();
        let reference_node = nodes.get_node(reference.node_id());
        if !matches!(reference_node.kind(), AstKind::IdentifierReference(_)) {
            return None;
        }

        let mut group_node_id = None;
        let mut child_id = reference.node_id();
        let mut current_id = nodes.parent_id(child_id);

        while current_id != NodeId::ROOT {
            let current_node = nodes.get_node(current_id);
            let child_span = nodes.get_node(child_id).span();

            if Self::is_sentinel(current_node) {
                if let Some((loop_span, loop_kind)) =
                    Self::loop_condition_from_sentinel(current_node, child_span)
                {
                    return Some(LoopConditionInfo {
                        reference_node_id: reference.node_id(),
                        group_node_id,
                        loop_span,
                        loop_kind,
                        modified: false,
                    });
                }
                break;
            }

            if matches!(
                current_node.kind(),
                AstKind::BinaryExpression(_) | AstKind::ConditionalExpression(_)
            ) {
                if Self::has_dynamic_expressions(current_id, ctx) {
                    break;
                }
                group_node_id = Some(current_id);
            }

            child_id = current_id;
            current_id = nodes.parent_id(current_id);
        }

        None
    }

    fn loop_condition_from_sentinel(
        node: &AstNode<'_>,
        child_span: Span,
    ) -> Option<(Span, LoopKind)> {
        match node.kind() {
            AstKind::WhileStatement(statement) if statement.test.span() == child_span => {
                Some((statement.span, LoopKind::While))
            }
            AstKind::DoWhileStatement(statement) if statement.test.span() == child_span => {
                Some((statement.span, LoopKind::DoWhile))
            }
            AstKind::ForStatement(statement)
                if statement.test.as_ref().is_some_and(|test| test.span() == child_span) =>
            {
                Some((
                    statement.span,
                    LoopKind::For { init_span: statement.init.as_ref().map(GetSpan::span) },
                ))
            }
            _ => None,
        }
    }

    fn is_sentinel(node: &AstNode<'_>) -> bool {
        match node.kind() {
            AstKind::CallExpression(_)
            | AstKind::Class(_)
            | AstKind::Function(_)
            | AstKind::StaticMemberExpression(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::PrivateFieldExpression(_)
            | AstKind::NewExpression(_)
            | AstKind::YieldExpression(_) => true,
            kind if kind.is_statement() || kind.is_declaration() => true,
            _ => false,
        }
    }

    fn has_dynamic_expressions(group_node_id: NodeId, ctx: &LintContext<'_>) -> bool {
        for node in ctx.nodes() {
            if node.id() == group_node_id {
                continue;
            }

            let mut current_id = node.id();
            let mut inside_group = false;
            let mut skipped_by_boundary = false;

            while current_id != NodeId::ROOT {
                if current_id == group_node_id {
                    inside_group = true;
                    break;
                }

                let current_node = ctx.nodes().get_node(current_id);
                if current_id != node.id()
                    && matches!(
                        current_node.kind(),
                        AstKind::ArrowFunctionExpression(_)
                            | AstKind::Class(_)
                            | AstKind::Function(_)
                    )
                {
                    skipped_by_boundary = true;
                    break;
                }

                current_id = ctx.nodes().parent_id(current_id);
            }

            if !inside_group || skipped_by_boundary {
                continue;
            }

            if matches!(
                node.kind(),
                AstKind::CallExpression(_)
                    | AstKind::StaticMemberExpression(_)
                    | AstKind::ComputedMemberExpression(_)
                    | AstKind::PrivateFieldExpression(_)
                    | AstKind::NewExpression(_)
                    | AstKind::TaggedTemplateExpression(_)
                    | AstKind::YieldExpression(_)
            ) {
                return true;
            }
        }

        false
    }
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
        "var foo; do { } while (foo);",
        "for (var foo = 0; foo < 10; ) { } foo = 1;",
    ];

    Tester::new(NoUnmodifiedLoopCondition::NAME, NoUnmodifiedLoopCondition::PLUGIN, pass, fail)
        .test_and_snapshot();
}
