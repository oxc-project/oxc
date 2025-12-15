use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpressionElement, AssignmentTarget, AssignmentTargetMaybeDefault,
        AssignmentTargetProperty, Expression, MemberExpression, ObjectProperty, ObjectPropertyKind,
        SimpleAssignmentTarget, match_assignment_target, match_simple_assignment_target,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::AssignmentOperator;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_self_assign_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("this expression is assigned to itself").with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoSelfAssign {
    /// The `props` option when set to `false`, disables the checking of properties.
    ///
    /// With `props` set to `false` the following are examples of correct code:
    /// ```javascript
    /// obj.a = obj.a;
    /// obj.a.b = obj.a.b;
    /// obj["a"] = obj["a"];
    /// obj[a] = obj[a];
    /// ```
    props: bool,
}

impl Default for NoSelfAssign {
    fn default() -> Self {
        Self { props: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow assignments where both sides are exactly the same.
    ///
    /// ### Why is this bad?
    ///
    /// Self assignments have no effect, so probably those are an error due to incomplete
    /// refactoring. Those indicate that what you should do is still remaining.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo = foo;
    ///
    /// [a, b] = [a, b];
    /// [a, ...b] = [x, ...b];
    ///
    /// ({a, b} = {a, x});
    ///
    /// foo &&= foo;
    /// foo ||= foo;
    /// foo ??= foo;
    /// ```
    ///
    /// ```javascript
    /// obj.a = obj.a;
    /// obj.a.b = obj.a.b;
    /// obj["a"] = obj["a"];
    /// obj[a] = obj[a];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo = bar;
    /// [a, b] = [b, a];
    ///
    /// // This pattern is warned by the `no-use-before-define` rule.
    /// let foo = foo;
    ///
    /// // The default values have an effect.
    /// [foo = 1] = [foo];
    ///
    /// // This ignores if there is a function call.
    /// obj.a().b = obj.a().b;
    /// a().b = a().b;
    ///
    /// // `&=` and `|=` have an effect on non-integers.
    /// foo &= foo;
    /// foo |= foo;
    /// ```
    NoSelfAssign,
    eslint,
    correctness,
    config = NoSelfAssign
);

impl Rule for NoSelfAssign {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoSelfAssign>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assignment) = node.kind() else {
            return;
        };
        if matches!(
            assignment.operator,
            AssignmentOperator::Assign
                | AssignmentOperator::LogicalAnd
                | AssignmentOperator::LogicalOr
                | AssignmentOperator::LogicalNullish
        ) {
            self.each_self_assignment(&assignment.left, &assignment.right, ctx);
        }
    }
}

