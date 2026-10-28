use oxc_ast::{
    AstKind,
    ast::{AssignmentExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, ReferenceId};
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ast_util::iter_outer_expressions,
    config::GlobalValue,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
    utils::{is_global_exports_assignment_target, is_global_module_exports},
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
    version = "1.76.0",
    short_description = "Enforce either `module.exports` or `exports`.",
);

impl Rule for ExportsStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        match self.0 {
            ExportsStyleMode::ModuleExports => self.check_exports_references(ctx),
            ExportsStyleMode::Exports => self.check_module_exports_references(ctx),
        }
    }
}

impl ExportsStyle {
    /// Report every reference to the global `exports` (the `"module.exports"` mode).
    fn check_exports_references(&self, ctx: &LintContext) {
        for &reference_id in global_reference_ids("exports", ctx) {
            let node_id = ctx.scoping().get_reference(reference_id).node_id();

            if self.1.allow_batch_assign
                && top_assignment_id(node_id, ctx)
                    .is_some_and(|top| assignment_chain_has_module_exports(top, ctx))
            {
                continue;
            }

            ctx.diagnostic(unexpected_exports_diagnostic(ctx.nodes().kind(node_id).span()));
        }
    }

    /// Report every `module.exports` access and every assignment to the global `exports`
    /// (the `"exports"` mode).
    fn check_module_exports_references(&self, ctx: &LintContext) {
        for &reference_id in global_reference_ids("module", ctx) {
            let node_id = ctx.scoping().get_reference(reference_id).node_id();
            // `module` may be wrapped, as in `(module as any).exports`, so climb to the first
            // non-wrapper ancestor and unwrap the member expression's object to match it.
            let Some(parent) = iter_outer_expressions(ctx.nodes(), node_id).next() else {
                continue;
            };
            let Some(member_expr) = parent.as_member_expression_kind() else {
                continue;
            };
            // Skip `module` in property position (`foo[module]`) and any property but `exports`.
            if member_expr.object().get_inner_expression().node_id() != node_id
                || !member_expr.static_property_name().is_some_and(|name| name == "exports")
            {
                continue;
            }

            if self.1.allow_batch_assign
                && top_assignment_id(parent.node_id(), ctx)
                    .is_some_and(|top| assignment_chain_has_exports(top, ctx))
            {
                continue;
            }

            ctx.diagnostic(unexpected_module_exports_diagnostic(member_expr.span()));
        }

        for &reference_id in global_reference_ids("exports", ctx) {
            let node_id = ctx.scoping().get_reference(reference_id).node_id();
            let assign_id = ctx.nodes().parent_id(node_id);
            let AstKind::AssignmentExpression(assign_expr) = ctx.nodes().kind(assign_id) else {
                continue;
            };
            // The reference must be the assignment target itself, not part of the
            // right-hand side of the same assignment.
            if assign_expr.left.node_id() != node_id {
                continue;
            }

            if self.1.allow_batch_assign
                && assignment_chain_has_module_exports(outermost_assignment_id(assign_id, ctx), ctx)
            {
                continue;
            }

            ctx.diagnostic(unexpected_assignment_diagnostic(assign_expr.left.span()));
        }
    }
}

/// References to the global binding `name`, empty if the file has none or the user turned the
/// global off with `globals: { <name>: "off" }`.
///
/// References in this list are global by construction, so callers don't need to re-check that
/// with [`LintContext::is_reference_to_global_variable`].
fn global_reference_ids<'c>(name: &str, ctx: &'c LintContext) -> &'c [ReferenceId] {
    // Check the reference list first: most files have no CommonJS references at all, and the
    // `globals` lookup is only worth paying for when there is something to report.
    let Some(references) = ctx.scoping().root_unresolved_references().get(name) else {
        return &[];
    };
    if ctx.globals().get(name).is_some_and(|value| *value == GlobalValue::Off) {
        return &[];
    }
    references
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

/// Whether any assignment in a `a = b = c` chain, starting at `top_assignment_id`, matches
/// `predicate`.
fn assignment_chain_has(
    top_assignment_id: NodeId,
    ctx: &LintContext,
    predicate: impl Fn(&AssignmentExpression) -> bool,
) -> bool {
    let AstKind::AssignmentExpression(mut assign_expr) = ctx.nodes().kind(top_assignment_id) else {
        return false;
    };
    loop {
        if predicate(assign_expr) {
            return true;
        }

        let Expression::AssignmentExpression(right_assign_expr) = &assign_expr.right else {
            return false;
        };
        assign_expr = right_assign_expr;
    }
}

/// The outermost assignment expression that `node_id` is assigned by, if any.
///
/// Member accesses are climbed first, so an `exports` reference resolves through
/// `exports.foo.bar = 1`, and nested assignments after that, so it resolves to the whole of
/// `exports = module.exports = {}`.
fn top_assignment_id(mut node_id: NodeId, ctx: &LintContext) -> Option<NodeId> {
    while let Some(member_expr) = ctx.nodes().parent_kind(node_id).as_member_expression_kind() {
        // Stop if the node is the property rather than the object, as in `foo[exports]`.
        if member_expr.object().node_id() != node_id {
            break;
        }
        node_id = ctx.nodes().parent_id(node_id);
    }

    let assign_id = ctx.nodes().parent_id(node_id);
    let AstKind::AssignmentExpression(assign_expr) = ctx.nodes().kind(assign_id) else {
        return None;
    };
    if assign_expr.left.node_id() != node_id {
        return None;
    }

    Some(outermost_assignment_id(assign_id, ctx))
}

/// Climb from an assignment expression to the outermost one enclosing it, as in
/// `a = b = c` from `b = c`.
fn outermost_assignment_id(mut node_id: NodeId, ctx: &LintContext) -> NodeId {
    while matches!(ctx.nodes().parent_kind(node_id), AstKind::AssignmentExpression(_)) {
        node_id = ctx.nodes().parent_id(node_id);
    }
    node_id
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
        // `exports` is read, not assigned to
        ("foo = exports", Some(serde_json::json!(["exports"])), None),
        // `module` is the property, not the object
        ("foo[module] = 1", Some(serde_json::json!(["exports"])), None),
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
        ("(module).exports = {foo: 1}", Some(serde_json::json!(["exports"])), None),
        ("(module as any).exports = {foo: 1}", Some(serde_json::json!(["exports"])), None),
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
