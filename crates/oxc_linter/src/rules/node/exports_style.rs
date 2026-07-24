use oxc_ast::{
    AstKind, MemberExpressionKind,
    ast::{AssignmentExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
    utils::{
        is_global_exports_assignment_target, is_global_exports_reference, is_global_module_exports,
        is_global_module_reference,
    },
};

fn unexpected_exports_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected access to `exports`.")
        .with_help("Use `module.exports` instead.")
        .with_label(span)
}

fn unexpected_module_exports_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected access to `module.exports`.")
        .with_help("Use `exports` instead.")
        .with_label(span)
}

fn unexpected_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected assignment to `exports`.")
        .with_help("Do not modify `exports` itself.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum ExportsStyleMode {
    #[default]
    #[serde(rename = "module.exports")]
    /// Requires `module.exports` and disallows `exports`
    ModuleExports,
    /// Requires `exports` and disallows `module.exports`
    Exports,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct ExportsStyleOptions {
    /// If this option is set to `true`, `module.exports = exports = obj` are allowed.
    allow_batch_assign: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct ExportsStyle(ExportsStyleMode, ExportsStyleOptions);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce either `module.exports` or `exports`.
    ///
    /// ### Why is this bad?
    ///
    /// `module.exports` and `exports` are the same instance by default. But those come to be different if one of them is modified.
    ///  ```js
    /// module.exports = {
    ///     foo: 1
    /// }
    ///
    /// exports.bar = 2
    /// ```
    /// In this case, `exports.bar` will be lost since only the instance of `module.exports` will be exported.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for the `"module.exports"` option:
    /// ```js
    /// exports.foo = 1
    /// exports.bar = 2
    /// ```
    ///
    /// Examples of **correct** code for the `"module.exports"` option:
    /// ```js
    /// module.exports = {
    ///     foo: 1,
    ///     bar: 2
    /// }
    /// module.exports.baz = 3
    /// ```
    ///
    /// Examples of **incorrect** code for the `"exports"` option:
    /// ```js
    /// module.exports = {
    ///     foo: 1,
    ///     bar: 2
    /// }
    /// module.exports.baz = 3
    /// ```
    ///
    /// Examples of **correct** code for the `"exports"` option:
    /// ```js
    /// exports.foo = 1
    /// exports.bar = 2
    /// ```
    ExportsStyle,
    node,
    style,
    pending,
    config = ExportsStyle,
    version = "next",
    short_description = "Enforce either `module.exports` or `exports`.",
);

impl Rule for ExportsStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        // Everything this rule flags hangs off a reference to the global `exports` or
        // `module` binding, so iterate those (usually empty) reference lists instead of
        // dispatching on every identifier, member expression, and assignment.
        match self.0 {
            ExportsStyleMode::ModuleExports => self.check_exports_references(ctx),
            ExportsStyleMode::Exports => self.check_module_exports_references(ctx),
        }
    }
}

impl ExportsStyle {
    /// Report every reference to the global `exports` (the `"module.exports"` mode).
    fn check_exports_references(&self, ctx: &LintContext) {
        let Some(references) = ctx.scoping().root_unresolved_references().get("exports") else {
            return;
        };
        for &reference_id in references {
            let reference = ctx.scoping().get_reference(reference_id);
            let node = ctx.nodes().get_node(reference.node_id());
            let AstKind::IdentifierReference(ident) = node.kind() else {
                continue;
            };
            if !is_global_exports_reference(ident, ctx) {
                continue;
            }

            if self.1.allow_batch_assign
                && top_assignment_id(node.id(), ctx)
                    .is_some_and(|top| assignment_chain_has_module_exports(top, ctx))
            {
                continue;
            }

            ctx.diagnostic(unexpected_exports_diagnostic(ident.span));
        }
    }

