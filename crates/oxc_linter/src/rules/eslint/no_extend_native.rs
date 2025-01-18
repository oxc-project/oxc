use oxc_ast::{
    ast::{CallExpression, ChainElement, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{cmp::ContentEq, CompactStr, GetSpan};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoExtendNative(Box<NoExtendNativeConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoExtendNativeConfig {
    /// A list of objects which are allowed to be exceptions to the rule.
    exceptions: Vec<CompactStr>,
}

impl std::ops::Deref for NoExtendNative {
    type Target = NoExtendNativeConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents extending native global objects such as `Object`, `String`, or `Array` with new
    /// properties.
    ///
    /// ### Why is this bad?
    ///
    /// Extending native objects can cause unexpected behavior and conflicts with other code.
    ///
    /// For example:
    /// ```js
    /// // Adding a new property, which might seem okay
    /// Object.prototype.extra = 55;
    ///
    /// // Defining a user object
    /// const users = {
    ///     "1": "user1",
    ///     "2": "user2",
    /// };
    ///
    /// for (const id in users) {
    ///     // This will print "extra" as well as "1" and "2":
    ///     console.log(id);
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Object.prototype.p = 0
    /// Object.defineProperty(Array.prototype, 'p', {value: 0})
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// x.prototype.p = 0
    /// Object.defineProperty(x.prototype, 'p', {value: 0})
    /// ```
    NoExtendNative,
    eslint,
    suspicious,
);

impl Rule for NoExtendNative {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self(Box::new(NoExtendNativeConfig {
            exceptions: obj
                .and_then(|v| v.get("exceptions"))
                .and_then(serde_json::Value::as_array)
                .unwrap_or(&vec![])
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(CompactStr::from)
                .collect(),
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let symbols = ctx.symbols();
        for reference_id_list in ctx.scopes().root_unresolved_references_ids() {
            for reference_id in reference_id_list {
                let reference = symbols.get_reference(reference_id);
                let name = ctx.semantic().reference_name(reference);
                // If the referenced name does not appear to be a global object, skip it.
                if !ctx.env_contains_var(name) {
                    continue;
                }
                // If the referenced name is explicitly allowed, skip it.
                let compact_name = CompactStr::from(name);
                if self.exceptions.contains(&compact_name) {
                    continue;
                }
                // If the first letter is capital, like `Object`, we will assume it is a native object
                let Some(first_char) = name.chars().next() else {
                    continue;
                };
                if first_char.is_lowercase() {
                    continue;
                }
                let node = ctx.nodes().get_node(reference.node_id());
                // If this is not `*.prototype` access, skip it.
                let Some(prop_access) = get_prototype_property_accessed(ctx, node) else {
                    continue;
                };
                // Check if being used like `String.prototype.xyz = 0`
                if let Some(prop_assign) = get_property_assignment(ctx, prop_access) {
                    ctx.diagnostic(
                        OxcDiagnostic::error(format!(
                            "{name} prototype is read-only, properties should not be added."
                        ))
                        .with_label(prop_assign.span()),
                    );
                }
                // Check if being used like `Object.defineProperty(String.prototype, 'xyz', 0)`
                else if let Some(define_property_call) =
                    get_define_property_call(ctx, prop_access)
                {
                    ctx.diagnostic(
                        OxcDiagnostic::error(format!(
                            "{name} prototype is read-only, properties should not be added."
                        ))
                        .with_label(define_property_call.span()),
                    );
                }
            }
        }
    }
}

/// If this usage of `*.prototype` is a `Object.defineProperty` or `Object.defineProperties` call,
/// then this function returns the `CallExpression` node.
fn get_define_property_call<'a>(
    ctx: &'a LintContext,
    node: &AstNode<'a>,
) -> Option<&'a AstNode<'a>> {
    for parent in ctx.nodes().ancestors(node.id()).skip(1) {
        if let AstKind::CallExpression(call_expr) = parent.kind() {
            if is_define_property_call(call_expr) {
                return Some(parent);
            }
        }
    }
    None
}

/// Checks if a given `CallExpression` is a call to `Object.defineProperty` or `Object.defineProperties`.
fn is_define_property_call(call_expr: &CallExpression) -> bool {
    let callee = call_expr.callee.without_parentheses();

    let member_expression = if let Expression::ChainExpression(chain_expr) = callee {
        chain_expr.expression.as_member_expression()
    } else {
        callee.as_member_expression()
    };
    match member_expression {
        Some(me) => {
            let prop_name = me.static_property_name();
            me.object()
                .get_identifier_reference()
                .is_some_and(|ident_ref| ident_ref.name == "Object")
                && (prop_name == Some("defineProperty") || prop_name == Some("defineProperties"))
        }
        _ => false,
    }
}

/// Get an assignment to the property of the given node.
/// Example: `*.prop = 0` where `*.prop` is the given node.
fn get_property_assignment<'a>(
    ctx: &'a LintContext,
    node: &AstNode<'a>,
) -> Option<&'a AstNode<'a>> {
    for parent in ctx.nodes().ancestors(node.id()).skip(1) {
        match parent.kind() {
            AstKind::AssignmentExpression(_) => return Some(parent),
            AstKind::MemberExpression(MemberExpression::ComputedMemberExpression(computed)) => {
                if let AstKind::MemberExpression(node_expr) = node.kind() {
                    // Ignore computed member expressions like `obj[Object.prototype] = 0` (i.e., the
                    // given node is the `expression` of the computed member expression)
                    if computed
                        .expression
                        .as_member_expression()
                        .is_some_and(|expression| expression.content_eq(node_expr))
                    {
                        return None;
                    }
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}

/// Returns the ASTNode that represents a prototype property access, such as
/// `Object?.['prototype']`
fn get_prototype_property_accessed<'a>(
    ctx: &'a LintContext,
    node: &AstNode<'a>,
) -> Option<&'a AstNode<'a>> {
    let AstKind::IdentifierReference(_) = node.kind() else {
        return None;
    };
    let parent = ctx.nodes().parent_node(node.id())?;
    let mut prototype_node = Some(parent);
    let AstKind::MemberExpression(prop_access_expr) = parent.kind() else {
        return None;
    };
    let prop_name = prop_access_expr.static_property_name()?;
    if prop_name != "prototype" {
        return None;
    }
    let grandparent_node = ctx.nodes().parent_node(parent.id())?;

    if let AstKind::ChainExpression(_) = grandparent_node.kind() {
        prototype_node = Some(grandparent_node);
        if let Some(grandparent_parent) = ctx.nodes().parent_node(grandparent_node.id()) {
            prototype_node = Some(grandparent_parent);
        }
    }

    if is_computed_member_expression_matching(grandparent_node, prop_access_expr) {
        prototype_node = Some(grandparent_node);
    }

    prototype_node
}

fn is_computed_member_expression_matching(
    node: &AstNode,
    prop_access_expr: &MemberExpression,
) -> bool {
    match node.kind() {
        AstKind::ChainExpression(chain_expr) => {
            if let ChainElement::ComputedMemberExpression(computed) = &chain_expr.expression {
                return computed
                    .object
                    .as_member_expression()
                    .is_some_and(|object| object.content_eq(prop_access_expr));
            }
        }
        AstKind::MemberExpression(MemberExpression::ComputedMemberExpression(computed)) => {
            return computed
                .object
                .as_member_expression()
                .is_some_and(|object| object.content_eq(prop_access_expr));
        }
        _ => {}
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("x.prototype.p = 0", None),
        ("x.prototype['p'] = 0", None),
        ("Object.p = 0", None),
        ("Object.toString.bind = 0", None),
        ("Object['toString'].bind = 0", None),
        ("Object.defineProperty(x, 'p', {value: 0})", None),
        ("Object.defineProperty(x.prototype, 'p', {value: 0})", None),
        ("Object.defineProperties(x, {p: {value: 0}})", None),
        ("global.Object.prototype.toString = 0", None),
        ("this.Object.prototype.toString = 0", None),
        ("with(Object) { prototype.p = 0; }", None),
        ("o = Object; o.prototype.toString = 0", None),
        ("eval('Object.prototype.toString = 0')", None),
        ("parseFloat.prototype.x = 1", None),
        ("Object.prototype.g = 0", Some(serde_json::json!([{ "exceptions": ["Object"] }]))),
        ("obj[Object.prototype] = 0", None),
        ("Object.defineProperty()", None),
        ("Object.defineProperties()", None),
        ("function foo() { var Object = function() {}; Object.prototype.p = 0 }", None),
        ("{ let Object = function() {}; Object.prototype.p = 0 }", None), // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        ("Object.prototype.p = 0", None),
        ("BigInt.prototype.p = 0", None), // { "ecmaVersion": 2020 },
        ("WeakRef.prototype.p = 0", None), // { "ecmaVersion": 2021 },
        ("FinalizationRegistry.prototype.p = 0", None), // { "ecmaVersion": 2021 },
        ("AggregateError.prototype.p = 0", None), // { "ecmaVersion": 2021 },
        ("Function.prototype['p'] = 0", None),
        ("String['prototype'].p = 0", None),
        ("Number['prototype']['p'] = 0", None),
        ("Object.defineProperty(Array.prototype, 'p', {value: 0})", None),
        ("Object['defineProperty'](Array.prototype, 'p', {value: 0})", None),
        ("Object['defineProperty'](Array['prototype'], 'p', {value: 0})", None),
        ("Object.defineProperties(Array.prototype, {p: {value: 0}})", None),
        ("Object.defineProperties(Array.prototype, {p: {value: 0}, q: {value: 0}})", None),
        ("Number['prototype']['p'] = 0", Some(serde_json::json!([{ "exceptions": ["Object"] }]))),
        ("Object.prototype.p = 0; Object.prototype.q = 0", None),
        ("function foo() { Object.prototype.p = 0 }", None),
        ("(Object?.prototype).p = 0", None), // { "ecmaVersion": 2020 },
        ("(Object?.['prototype'])['p'] = 0", None),
        ("Object.defineProperty(Object?.prototype, 'p', { value: 0 })", None), // { "ecmaVersion": 2020 },
        ("Object?.defineProperty(Object.prototype, 'p', { value: 0 })", None), // { "ecmaVersion": 2020 },
        ("Object?.['defineProperty'](Object?.['prototype'], 'p', {value: 0})", None),
        ("(Object?.defineProperty)(Object.prototype, 'p', { value: 0 })", None), // { "ecmaVersion": 2020 },
        ("Array.prototype.p &&= 0", None), // { "ecmaVersion": 2021 },
        ("Array.prototype.p ||= 0", None), // { "ecmaVersion": 2021 },
        ("Array.prototype.p ??= 0", None), // { "ecmaVersion": 2021 }
    ];

    Tester::new(NoExtendNative::NAME, NoExtendNative::PLUGIN, pass, fail).test_and_snapshot();
}
