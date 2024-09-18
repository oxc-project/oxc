use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

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
                let Some(prop_assign) = get_property_assignment(ctx, prop_access) else {
                    continue;
                };
                dbg!(prop_assign);
                if ctx.env_contains_var(name) {
                    ctx.diagnostic(
                        OxcDiagnostic::error(format!(
                            "{} prototype is read only, properties should not be added.",
                            name
                        ))
                        .with_label(ctx.semantic().reference_span(reference)),
                    );
                }
            }
        }
    }
}

/// Get an assignment to the property of the given node.
/// Example: `*.prop = 0` where `*.prop` is the given node.
fn get_property_assignment<'a>(
    ctx: &'a LintContext,
    node: &AstNode<'a>,
) -> Option<&'a AstNode<'a>> {
    let mut parent = ctx.nodes().parent_node(node.id())?;
    loop {
        if let AstKind::AssignmentExpression(_) | AstKind::SimpleAssignmentTarget(_) = parent.kind()
        {
            return Some(parent);
        } else if let AstKind::MemberExpression(expr) = parent.kind() {
            // Ignore computed member expressions like `obj[Object.prototype] = 0`
            if let MemberExpression::ComputedMemberExpression(_) = expr {
                return None;
            }
            parent = ctx.nodes().parent_node(parent.id())?;
        } else {
            return None;
        }
    }
}

fn get_prototype_property_accessed<'a>(
    ctx: &'a LintContext,
    node: &AstNode<'a>,
) -> Option<&'a AstNode<'a>> {
    let AstKind::IdentifierReference(ident) = node.kind() else {
        return None;
    };
    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return None;
    };
    let mut prototype_node = Some(parent);
    let AstKind::MemberExpression(prop_access) = parent.kind() else {
        return None;
    };
    let MemberExpression::StaticMemberExpression(prop_access) = prop_access else {
        return None;
    };
    let prop_name = prop_access.property.name.as_str();
    if prop_name != "prototype" {
        return None;
    }
    let Some(grandparent) = ctx.nodes().parent_node(parent.id()) else {
        return None;
    };
    dbg!(grandparent);
    // Expand the search to include computed member expressions like `Object.prototype['p']`
    if let AstKind::MemberExpression(expr) = grandparent.kind() {
        if let MemberExpression::ComputedMemberExpression(computed) = expr {
            if computed.object == prop_access {
                prototype_node = Some(grandparent);
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
        ("Object.defineProperties(Array.prototype, {p: {value: 0}})", None),
        ("Object.defineProperties(Array.prototype, {p: {value: 0}, q: {value: 0}})", None),
        ("Number['prototype']['p'] = 0", Some(serde_json::json!([{ "exceptions": ["Object"] }]))),
        ("Object.prototype.p = 0; Object.prototype.q = 0", None),
        ("function foo() { Object.prototype.p = 0 }", None),
        ("(Object?.prototype).p = 0", None), // { "ecmaVersion": 2020 },
        ("Object.defineProperty(Object?.prototype, 'p', { value: 0 })", None), // { "ecmaVersion": 2020 },
        ("Object?.defineProperty(Object.prototype, 'p', { value: 0 })", None), // { "ecmaVersion": 2020 },
        ("(Object?.defineProperty)(Object.prototype, 'p', { value: 0 })", None), // { "ecmaVersion": 2020 },
        ("Array.prototype.p &&= 0", None), // { "ecmaVersion": 2021 },
        ("Array.prototype.p ||= 0", None), // { "ecmaVersion": 2021 },
        ("Array.prototype.p ??= 0", None), // { "ecmaVersion": 2021 }
    ];

    Tester::new(NoExtendNative::NAME, pass, fail).test_and_snapshot();
}