impl NoSelfAssign {
    fn each_self_assignment<'a>(
        &self,
        left: &'a AssignmentTarget<'a>,
        right: &'a Expression<'a>,
        ctx: &LintContext<'a>,
    ) {
        match left {
            match_simple_assignment_target!(AssignmentTarget) => {
                let simple_assignment_target = left.to_simple_assignment_target();
                if let Expression::Identifier(id2) = right.without_parentheses() {
                    let self_assign = matches!(simple_assignment_target.get_expression(), Some(Expression::Identifier(id1)) if id1.name == id2.name)
                        || matches!(simple_assignment_target, SimpleAssignmentTarget::AssignmentTargetIdentifier(id1) if id1.name == id2.name);

                    if self_assign {
                        ctx.diagnostic(no_self_assign_diagnostic(right.span()));
                    }
                }

                let Some(member_target) = simple_assignment_target.as_member_expression() else {
                    return;
                };

                let Some(member_expr) = right.without_parentheses().get_member_expr() else {
                    return;
                };

                if self.is_member_expression_same_reference(member_expr, member_target) {
                    ctx.diagnostic(no_self_assign_diagnostic(member_expr.span()));
                }
            }

            AssignmentTarget::ArrayAssignmentTarget(array_pattern) => {
                let Expression::ArrayExpression(array_expr) = right.without_parentheses() else {
                    return;
                };
                let end = std::cmp::min(array_pattern.elements.len(), array_expr.elements.len());
                let mut i = 0;
                while i < end {
                    let left = array_pattern.elements[i].as_ref();
                    let right = &array_expr.elements[i];

                    if let Some(left) = left
                        && let Some(left_target) = left.as_assignment_target()
                        && let Some(expr) = right.as_expression()
                    {
                        self.each_self_assignment(left_target, expr, ctx);
                    }

                    // After a spread element, those indices are unknown.
                    if let ArrayExpressionElement::SpreadElement(_) = right {
                        break;
                    }

                    i += 1;
                }
            }

            AssignmentTarget::ObjectAssignmentTarget(object_pattern) => {
                let Expression::ObjectExpression(object_expr) = right.get_inner_expression() else {
                    return;
                };

                if !object_expr.properties.is_empty() {
                    let mut start_j = 0;
                    let mut i = object_expr.properties.len();
                    while i >= 1 {
                        if let ObjectPropertyKind::SpreadProperty(_) = object_expr.properties[i - 1]
                        {
                            start_j = i;
                            break;
                        }
                        i -= 1;
                    }

                    let mut i = 0;
                    while i < object_pattern.properties.len() {
                        let mut j = start_j;
                        while j < object_expr.properties.len() {
                            let left = &object_pattern.properties[i];
                            let right = &object_expr.properties[j];

                            self.each_property_self_assignment(left, right, ctx);

                            j += 1;
                        }
                        i += 1;
                    }
                }
            }
        }
    }

    fn is_same_reference<'a>(&self, left: &'a Expression<'a>, right: &'a Expression<'a>) -> bool {
        let left = left.get_inner_expression();
        let right = right.get_inner_expression();

        if matches!(
            (left, right),
            (Expression::Super(_), Expression::Super(_))
                | (Expression::ThisExpression(_), Expression::ThisExpression(_))
        ) {
            return true;
        }

        if let (Expression::Identifier(id1), Expression::Identifier(id2)) = (left, right) {
            return id1.name == id2.name;
        }

        if let (Some(member1), Some(member2)) = (left.get_member_expr(), right.get_member_expr()) {
            self.is_member_expression_same_reference(member1, member2)
        } else {
            false
        }
    }

    fn is_member_expression_same_reference<'a>(
        &self,
        member1: &'a MemberExpression<'a>,
        member2: &'a MemberExpression<'a>,
    ) -> bool {
        if !self.props {
            return false;
        }
        let member1_static_property_name = member1.static_property_name();
        if member1_static_property_name.is_some()
            && member1_static_property_name == member2.static_property_name()
        {
            return self.is_same_reference(member1.object(), member2.object());
        }

        if matches!(member1, MemberExpression::ComputedMemberExpression(_))
            == matches!(member2, MemberExpression::ComputedMemberExpression(_))
            && self.is_same_reference(member1.object(), member2.object())
        {
            return match (member1, member2) {
                (
                    MemberExpression::ComputedMemberExpression(computed1),
                    MemberExpression::ComputedMemberExpression(computed2),
                ) => self.is_same_reference(&computed1.expression, &computed2.expression),
                (
                    MemberExpression::PrivateFieldExpression(private1),
                    MemberExpression::PrivateFieldExpression(private2),
                ) => private1.field.name == private2.field.name,
                _ => false,
            };
        }

        false
    }

    fn each_property_self_assignment<'a>(
        &self,
        left: &'a AssignmentTargetProperty<'a>,
        right: &'a ObjectPropertyKind<'a>,
        ctx: &LintContext<'a>,
    ) {
        match left {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id1)
                if id1.init.is_none() =>
            {
                let ObjectPropertyKind::ObjectProperty(obj_prop) = right else {
                    return;
                };

                let ObjectProperty { method: false, value: expr, span, key, .. } = &**obj_prop
                else {
                    return;
                };
                if key.static_name().is_some_and(|name| name == id1.binding.name)
                    && let Expression::Identifier(id2) = expr.without_parentheses()
                    && id1.binding.name == id2.name
                {
                    ctx.diagnostic(no_self_assign_diagnostic(*span));
                }
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                let left = match &property.binding {
                    binding @ match_assignment_target!(AssignmentTargetMaybeDefault) => {
                        binding.to_assignment_target()
                    }
                    AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(_) => return,
                };
                let ObjectPropertyKind::ObjectProperty(obj_prop) = right else {
                    return;
                };
                let ObjectProperty { method: false, value: expr, key, .. } = &**obj_prop else {
                    return;
                };

                let property_name = property.name.static_name();
                let key_name = key.static_name();
                if property_name.is_some() && property_name == key_name {
                    self.each_self_assignment(left, expr, ctx);
                }
            }
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a = a", None),
        ("a = b", None),
        ("a += a", None),
        ("a = +a", None),
        ("a = [a]", None),
        ("a &= a", None),
        ("a |= a", None),
        ("let a = a", None),
        ("const a = a", None),
        ("[a] = a", None),
        ("[a = 1] = [a]", None),
        ("[a, b] = [b, a]", None),
        ("[a,, b] = [, b, a]", None),
        ("[x, a] = [...x, a]", None),
        ("[...a] = [...a, 1]", None),
        ("[a, ...b] = [0, ...b, 1]", None),
        ("[a, b] = {a, b}", None),
        ("({a} = a)", None),
        ("[foo = 1] = [foo];", None),
        ("({a = 1} = {a})", None),
        ("({a: b} = {a})", None),
        ("({a} = {a: b})", None),
        ("({a} = {a() {}})", None),
        ("({a} = {[a]: a})", None),
        ("({[a]: b} = {[a]: b})", None),
        ("({'foo': a, 1: a} = {'bar': a, 2: a})", None),
        ("({a, ...b} = {a, ...b})", None),
        ("a.b = a.c", Some(serde_json::json!([{ "props": true }]))),
        ("a.b = c.b", Some(serde_json::json!([{ "props": true }]))),
        ("a.b = a[b]", Some(serde_json::json!([{ "props": true }]))),
        ("a[b] = a.b", Some(serde_json::json!([{ "props": true }]))),
        ("a.b().c = a.b().c", Some(serde_json::json!([{ "props": true }]))),
        ("b().c = b().c", Some(serde_json::json!([{ "props": true }]))),
        ("a.null = a[/(?<zero>0)/]", Some(serde_json::json!([{ "props": true }]))),
        ("a[b + 1] = a[b + 1]", Some(serde_json::json!([{ "props": true }]))),
        ("a.b = a.b", Some(serde_json::json!([{ "props": false }]))),
        ("a.b.c = a.b.c", Some(serde_json::json!([{ "props": false }]))),
        ("a[b] = a[b]", Some(serde_json::json!([{ "props": false }]))),
        ("a['b'] = a['b']", Some(serde_json::json!([{ "props": false }]))),
        (r#"obj[a] = obj["a"];"#, None),
        ("a[\n    'b'\n] = a[\n    'b'\n]", Some(serde_json::json!([{ "props": false }]))),
        ("this.x = this.y", Some(serde_json::json!([{ "props": true }]))),
        ("this.x = this.x", Some(serde_json::json!([{ "props": false }]))),
        ("class C { #field; foo() { this['#field'] = this.#field; } }", None),
        ("class C { #field; foo() { this.#field = this['#field']; } }", None),
        (r#"obj["a" + "b"] = obj["a" + "b"];"#, None),
        ("obj[a + b] = obj[a + b];", None),
        // `&=` and `|=` have an effect on non-integers.
        ("foo |= foo;", None),
        ("foo &= foo;", None),
        ("let foo = foo;", None),
    ];

    let fail = vec![
        ("a = a", None),
        ("[a] = [a]", None),
        ("[a, b] = [a, b]", None),
        ("[a, b] = [a, c]", None),
        ("[a, b] = [, b]", None),
        ("[a, ...b] = [a, ...b]", None),
        ("[[a], {b}] = [[a], {b}]", None),
        ("({a} = {a})", None),
        ("({a: b} = {a: b})", None),
        ("({'a': b} = {'a': b})", None),
        ("({a: b} = {'a': b})", None),
        ("({'a': b} = {a: b})", None),
        ("({1: b} = {1: b})", None),
        ("({1: b} = {'1': b})", None),
        ("({'1': b} = {1: b})", None),
        ("({['a']: b} = {a: b})", None),
        ("({'a': b} = {[`a`]: b})", None),
        ("({1: b} = {[1]: b})", None),
        ("({a, b} = {a, b})", None),
        ("({a, b} = {b, a})", None),
        ("({a, b} = {c, a})", None),
        ("({a: {b}, c: [d]} = {a: {b}, c: [d]})", None),
        ("({a, b} = {a, ...x, b})", None),
        ("a.b = a.b", None),
        ("a.b.c = a.b.c", None),
        ("a[b] = a[b]", None),
        ("a['b'] = a['b']", None),
        ("a[\n    'b'\n] = a[\n    'b'\n]", None),
        ("a.b = a.b", Some(serde_json::json!([{ "props": true }]))),
        ("a.b.c = a.b.c", Some(serde_json::json!([{ "props": true }]))),
        ("a[b] = a[b]", Some(serde_json::json!([{ "props": true }]))),
        ("a['b'] = a['b']", Some(serde_json::json!([{ "props": true }]))),
        ("a[\n    'b'\n] = a[\n    'b'\n]", Some(serde_json::json!([{ "props": true }]))),
        ("this.x = this.x", Some(serde_json::json!([{ "props": true }]))),
        // TODO: <https://github.com/eslint/eslint/blob/eb3d7946e1e9f70254008744dba2397aaa730114/lib/rules/utils/ast-utils.js#L362>
        // ("a['/(?<zero>0)/'] = a[/(?<zero>0)/]", Some(serde_json::json!([{ "props": true }]))),
        ("(a?.b).c = (a?.b).c", None),
        ("a.b = a?.b", None),
        ("class C { #field; foo() { this.#field = this.#field; } }", None),
        ("class C { #field; foo() { [this.#field] = [this.#field]; } }", None),
        ("a &&= a", None),
        ("a ||= a", None),
        ("a ??= a", None),
    ];

    Tester::new(NoSelfAssign::NAME, NoSelfAssign::PLUGIN, pass, fail).test_and_snapshot();
}
