use oxc_allocator::Box as OBox;
use oxc_ast::{
    ast::{
        ArrayExpressionElement, AssignmentTarget, AssignmentTargetMaybeDefault,
        AssignmentTargetPattern, AssignmentTargetProperty, ChainElement, ChainExpression,
        Expression, MemberExpression, ObjectProperty, ObjectPropertyKind, SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::AssignmentOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-self-assign): this expression is assigned to itself")]
#[diagnostic(severity(warning))]
struct NoSelfAssignDiagnostic(#[label] pub Span);

#[derive(Debug, Clone)]
pub struct NoSelfAssign {
    /// if this is true, no-self-assign rule warns self-assignments of properties. Default is true.
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
    /// Disallow assignments where both sides are exactly the same
    ///
    /// ### Why is this bad?
    ///
    /// Self assignments have no effect, so probably those are an error due to incomplete refactoring. Those indicate that what you should do is still remaining.
    ///
    /// ### Example
    /// ```javascript
    /// foo = foo;
    /// [bar, baz] = [bar, qiz];
    /// ```
    NoSelfAssign,
    correctness
);

impl Rule for NoSelfAssign {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            props: value
                .get(0)
                .and_then(|v| v.get("props"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AssignmentExpression(assignment) = node.kind()
          && matches!(
            assignment.operator,
            AssignmentOperator::Assign | AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish
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
        AssignmentTarget::SimpleAssignmentTarget(simple_assignment_target) => {
            if let Expression::Identifier(id2) = right.without_parenthesized() {
                let mut self_assign = false;
                if let Some(Expression::Identifier(id1)) = simple_assignment_target.get_expression() && id1.name == id2.name {
                    self_assign = true;
                } else if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id1) = simple_assignment_target && id1.name == id2.name {
                    self_assign = true;
                }

                if self_assign {
                    ctx.diagnostic(NoSelfAssignDiagnostic(
                        right.span(),
                    ));
                }
            }
        },

            AssignmentTarget::AssignmentTargetPattern(
                AssignmentTargetPattern::ArrayAssignmentTarget(array_pattern),
            ) => {
            if let Expression::ArrayExpression(array_expr) = right.without_parenthesized() {
                let end = std::cmp::min(array_pattern.elements.len(), array_expr.elements.len());
                let mut i = 0;
                while i < end {
                    let left = array_pattern.elements[i].as_ref();
                    let right = &array_expr.elements[i];

                    let left_target = match left {
                        Some(AssignmentTargetMaybeDefault::AssignmentTarget(target)) => {
                            Some(target)
                        }
                        _ => None,
                    };


                    if let Some(left_target) = left_target {
                      if let ArrayExpressionElement::Expression(expr) = right {
                            self.each_self_assignment(left_target, expr, ctx);
                        }
                    }

                    // After a spread element, those indices are unknown.
                    if let ArrayExpressionElement::SpreadElement(_) = right {
                        break;
                    }

                    i += 1;
                }
            }
        }
            AssignmentTarget::AssignmentTargetPattern(
                AssignmentTargetPattern::ObjectAssignmentTarget(object_pattern),
            ) => if let Expression::ObjectExpression(object_expr) = right.get_inner_expression() && !object_expr.properties.is_empty() {
                let mut start_j = 0;
                let mut i = object_expr.properties.len();
                while i >= 1 {
                    if let ObjectPropertyKind::SpreadProperty(_) = object_expr.properties[i - 1] {
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
        },
    }

        if let AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberAssignmentTarget(member_target),
        ) = &left
        {
            if let Expression::MemberExpression(member_expr)
            | Expression::ChainExpression(OBox(ChainExpression {
                expression: ChainElement::MemberExpression(member_expr),
                ..
            })) = right.without_parenthesized()
            {
                if self.is_member_expression_same_reference(member_expr, member_target) {
                    ctx.diagnostic(NoSelfAssignDiagnostic(member_expr.span()));
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

        match (left, right) {
            (Expression::Identifier(id1), Expression::Identifier(id2)) => id1.name == id2.name,
            (Expression::MemberExpression(member1), Expression::MemberExpression(member2)) => {
                self.is_member_expression_same_reference(member1, member2)
            }
            (
                Expression::ChainExpression(OBox(ChainExpression {
                    expression: ChainElement::MemberExpression(member1),
                    ..
                }))
                | Expression::MemberExpression(member1),
                Expression::ChainExpression(OBox(ChainExpression {
                    expression: ChainElement::MemberExpression(member2),
                    ..
                })),
            ) => self.is_member_expression_same_reference(member1, member2),
            _ => false,
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
        if let Some(name1) = member1.static_property_name() && name1 == member2.static_property_name().unwrap_or_default() {
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
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id1) if id1.init.is_none() => {
          if let ObjectPropertyKind::ObjectProperty(OBox(ObjectProperty {
              method: false,
              value: expr,
              span,
              key,
              ..
          })) = right && key.static_name().is_some_and(|name|name == id1.binding.name)
              && let Expression::Identifier(id2) = expr.without_parenthesized() && id1.binding.name == id2.name {
                  ctx.diagnostic(NoSelfAssignDiagnostic(*span));
              }
        }
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
            let left = match &property.binding {
                AssignmentTargetMaybeDefault::AssignmentTarget(target) => target,
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(_) => {
                    return;
                }
            };
            if let ObjectPropertyKind::ObjectProperty(OBox(ObjectProperty {
                method: false,
                value: expr,
                key,
                ..
            })) = right
            {
                let property_name = property.name.static_name();
                let key_name = key.static_name();
                if property_name.is_some() && property_name == key_name {
                    self.each_self_assignment(left, expr, ctx);
                }
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
        ("a[\n    'b'\n] = a[\n    'b'\n]", Some(serde_json::json!([{ "props": false }]))),
        ("this.x = this.y", Some(serde_json::json!([{ "props": true }]))),
        ("this.x = this.x", Some(serde_json::json!([{ "props": false }]))),
        ("class C { #field; foo() { this['#field'] = this.#field; } }", None),
        ("class C { #field; foo() { this.#field = this['#field']; } }", None),
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

    Tester::new(NoSelfAssign::NAME, pass, fail).test_and_snapshot();
}