    /// Report every `module.exports` access and every assignment to the global `exports`
    /// (the `"exports"` mode). Diagnostics are collected and emitted in source order,
    /// matching the previous single-pass emission order.
    fn check_module_exports_references(&self, ctx: &LintContext) {
        let mut diagnostics: Vec<(u32, OxcDiagnostic)> = Vec::new();

        if let Some(references) = ctx.scoping().root_unresolved_references().get("module") {
            for &reference_id in references {
                let reference = ctx.scoping().get_reference(reference_id);
                let node = ctx.nodes().get_node(reference.node_id());
                let AstKind::IdentifierReference(_) = node.kind() else {
                    continue;
                };
                let member_id = outermost_wrapped_expression_id(node.id(), ctx);
                let member_expr = match ctx.nodes().parent_kind(member_id) {
                    AstKind::StaticMemberExpression(member_expr) => {
                        MemberExpressionKind::Static(member_expr)
                    }
                    AstKind::ComputedMemberExpression(member_expr) => {
                        MemberExpressionKind::Computed(member_expr)
                    }
                    _ => continue,
                };
                if !is_global_module_exports_kind(&member_expr, ctx) {
                    continue;
                }

                if self.1.allow_batch_assign
                    && top_assignment_id(ctx.nodes().parent_id(member_id), ctx)
                        .is_some_and(|top| assignment_chain_has_exports(top, ctx))
                {
                    continue;
                }

                let span = member_expr.span();
                diagnostics.push((span.start, unexpected_module_exports_diagnostic(span)));
            }
        }

        if let Some(references) = ctx.scoping().root_unresolved_references().get("exports") {
            for &reference_id in references {
                let reference = ctx.scoping().get_reference(reference_id);
                let node = ctx.nodes().get_node(reference.node_id());
                let AstKind::IdentifierReference(ident) = node.kind() else {
                    continue;
                };
                let AstKind::AssignmentExpression(assign_expr) = ctx.nodes().parent_kind(node.id())
                else {
                    continue;
                };
                // The reference must be the assignment target itself, not part of the
                // right-hand side of the same assignment.
                if assign_expr.left.span() != ident.span {
                    continue;
                }
                if !is_global_exports_assignment_target(&assign_expr.left, ctx) {
                    continue;
                }

                if self.1.allow_batch_assign
                    && top_assignment_from_assignment_id(ctx.nodes().parent_id(node.id()), ctx)
                        .is_some_and(|top| assignment_chain_has_module_exports(top, ctx))
                {
                    continue;
                }

                let span = assign_expr.left.span();
                diagnostics.push((span.start, unexpected_assignment_diagnostic(span)));
            }
        }

        diagnostics.sort_unstable_by_key(|(start, _)| *start);
        for (_, diagnostic) in diagnostics {
            ctx.diagnostic(diagnostic);
        }
    }
}

/// Walk up past the wrapper nodes that [`Expression::get_inner_expression`] unwraps
/// (parentheses and TS assertion/satisfies/instantiation/non-null expressions), so that
/// e.g. `(module).exports` and `(module as any).exports` are still detected.
fn outermost_wrapped_expression_id(mut node_id: NodeId, ctx: &LintContext) -> NodeId {
    while matches!(
        ctx.nodes().parent_kind(node_id),
        AstKind::ParenthesizedExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSInstantiationExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
    ) {
        node_id = ctx.nodes().parent_id(node_id);
    }
    node_id
}

fn is_global_module_exports_kind(member_expr: &MemberExpressionKind, ctx: &LintContext) -> bool {
    member_expr.static_property_name().is_some_and(|name| name == "exports")
        && member_expr
            .object()
            .get_identifier_reference()
            .is_some_and(|ident| is_global_module_reference(ident, ctx))
}

fn assignment_chain_has_exports(top_assignment_id: NodeId, ctx: &LintContext) -> bool {
    assignment_chain_has(top_assignment_id, ctx, |assign_expr| {
        is_global_exports_assignment_target(&assign_expr.left, ctx)
    })
}

fn assignment_chain_has_module_exports(top_assignment_id: NodeId, ctx: &LintContext) -> bool {
    assignment_chain_has(top_assignment_id, ctx, |assign_expr| {
        assign_expr
            .left
            .as_member_expression()
            .is_some_and(|member| is_global_module_exports(member, ctx))
    })
}

