use oxc_ast::ast::{CallExpression, ChainElement, Expression};
use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::cmp::ContentEq;
use oxc_span::{CompactStr, GetSpan};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoExtendNative {
    /// A list of objects which are allowed to be exceptions to the rule.
    exceptions: Vec<CompactStr>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoExtendNative,
    suspicious,
);

impl Rule for NoExtendNative {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self {
            exceptions: obj
                .and_then(|v| v.get("exceptions"))
                .and_then(serde_json::Value::as_array)
                .unwrap_or(&vec![])
                .iter()
                .map(serde_json::Value::as_str)
                .filter(Option::is_some)
                .map(|x| x.unwrap().into())
                .collect::<Vec<CompactStr>>(),
        }
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
                // If the first letter is capital, like `Object`, we will assume it is a native obejct
                let Some(first_char) = name.chars().next() else {
                    continue;
                };
                if first_char.is_lowercase() {
                    continue;
                }
                let node = ctx.nodes().get_node(reference.node_id());
                dbg!(name);
                dbg!(node);
                // If this is not `*.prototype` access, skip it.
                let Some(prop_access) = get_prototype_property_accessed(ctx, node) else {
                    continue;
                };
                dbg!(prop_access);
                // Check if being used like `String.prototype.xyz = 0`
                if let Some(prop_assign) = get_property_assignment(ctx, prop_access) {
                    dbg!(prop_assign);
                    ctx.diagnostic(
                        OxcDiagnostic::error(format!(
                            "{} prototype is read-only, properties should not be added.",
                            name
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
                            "{} prototype is read-only, properties should not be added.",
                            name
                        ))
                        .with_label(define_property_call.span()),
                    );
                }
            }
        }
    }
}

fn get_define_property_call<'a>(
    ctx: &'a LintContext,
    node: &AstNode<'a>,
) -> Option<&'a AstNode<'a>> {
    dbg!("get_define_property_call1", node);
    let Some(mut ancestor) = ctx.nodes().parent_node(node.id()) else {
        return None;
    };
    loop {
        dbg!(ancestor);
        if let AstKind::CallExpression(call_expr) = ancestor.kind() {
            if !is_define_property_call(call_expr) {
                return None;
            }
            return Some(ancestor);
        } else if let AstKind::ChainExpression(_) | AstKind::Argument(_) = ancestor.kind() {
            ancestor = ctx.nodes().parent_node(ancestor.id())?;
        } else {
            return None;
        }
    }
}

fn is_define_property_call(call_expr: &CallExpression) -> bool {
    dbg!(call_expr.callee.without_parentheses());
    let callee = call_expr.callee.without_parentheses();

    let member_expression = if let Expression::ChainExpression(chain_expr) = callee {
        dbg!(chain_expr.expression.as_member_expression())
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
    dbg!(node);
    let mut parent = ctx.nodes().parent_node(node.id())?;
    loop {
        dbg!(parent);
        if let AstKind::AssignmentExpression(_) = parent.kind() {
            return Some(parent);
        } else if let AstKind::AssignmentTarget(_) | AstKind::SimpleAssignmentTarget(_) =
            parent.kind()
        {
            parent = ctx.nodes().parent_node(parent.id())?;
        } else if let AstKind::MemberExpression(member_expr) = parent.kind() {
            if let MemberExpression::ComputedMemberExpression(computed) = member_expr {
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
            parent = ctx.nodes().parent_node(parent.id())?;
        } else {
            return None;
        }
    }
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
    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return None;
    };
    dbg!(parent);
    let mut prototype_node = Some(parent);
    let AstKind::MemberExpression(prop_access_expr) = parent.kind() else {
        return None;
    };
    let Some(prop_name) = prop_access_expr.static_property_name() else {
        return None;
    };
    if prop_name != "prototype" {
        return None;
    }
    let Some(grandparent_node) = ctx.nodes().parent_node(parent.id()) else {
        return None;
    };
    dbg!(grandparent_node);
    if let AstKind::ChainExpression(chain_expr) = grandparent_node.kind() {
        prototype_node = Some(grandparent_node);
        if let Some(grandparent_parent) = ctx.nodes().parent_node(grandparent_node.id()) {
            dbg!(grandparent_parent);
            prototype_node = Some(grandparent_parent);
        };
        if let ChainElement::ComputedMemberExpression(computed) = &chain_expr.expression {
            if computed
                .object
                .as_member_expression()
                .is_some_and(|object| object.content_eq(prop_access_expr))
            {
                prototype_node = Some(grandparent_node);
            }
        }
    }
    // Expand the search to include computed member expressions like `Object['prototype']`
    else if let AstKind::MemberExpression(expr) = grandparent_node.kind() {
        if let MemberExpression::ComputedMemberExpression(computed) = expr {
            if computed
                .object
                .as_member_expression()
                .is_some_and(|object| object.content_eq(prop_access_expr))
            {
                prototype_node = Some(grandparent_node);
            }
        }
    }
    prototype_node
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

    Tester::new(NoExtendNative::NAME, pass, fail).test_and_snapshot();
}
