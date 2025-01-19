use oxc_ast::{
    ast::{
        BinaryExpression, Expression, LogicalExpression, MemberExpression, StaticMemberExpression,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_boolean_ancestor, is_boolean_node},
    AstNode,
};

fn non_zero(span: Span, prop_name: &str, op_and_rhs: &str, help: Option<String>) -> OxcDiagnostic {
    let mut d = OxcDiagnostic::warn(format!(
        "Use `.{prop_name} {op_and_rhs}` when checking {prop_name} is not zero."
    ))
    .with_label(span);
    if let Some(x) = help {
        d = d.with_help(x);
    }
    d
}

fn zero(span: Span, prop_name: &str, op_and_rhs: &str, help: Option<String>) -> OxcDiagnostic {
    let mut d = OxcDiagnostic::warn(format!(
        "Use `.{prop_name} {op_and_rhs}` when checking {prop_name} is zero."
    ));
    if let Some(x) = help {
        d = d.with_help(x);
    }
    d.with_label(span)
}

#[derive(Debug, Default, Clone)]
enum NonZero {
    #[default]
    GreaterThan,
    NotEqual,
}
impl NonZero {
    pub fn from(raw: &str) -> Self {
        match raw {
            "not-equal" => Self::NotEqual,
            _ => Self::GreaterThan,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct ExplicitLengthCheck {
    non_zero: NonZero,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce explicitly comparing the length or size property of a value.
    ///
    /// The non-zero option can be configured with one of the following:
    /// greater-than (default)
    ///     Enforces non-zero to be checked with: foo.length > 0
    /// not-equal
    ///     Enforces non-zero to be checked with: foo.length !== 0
    ///
    /// ### Why is this bad?
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const isEmpty = foo.length == 0;
    /// const isEmpty = foo.length < 1;
    /// const isEmpty = 0 === foo.length;
    /// const isEmpty = 0 == foo.length;
    /// const isEmpty = 1 > foo.length;
    ///
    /// const isEmpty = !foo.length;
    /// const isEmpty = !(foo.length > 0);
    /// const isEmptySet = !foo.size;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const isEmpty = foo.length === 0;
    /// ```
    ExplicitLengthCheck,
    unicorn,
    pedantic,
    conditional_fix
);
fn is_literal(expr: &Expression, value: f64) -> bool {
    matches!(expr, Expression::NumericLiteral(lit) if (lit.value - value).abs() < f64::EPSILON)
}
fn is_compare_left(expr: &BinaryExpression, op: BinaryOperator, value: f64) -> bool {
    matches!(
        expr,
        BinaryExpression {
            operator,
            left,
            ..
        } if is_literal(left, value) && op == *operator
    )
}
fn is_compare_right(expr: &BinaryExpression, op: BinaryOperator, value: f64) -> bool {
    matches!(
        expr,
        BinaryExpression {
            operator,
            right,
            ..
        } if is_literal(right, value) && op == *operator
    )
}
fn get_length_check_node<'a, 'b>(
    node: &AstNode<'a>,
    ctx: &'b LintContext<'a>,
    // (is_zero_length_check, length_check_node)
) -> Option<(bool, &'b AstNode<'a>)> {
    let parent = ctx.nodes().parent_node(node.id());
    parent.and_then(|parent| {
        if let AstKind::BinaryExpression(binary_expr) = parent.kind() {
            // Zero length check
            // `foo.length === 0`
            if is_compare_right(binary_expr, BinaryOperator::StrictEquality, 0.0)
            // `foo.length == 0`
                || is_compare_right(binary_expr, BinaryOperator::Equality, 0.0)
                // `foo.length < 1`
                || is_compare_right(binary_expr, BinaryOperator::LessThan, 1.0)
                // `0 === foo.length`
                || is_compare_left(binary_expr, BinaryOperator::StrictEquality, 0.0)
                // `0 == foo.length`
                || is_compare_left(binary_expr, BinaryOperator::Equality, 0.0)
                // `1 > foo.length`
                || is_compare_left(binary_expr, BinaryOperator::GreaterThan, 1.0)
            {
                return Some((true, parent));
            }
            // Non-Zero length check
            // `foo.length !== 0`
            if is_compare_right(binary_expr, BinaryOperator::StrictInequality, 0.0)
            // `foo.length != 0`
                || is_compare_right(binary_expr, BinaryOperator::Inequality, 0.0)
                // `foo.length > 0`
                || is_compare_right(binary_expr, BinaryOperator::GreaterThan, 0.0)
                // `foo.length >= 1`
                || is_compare_right(binary_expr, BinaryOperator::GreaterEqualThan, 1.0)
                // `0 !== foo.length`
                || is_compare_left(binary_expr, BinaryOperator::StrictInequality, 0.0)
                // `0 !== foo.length`
                || is_compare_left(binary_expr, BinaryOperator::Inequality, 0.0)
                // `0 < foo.length`
                || is_compare_left(binary_expr, BinaryOperator::LessThan, 0.0)
                // `1 <= foo.length`
                || is_compare_left(binary_expr, BinaryOperator::LessEqualThan, 1.0)
            {
                return Some((false, parent));
            }
            return None;
        }
        None
    })
}

impl ExplicitLengthCheck {
    fn report<'a>(
        &self,
        ctx: &LintContext<'a>,
        node: &AstNode<'a>,
        is_zero_length_check: bool,
        static_member_expr: &StaticMemberExpression,
        auto_fix: bool,
    ) {
        let kind = node.kind();
        let check_code = if is_zero_length_check {
            if matches!(kind, AstKind::BinaryExpression(BinaryExpression{operator:BinaryOperator::StrictEquality,right,..}) if right.is_number_0())
            {
                return;
            }
            "=== 0"
        } else {
            match self.non_zero {
                NonZero::GreaterThan => {
                    if matches!(kind, AstKind::BinaryExpression(BinaryExpression{operator:BinaryOperator::GreaterThan,right,..}) if right.is_number_0())
                    {
                        return;
                    }
                    "> 0"
                }
                NonZero::NotEqual => {
                    if matches!(kind, AstKind::BinaryExpression(BinaryExpression{operator:BinaryOperator::StrictInequality,right,..}) if right.is_number_0())
                    {
                        return;
                    }
                    "!== 0"
                }
            }
        };

        let span = kind.span();
        let mut need_pad_start = false;
        let mut need_pad_end = false;
        let parent = ctx.nodes().parent_kind(node.id());
        let need_paren = matches!(kind, AstKind::UnaryExpression(_))
            && matches!(parent, Some(AstKind::UnaryExpression(_) | AstKind::AwaitExpression(_)));
        if span.start > 1 {
            let start = ctx.source_text().as_bytes()[span.start as usize - 1];
            need_pad_start = start.is_ascii_alphabetic() || !start.is_ascii();
        }
        if (span.end as usize) < ctx.source_text().len() {
            let end = ctx.source_text().as_bytes()[span.end as usize];
            need_pad_end = end.is_ascii_alphabetic() || !end.is_ascii();
        }

        let fixed = format!(
            "{}{}{} {}{}{}",
            if need_pad_start { " " } else { "" },
            if need_paren { "(" } else { "" },
            static_member_expr.span.source_text(ctx.source_text()),
            check_code,
            if need_paren { ")" } else { "" },
            if need_pad_end { " " } else { "" },
        );
        let property = static_member_expr.property.name;
        let help = if auto_fix {
            None
        } else {
            Some(format!("Replace `.{property}` with `.{property} {check_code}`."))
        };
        let diagnostic = if is_zero_length_check {
            zero(span, property.as_str(), check_code, help)
        } else {
            non_zero(span, property.as_str(), check_code, help)
        };
        if auto_fix {
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(span, fixed));
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}
impl Rule for ExplicitLengthCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::MemberExpression(MemberExpression::StaticMemberExpression(
            static_member_expr,
        )) = node.kind()
        {
            let StaticMemberExpression { object, property, .. } = &**static_member_expr;
            if property.name != "length" && property.name != "size" {
                return;
            }
            if let Expression::ThisExpression(_) = object {
                return;
            }

            if let Some((mut is_zero_length_check, length_check_node)) =
                get_length_check_node(node, ctx)
            {
                let (ancestor, is_negative) = get_boolean_ancestor(length_check_node, ctx);
                if is_negative {
                    is_zero_length_check = !is_zero_length_check;
                }
                self.report(ctx, ancestor, is_zero_length_check, static_member_expr, true);
            } else {
                let (ancestor, is_negative) = get_boolean_ancestor(node, ctx);
                if is_boolean_node(ancestor, ctx) {
                    self.report(ctx, ancestor, is_negative, static_member_expr, true);
                    return;
                }
                match ctx.nodes().parent_kind(node.id()) {
                    Some(AstKind::LogicalExpression(LogicalExpression {
                        operator, right, ..
                    })) if *operator == LogicalOperator::And
                        || (*operator == LogicalOperator::Or
                            && !matches!(right, Expression::NumericLiteral(_))) =>
                    {
                        self.report(ctx, ancestor, is_negative, static_member_expr, false);
                    }
                    _ => {}
                }
            };
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            non_zero: value
                .get(0)
                .and_then(|v| v.get("non-zero"))
                .and_then(serde_json::Value::as_str)
                .map(NonZero::from)
                .unwrap_or_default(),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Not `.length`
        ("if (foo.notLength) {}", None),
        ("if (length) {}", None),
        ("if (foo[length]) {}", None),
        (r#"if (foo["length"]) {}"#, None),
        // Already in wanted style
        ("foo.length === 0", None),
        ("foo.length > 0", None),
        // Not boolean
        ("const bar = foo.length", None),
        ("const bar = +foo.length", None),
        ("const x = Boolean(foo.length, foo.length)", None),
        ("const x = new Boolean(foo.length)", None),
        ("const x = NotBoolean(foo.length)", None),
        ("const length = foo.length ?? 0", None),
        ("if (foo.length ?? bar) {}", None),
        // Checking 'non-zero'
        ("if (foo.length > 0) {}", None),
        ("if (foo.length > 0) {}", Some(serde_json::json!([{"non-zero": "greater-than"}]))),
        ("if (foo.length !== 0) {}", Some(serde_json::json!([{"non-zero": "not-equal"}]))),
        // Checking "non-zero"
        ("if (foo.length === 0) {}", None),
        // `ConditionalExpression`
        ("const bar = foo.length === 0 ? 1 : 2", None),
        ("while (foo.length > 0) { foo.pop(); }", None),
        ("do { foo.pop(); } while (foo.length > 0);", None),
        // `ForStatement`
        ("for (; foo.length > 0; foo.pop());", None),
        ("if (foo.length !== 1) {}", None),
        ("if (foo.length > 1) {}", None),
        ("if (foo.length < 2) {}", None),
        // need getStaticValue
        // With known static length value
        // (r#"const foo = { size: "small" }; if (foo.size) {}"#, None), // Not a number
        // ("const foo = { length: -1 }; if (foo.length) {}", None), // Array lengths cannot be negative
        // ("const foo = { length: 1.5 }; if (foo.length) {}", None), // Array lengths must be integers
        // ("const foo = { length: NaN }; if (foo.length) {}", None), // Array lengths cannot be NaN
        // ("const foo = { length: Infinity }; if (foo.length) {}", None), // Array lengths cannot be Infinity
        // Logical OR
        ("const x = foo.length || 2", None),
        // need getStaticValue
        // ("const A_NUMBER = 2; const x = foo.length || A_NUMBER", None),
        ("class A { a(){ if(this.length); while(!this.size || foo);}}", None),
        // Use of .size but not in conditional "test" position
        (
            "const totalCount = tests.reduce((count, test) => count + (test.enabled ? test.maxSize : test.size), 0)",
            None,
        ),
    ];

    let fail = vec![
        ("const x = foo.length || bar()", None),
        ("const x = foo.length || unknown", None),
        ("bar(!foo.length || foo.length)", None),
        // need getStaticValue
        // (r#"const NON_NUMBER = "2"; const x = foo.length || NON_NUMBER"#, None),
        ("const x = foo.length || bar()", Some(serde_json::json!([{"non-zero": "not-equal"}]))),
        ("const x = foo.length || bar()", Some(serde_json::json!([{"non-zero": "greater-than"}]))),
        ("const x = foo.length || bar()", None),
        ("() => foo.length && bar()", None),
        ("alert(foo.length && bar())", None),
        // Use of .size in conditional "test" position
        ("let foo = arr.length ? 'non-empty' : 'empty'", None),
    ];

    let fixes = vec![
        (
            r"if ( !!!( !foo.length && foo.length == 0 && foo.length < 1 && 0 === foo.length && 0 == foo.length && 1 > foo.length ) ||
!( foo.length || !!foo.length || foo.length !== 0 || foo.length != 0 || foo.length >= 1 || 0 !== foo.length || 0 != foo.length || 0 < foo.length || 1 <= foo.length )
            ) {}",
            "if ( !!!( foo.length === 0 && foo.length === 0 && foo.length === 0 && foo.length === 0 && foo.length === 0 && foo.length === 0 ) ||
!( foo.length > 0 || foo.length > 0 || foo.length > 0 || foo.length > 0 || foo.length > 0 || foo.length > 0 || foo.length > 0 || foo.length > 0 || foo.length > 0 )
            ) {}",
            None,
        ),
        ("if ( foo.length || !!foo.length || foo.length != 0 || foo.length > 0 || foo.length >= 1 || 0 !== foo.length || 0 != foo.length || 0 < foo.length || 1 <= foo.length ) {}", "if ( foo.length !== 0 || foo.length !== 0 || foo.length !== 0 || foo.length !== 0 || foo.length !== 0 || foo.length !== 0 || foo.length !== 0 || foo.length !== 0 || foo.length !== 0 ) {}", Some(serde_json::json!([{"non-zero": "not-equal"}]))),
        ("const foo = { length: 123 }; if (foo.length) {}", "const foo = { length: 123 }; if (foo.length !== 0) {}", Some(serde_json::json!([{"non-zero": "not-equal"}]))),
        ("if (foo.bar && foo.bar.length) {}", "if (foo.bar && foo.bar.length > 0) {}", None),
        ("if (foo.length || foo.bar()) {}", "if (foo.length > 0 || foo.bar()) {}", None),
        ("if (!!(!!foo.length)) {}", "if (foo.length > 0) {}", None),
        ("if (!(foo.length === 0)) {}", "if (foo.length > 0) {}", None),
        ("while (foo.length >= 1) {}", "while (foo.length > 0) {}", None),
        ("do {} while (foo.length);", "do {} while (foo.length > 0);", None),
        ("for (let i = 0; (bar && !foo.length); i ++) {}", "for (let i = 0; (bar && foo.length === 0); i ++) {}", None),
        ("const isEmpty = foo.length < 1;", "const isEmpty = foo.length === 0;", None),
        ("bar(foo.length >= 1)", "bar(foo.length > 0)", None),
        // ("const bar = void !foo.length;", "const bar = void (foo.length === 0);", None),
        ("const isNotEmpty = Boolean(foo.length)", "const isNotEmpty = foo.length > 0", None),
        ("const isNotEmpty = Boolean(foo.length || bar)", "const isNotEmpty = Boolean(foo.length > 0 || bar)", None),
        ("const isEmpty = Boolean(!foo.length)", "const isEmpty = foo.length === 0", None),
        ("const isEmpty = Boolean(foo.length === 0)", "const isEmpty = foo.length === 0", None),
        ("const isNotEmpty = !Boolean(foo.length === 0)", "const isNotEmpty = foo.length > 0", None),
        ("const isEmpty = !Boolean(!Boolean(foo.length === 0))", "const isEmpty = foo.length === 0", None),
        ("if (foo.size) {}", "if (foo.size > 0) {}", None),
        ("if (foo.size && bar.length) {}", "if (foo.size > 0 && bar.length > 0) {}", None),
        // Space after keywords
        ("function foo() {return!foo.length}", "function foo() {return foo.length === 0}", None),
        ("function foo() {throw!foo.length}", "function foo() {throw foo.length === 0}", None),
        ("async function foo() {await!foo.length}", "async function foo() {await (foo.length === 0)}", None),
        ("function * foo() {yield!foo.length}", "function * foo() {yield foo.length === 0}", None),
        ("function * foo() {yield*!foo.length}", "function * foo() {yield*foo.length === 0}", None),
        ("delete!foo.length", "delete (foo.length === 0)", None),
        ("typeof!foo.length", "typeof (foo.length === 0)", None),
        ("void!foo.length", "void (foo.length === 0)", None),
        ("a instanceof!foo.length", "a instanceof foo.length === 0", None),
        ("a in!foo.length", "a in foo.length === 0", None),
        ("export default!foo.length", "export default foo.length === 0", None),
        ("if(true){}else!foo.length", "if(true){}else foo.length === 0", None),
        ("do!foo.length;while(true) {}", "do foo.length === 0;while(true) {}", None),
        ("switch(foo){case!foo.length:{}}", "switch(foo){case foo.length === 0:{}}", None),
        ("for(const a of!foo.length);", "for(const a of foo.length === 0);", None),
        ("for(const a in!foo.length);", "for(const a in foo.length === 0);", None),
    ];
    Tester::new(ExplicitLengthCheck::NAME, ExplicitLengthCheck::PLUGIN, pass, fail)
        .expect_fix(fixes)
        .test_and_snapshot();
}