fn assignment_chain_has(
    mut node_id: NodeId,
    ctx: &LintContext,
    predicate: impl Fn(&AssignmentExpression) -> bool,
) -> bool {
    loop {
        let AstKind::AssignmentExpression(assign_expr) = ctx.nodes().kind(node_id) else {
            return false;
        };
        if predicate(assign_expr) {
            return true;
        }

        let Expression::AssignmentExpression(right_assign_expr) = &assign_expr.right else {
            return false;
        };
        node_id = right_assign_expr.node_id.get();
    }
}

fn top_assignment_id(mut node_id: NodeId, ctx: &LintContext) -> Option<NodeId> {
    while let parent @ (AstKind::StaticMemberExpression(_) | AstKind::ComputedMemberExpression(_)) =
        ctx.nodes().parent_kind(node_id)
    {
        let object_span = match parent {
            AstKind::StaticMemberExpression(member_expr) => member_expr.object.span(),
            AstKind::ComputedMemberExpression(member_expr) => member_expr.object.span(),
            _ => return None,
        };
        if object_span != ctx.nodes().kind(node_id).span() {
            break;
        }
        node_id = ctx.nodes().parent_id(node_id);
    }

    let AstKind::AssignmentExpression(assign_expr) = ctx.nodes().parent_kind(node_id) else {
        return None;
    };
    if assign_expr.left.span() != ctx.nodes().kind(node_id).span() {
        return None;
    }

    node_id = ctx.nodes().parent_id(node_id);
    while matches!(ctx.nodes().parent_kind(node_id), AstKind::AssignmentExpression(_)) {
        node_id = ctx.nodes().parent_id(node_id);
    }

    Some(node_id)
}

fn top_assignment_from_assignment_id(mut node_id: NodeId, ctx: &LintContext) -> Option<NodeId> {
    if !matches!(ctx.nodes().kind(node_id), AstKind::AssignmentExpression(_)) {
        return None;
    }

    while matches!(ctx.nodes().parent_kind(node_id), AstKind::AssignmentExpression(_)) {
        node_id = ctx.nodes().parent_id(node_id);
    }

    Some(node_id)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("module.exports = {foo: 1}", None, None),
        ("module.exports = {foo: 1}", Some(serde_json::json!(["module.exports"])), None),
        ("exports.foo = 1", Some(serde_json::json!(["exports"])), None),
        (
            "exports = module.exports = {foo: 1}",
            Some(serde_json::json!(["module.exports", { "allowBatchAssign": true }])),
            None,
        ),
        (
            "module.exports = exports = {foo: 1}",
            Some(serde_json::json!(["module.exports", { "allowBatchAssign": true }])),
            None,
        ),
        (
            "exports = module.exports = {foo: 1}",
            Some(serde_json::json!(["exports", { "allowBatchAssign": true }])),
            None,
        ),
        (
            "module.exports = exports = {foo: 1}",
            Some(serde_json::json!(["exports", { "allowBatchAssign": true }])),
            None,
        ),
        (
            "exports = module.exports = {foo: 1}; exports.bar = 2",
            Some(serde_json::json!(["exports", { "allowBatchAssign": true }])),
            None,
        ),
        (
            "module.exports = exports = {foo: 1}; exports.bar = 2",
            Some(serde_json::json!(["exports", { "allowBatchAssign": true }])),
            None,
        ),
        ("module = {}; module.foo = 1", Some(serde_json::json!(["exports"])), None),
        (
            "exports.foo = 1",
            Some(serde_json::json!(["module.exports"])),
            Some(serde_json::json!({ "globals": { "exports": "off" } })),
        ),
        (
            "module.exports = {foo: 1}",
            Some(serde_json::json!(["exports"])),
            Some(serde_json::json!({ "globals": { "module": "off" } })),
        ),
    ];

    let fail = vec![
        ("exports = {foo: 1}", None, None),
        ("exports.foo = 1", None, None),
        ("module.exports = exports = {foo: 1}", None, None),
        ("exports = module.exports = {foo: 1}", None, None),
        ("exports = {foo: 1}", Some(serde_json::json!(["module.exports"])), None),
        ("exports.foo = 1", Some(serde_json::json!(["module.exports"])), None),
        ("module.exports = exports = {foo: 1}", Some(serde_json::json!(["module.exports"])), None),
        ("exports = module.exports = {foo: 1}", Some(serde_json::json!(["module.exports"])), None),
        ("exports = {foo: 1}", Some(serde_json::json!(["exports"])), None),
        ("module.exports = {foo: 1}", Some(serde_json::json!(["exports"])), None),
        ("module.exports.foo = 1", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { a: 1 }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { a: 1, b: 2 }", Some(serde_json::json!(["exports"])), None),
        (
            "module.exports = { // before a
            a: 1, // between a and b
            b: 2 // after b
            }",
            Some(serde_json::json!(["exports"])),
            None,
        ),
        ("foo(module.exports = {foo: 1})", Some(serde_json::json!(["exports"])), None),
        (
            "if(foo){ module.exports = { foo: 1};} else { module.exports = {foo: 2};}",
            Some(serde_json::json!(["exports"])),
            None,
        ),
        (
            "function bar() { module.exports = { foo: 1 }; }",
            Some(serde_json::json!(["exports"])),
            None,
        ),
        ("module.exports = { get a() {} }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { set a(a) {} }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { a }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { ...a }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { ['a' + 'b']: 1 }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { 'foo': 1 }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { foo(a) {} }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { *foo(a) {} }", Some(serde_json::json!(["exports"])), None),
        ("module.exports = { async foo(a) {} }", Some(serde_json::json!(["exports"])), None),
        ("module.exports.foo()", Some(serde_json::json!(["exports"])), None),
        (
            "a = module.exports.foo + module.exports['bar']",
            Some(serde_json::json!(["exports"])),
            None,
        ),
        ("module.exports = exports = {foo: 1}", Some(serde_json::json!(["exports"])), None),
        ("exports = module.exports = {foo: 1}", Some(serde_json::json!(["exports"])), None),
        (
            "module.exports = exports = {foo: 1}; exports = obj",
            Some(serde_json::json!(["exports", { "allowBatchAssign": true }])),
            None,
        ),
        (
            "exports = module.exports = {foo: 1}; exports = obj",
            Some(serde_json::json!(["exports", { "allowBatchAssign": true }])),
            None,
        ),
    ];

    let _fix = vec![
        ("module.exports = {foo: 1}", "exports.foo = 1;", Some(serde_json::json!(["exports"]))),
        ("module.exports.foo = 1", "exports.foo = 1", Some(serde_json::json!(["exports"]))),
        ("module.exports = { a: 1 }", "exports.a = 1;", Some(serde_json::json!(["exports"]))),
        (
            "module.exports = { a: 1, b: 2 }",
            "exports.a = 1;
            
            exports.b = 2;",
            Some(serde_json::json!(["exports"])),
        ),
        (
            "module.exports = { // before a
            a: 1, // between a and b
            b: 2 // after b
            }",
            "// before a
            exports.a = 1;
            
            // between a and b
            exports.b = 2;
            // after b",
            Some(serde_json::json!(["exports"])),
        ),
        ("module.exports = { a }", "exports.a = a;", Some(serde_json::json!(["exports"]))),
        (
            "module.exports = { ['a' + 'b']: 1 }",
            "exports['a' + 'b'] = 1;",
            Some(serde_json::json!(["exports"])),
        ),
        (
            "module.exports = { 'foo': 1 }",
            "exports['foo'] = 1;",
            Some(serde_json::json!(["exports"])),
        ),
        (
            "module.exports = { foo(a) {} }",
            "exports.foo = function (a) {};",
            Some(serde_json::json!(["exports"])),
        ),
        (
            "module.exports = { *foo(a) {} }",
            "exports.foo = function* (a) {};",
            Some(serde_json::json!(["exports"])),
        ),
        (
            "module.exports = { async foo(a) {} }",
            "exports.foo = async function (a) {};",
            Some(serde_json::json!(["exports"])),
        ),
        ("module.exports.foo()", "exports.foo()", Some(serde_json::json!(["exports"]))),
        (
            "a = module.exports.foo + module.exports['bar']",
            "a = exports.foo + exports['bar']",
            Some(serde_json::json!(["exports"])),
        ),
    ];

    Tester::new(ExportsStyle::NAME, ExportsStyle::PLUGIN, pass, fail).test_and_snapshot();
}
