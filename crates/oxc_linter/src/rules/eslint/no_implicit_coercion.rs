use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn boolean_coercion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected implicit coercion to boolean")
        .with_help("Use `Boolean(value)` instead")
        .with_label(span)
}

fn number_coercion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected implicit coercion to number")
        .with_help("Use `Number(value)` instead")
        .with_label(span)
}

fn string_coercion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected implicit coercion to string")
        .with_help("Use `String(value)` instead")
        .with_label(span)
}

/// Type of implicit coercion being detected
#[derive(Clone, Copy)]
enum CoercionKind {
    Boolean,
    Number,
    String,
}

/// Reports an implicit coercion with an auto-fix suggestion
fn report_coercion(ctx: &LintContext, span: Span, kind: CoercionKind, operand: &str) {
    let (diagnostic, wrapper) = match kind {
        CoercionKind::Boolean => (boolean_coercion_diagnostic(span), "Boolean"),
        CoercionKind::Number => (number_coercion_diagnostic(span), "Number"),
        CoercionKind::String => (string_coercion_diagnostic(span), "String"),
    };
    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        fixer.replace(span, format!("{wrapper}({operand})"))
    });
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoImplicitCoercionConfig {
    /// When `true`, warns on implicit boolean coercion (e.g., `!!foo`).
    /// Default: `true`
    boolean: bool,
    /// When `true`, warns on implicit number coercion (e.g., `+foo`).
    /// Default: `true`
    number: bool,
    /// When `true`, warns on implicit string coercion (e.g., `"" + foo`).
    /// Default: `true`
    string: bool,
    /// When `true`, disallows using template literals for string coercion (e.g., `` `${foo}` ``).
    /// Default: `false`
    disallow_template_shorthand: bool,
    /// List of operators to allow. Valid values: `"!!"`, `"~"`, `"+"`, `"-"`, `"- -"`, `"*"`
    #[schemars(with = "Vec<String>")]
    allow: AllowedOperators,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct AllowedOperators: u8 {
        const DOUBLE_NOT = 1 << 0;    // !!
        const TILDE = 1 << 1;         // ~
        const PLUS = 1 << 2;          // +
        const DOUBLE_MINUS = 1 << 3;  // - -
        const MINUS = 1 << 4;         // -
        const ASTERISK = 1 << 5;      // *
    }
}

impl<'de> Deserialize<'de> for AllowedOperators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, SeqAccess, Visitor};

        struct AllowedOperatorsVisitor;

        impl<'de> Visitor<'de> for AllowedOperatorsVisitor {
            type Value = AllowedOperators;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of operator strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut flags = AllowedOperators::empty();
                while let Some(s) = seq.next_element::<CompactStr>()? {
                    match s.as_str() {
                        "!!" => flags |= AllowedOperators::DOUBLE_NOT,
                        "~" => flags |= AllowedOperators::TILDE,
                        "+" => flags |= AllowedOperators::PLUS,
                        "- -" => flags |= AllowedOperators::DOUBLE_MINUS,
                        "-" => flags |= AllowedOperators::MINUS,
                        "*" => flags |= AllowedOperators::ASTERISK,
                        other => {
                            return Err(A::Error::custom(format!(
                                "Invalid operator '{other}' in allow list. Valid values are: \"!!\", \"~\", \"+\", \"- -\", \"-\", \"*\""
                            )));
                        }
                    }
                }
                Ok(flags)
            }
        }

        deserializer.deserialize_seq(AllowedOperatorsVisitor)
    }
}

impl Default for NoImplicitCoercionConfig {
    fn default() -> Self {
        Self {
            boolean: true,
            number: true,
            string: true,
            disallow_template_shorthand: false,
            allow: AllowedOperators::empty(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoImplicitCoercion(Box<NoImplicitCoercionConfig>);

impl std::ops::Deref for NoImplicitCoercion {
    type Target = NoImplicitCoercionConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows shorthand type conversions using operators like `!!`, `+`, `""+ `, etc.
    ///
    /// ### Why is this bad?
    ///
    /// Implicit type coercions using operators can be less clear than using explicit
    /// type conversion functions like `Boolean()`, `Number()`, and `String()`.
    /// Using explicit conversions makes the intent clearer and the code more readable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var b = !!foo;
    /// var n = +foo;
    /// var s = "" + foo;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var b = Boolean(foo);
    /// var n = Number(foo);
    /// var s = String(foo);
    /// ```
    NoImplicitCoercion,
    eslint,
    style,
    fix,
    config = NoImplicitCoercionConfig,
);

impl Rule for NoImplicitCoercion {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = serde_json::from_value::<DefaultRuleConfig<NoImplicitCoercionConfig>>(value)
            .unwrap_or_default()
            .into_inner();
        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::UnaryExpression(unary_expr) => {
                // Check for !!foo (boolean coercion)
                if self.boolean
                    && unary_expr.operator == UnaryOperator::LogicalNot
                    && !self.is_allowed(AllowedOperators::DOUBLE_NOT)
                    && let Expression::UnaryExpression(inner) = &unary_expr.argument
                    && inner.operator == UnaryOperator::LogicalNot
                {
                    let operand = get_operand_text(ctx, &inner.argument);
                    report_coercion(ctx, unary_expr.span, CoercionKind::Boolean, operand);
                    return;
                }

                // Check for +foo (number coercion) - but not for numeric literals
                if self.number
                    && unary_expr.operator == UnaryOperator::UnaryPlus
                    && !self.is_allowed(AllowedOperators::PLUS)
                    && !is_numeric_literal(&unary_expr.argument)
                    && !is_already_numeric(&unary_expr.argument)
                {
                    let operand = get_operand_text(ctx, &unary_expr.argument);
                    report_coercion(ctx, unary_expr.span, CoercionKind::Number, operand);
                    return;
                }

                // Check for - -foo (number coercion via double negation)
                if self.number
                    && unary_expr.operator == UnaryOperator::UnaryNegation
                    && !self.is_allowed(AllowedOperators::DOUBLE_MINUS)
                    && let Expression::UnaryExpression(inner) =
                        unary_expr.argument.without_parentheses()
                    && inner.operator == UnaryOperator::UnaryNegation
                    && !is_numeric_literal(&inner.argument)
                    && !is_already_numeric(&inner.argument)
                {
                    let operand = get_operand_text(ctx, &inner.argument);
                    report_coercion(ctx, unary_expr.span, CoercionKind::Number, operand);
                    return;
                }

                // Check for ~foo.indexOf() (boolean coercion)
                if self.boolean
                    && unary_expr.operator == UnaryOperator::BitwiseNot
                    && !self.is_allowed(AllowedOperators::TILDE)
                    && is_indexof_call(&unary_expr.argument)
                {
                    ctx.diagnostic(boolean_coercion_diagnostic(unary_expr.span));
                }
            }
            AstKind::BinaryExpression(bin_expr) => {
                // Check for foo * 1 or 1 * foo (number coercion)
                if self.number
                    && bin_expr.operator == BinaryOperator::Multiplication
                    && !self.is_allowed(AllowedOperators::ASTERISK)
                {
                    let left_is_one = bin_expr.left.without_parentheses().is_number_value(1.0);
                    let right_is_one = bin_expr.right.without_parentheses().is_number_value(1.0);

                    if left_is_one && !is_already_numeric(&bin_expr.right) {
                        // Check if this is part of a larger expression like `1 * a / 2`
                        // We should still report `1 * a` in such cases
                        let operand = get_operand_text(ctx, &bin_expr.right);
                        report_coercion(ctx, bin_expr.span, CoercionKind::Number, operand);
                        return;
                    }
                    if right_is_one
                        && !is_already_numeric(&bin_expr.left)
                        && !is_part_of_larger_binary_expression(ctx, node)
                    {
                        let operand = get_operand_text(ctx, &bin_expr.left);
                        report_coercion(ctx, bin_expr.span, CoercionKind::Number, operand);
                        return;
                    }
                }

                // Check for foo - 0 (number coercion)
                if self.number
                    && bin_expr.operator == BinaryOperator::Subtraction
                    && !self.is_allowed(AllowedOperators::MINUS)
                    && bin_expr.right.is_number_0()
                    && !is_already_numeric(&bin_expr.left)
                {
                    let operand = get_operand_text(ctx, &bin_expr.left);
                    report_coercion(ctx, bin_expr.span, CoercionKind::Number, operand);
                    return;
                }

                // Check for "" + foo or foo + "" (string coercion)
                if self.string
                    && bin_expr.operator == BinaryOperator::Addition
                    && !self.is_allowed(AllowedOperators::PLUS)
                {
                    let left_is_empty_string = is_empty_string(&bin_expr.left);
                    let right_is_empty_string = is_empty_string(&bin_expr.right);

                    if left_is_empty_string && !is_string_literal_or_string_call(&bin_expr.right) {
                        let operand = get_operand_text(ctx, &bin_expr.right);
                        report_coercion(ctx, bin_expr.span, CoercionKind::String, operand);
                        return;
                    }
                    if right_is_empty_string && !is_string_literal_or_string_call(&bin_expr.left) {
                        let operand = get_operand_text(ctx, &bin_expr.left);
                        report_coercion(ctx, bin_expr.span, CoercionKind::String, operand);
                    }
                }
            }
            AstKind::AssignmentExpression(assign_expr) => {
                // Check for foo += "" (string coercion)
                if self.string
                    && assign_expr.operator == AssignmentOperator::Addition
                    && !self.is_allowed(AllowedOperators::PLUS)
                    && is_empty_string(&assign_expr.right)
                {
                    ctx.diagnostic(string_coercion_diagnostic(assign_expr.span));
                }
            }
            AstKind::TemplateLiteral(template) => {
                // Check for `${foo}` (string coercion via template literal)
                // Skip if this is a tagged template literal (e.g., tag`${foo}`)
                if self.string
                    && self.disallow_template_shorthand
                    && template.quasis.len() == 2
                    && template.expressions.len() == 1
                    && !is_tagged_template(ctx, node)
                {
                    let first_quasi = &template.quasis[0];
                    let last_quasi = &template.quasis[1];

                    // Check if both quasi parts contain no meaningful content.
                    // We use the cooked value to handle escape sequences properly.
                    // ESLint considers templates like `\n${foo}` as coercion because
                    // escape sequences that become whitespace don't add meaningful content.
                    // We also allow backslash characters in the cooked value because they
                    // typically come from line continuations (e.g., `\\\n${foo}`) which
                    // ESLint also treats as implicit coercion.
                    let first_is_empty_or_whitespace = first_quasi
                        .value
                        .cooked
                        .as_ref()
                        .is_some_and(|s| s.chars().all(|c| c.is_whitespace() || c == '\\'));
                    let last_is_empty_or_whitespace = last_quasi
                        .value
                        .cooked
                        .as_ref()
                        .is_some_and(|s| s.chars().all(|c| c.is_whitespace() || c == '\\'));

                    if first_is_empty_or_whitespace && last_is_empty_or_whitespace {
                        let expr = &template.expressions[0];
                        // Don't warn if the expression is already a string literal or String() call
                        if !is_string_literal_or_string_call(expr) {
                            let operand = get_operand_text(ctx, expr);
                            report_coercion(ctx, template.span(), CoercionKind::String, operand);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl NoImplicitCoercion {
    fn is_allowed(&self, flag: AllowedOperators) -> bool {
        self.allow.contains(flag)
    }
}

fn get_operand_text<'a>(ctx: &LintContext<'a>, expr: &Expression<'a>) -> &'a str {
    ctx.source_range(expr.without_parentheses().span())
}

fn is_numeric_literal(expr: &Expression) -> bool {
    matches!(
        expr.without_parentheses(),
        Expression::NumericLiteral(_) | Expression::BigIntLiteral(_)
    )
}

fn is_empty_string(expr: &Expression) -> bool {
    match expr.without_parentheses() {
        Expression::StringLiteral(lit) => lit.value.is_empty(),
        Expression::TemplateLiteral(template) => {
            template.expressions.is_empty()
                && template.quasis.len() == 1
                && template.quasis[0].value.raw.is_empty()
        }
        _ => false,
    }
}

/// Checks if an expression already evaluates to a number, meaning
/// implicit coercion patterns like `* 1` or `- 0` would be redundant.
fn is_already_numeric(expr: &Expression) -> bool {
    match expr.without_parentheses() {
        Expression::CallExpression(call) => {
            call.callee.is_specific_id("Number")
                || call.callee.is_specific_id("parseInt")
                || call.callee.is_specific_id("parseFloat")
        }
        Expression::NumericLiteral(_) | Expression::BigIntLiteral(_) => true,
        // Binary arithmetic operations always return numbers
        Expression::BinaryExpression(bin) => {
            matches!(
                bin.operator,
                BinaryOperator::Multiplication
                    | BinaryOperator::Division
                    | BinaryOperator::Remainder
                    | BinaryOperator::Subtraction
                    | BinaryOperator::Exponential
            )
        }
        // Unary +/- already return numbers
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation)
        }
        _ => false,
    }
}

fn is_string_literal_or_string_call(expr: &Expression) -> bool {
    match expr.without_parentheses() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::CallExpression(call) => call.callee.is_specific_id("String"),
        _ => false,
    }
}

fn is_indexof_call(expr: &Expression) -> bool {
    match expr.without_parentheses() {
        Expression::CallExpression(call) => {
            let callee = call.callee.without_parentheses();
            if let Expression::StaticMemberExpression(member) = callee {
                return matches!(member.property.name.as_str(), "indexOf" | "lastIndexOf");
            }
            if let Expression::ChainExpression(chain) = callee
                && let oxc_ast::ast::ChainElement::StaticMemberExpression(member) =
                    &chain.expression
            {
                return matches!(member.property.name.as_str(), "indexOf" | "lastIndexOf");
            }
            false
        }
        Expression::ChainExpression(chain) => {
            if let oxc_ast::ast::ChainElement::CallExpression(call) = &chain.expression
                && let Expression::StaticMemberExpression(member) =
                    call.callee.without_parentheses()
            {
                return matches!(member.property.name.as_str(), "indexOf" | "lastIndexOf");
            }
            false
        }
        _ => false,
    }
}

/// Checks if a TemplateLiteral node is part of a TaggedTemplateExpression
fn is_tagged_template(ctx: &LintContext, node: &AstNode) -> bool {
    matches!(ctx.nodes().parent_kind(node.id()), AstKind::TaggedTemplateExpression(_))
}

/// Checks if a BinaryExpression is nested inside another arithmetic BinaryExpression
/// that uses the numeric result (division, multiplication, etc.).
/// Used to distinguish `foo * 1` (standalone coercion) from `foo * 1 / 2` (part of math expression).
/// Note: Addition is NOT included because `foo * 1 + bar` might be using `* 1` to ensure
/// numeric addition instead of string concatenation.
fn is_part_of_larger_binary_expression(ctx: &LintContext, node: &AstNode) -> bool {
    if let AstKind::BinaryExpression(parent_bin) = ctx.nodes().parent_kind(node.id()) {
        matches!(
            parent_bin.operator,
            BinaryOperator::Multiplication
                | BinaryOperator::Division
                | BinaryOperator::Remainder
                | BinaryOperator::Subtraction
                | BinaryOperator::Exponential
        )
    } else {
        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Boolean(foo)", None),
        ("foo.indexOf(1) !== -1", None),
        ("foo.lastIndexOf(1) !== -1", None),
        ("Number(foo)", None),
        ("parseInt(foo)", None),
        ("parseFloat(foo)", None),
        ("String(foo)", None),
        ("!foo", None),
        ("~foo", None),
        ("-foo", None),
        ("+1234", None),
        ("-1234", None),
        ("- -1234", None),
        ("+Number(lol)", None),
        ("-parseFloat(lol)", None),
        ("2 * foo", None),
        ("1 * 1234", None),
        ("123 - 0", None),
        ("1 * Number(foo)", None),
        ("1 * parseInt(foo)", None),
        ("1 * parseFloat(foo)", None),
        ("Number(foo) * 1", None),
        ("Number(foo) - 0", None),
        ("parseInt(foo) * 1", None),
        ("parseFloat(foo) * 1", None),
        ("- -Number(foo)", None),
        ("1 * 1234 * 678 * Number(foo)", None),
        ("1 * 1234 * 678 * parseInt(foo)", None),
        ("(1 - 0) * parseInt(foo)", None),
        ("1234 * 1 * 678 * Number(foo)", None),
        ("1234 * 1 * Number(foo) * Number(bar)", None),
        ("1234 * 1 * Number(foo) * parseInt(bar)", None),
        ("1234 * 1 * Number(foo) * parseFloat(bar)", None),
        ("1234 * 1 * parseInt(foo) * parseFloat(bar)", None),
        ("1234 * 1 * parseInt(foo) * Number(bar)", None),
        ("1234 * 1 * parseFloat(foo) * Number(bar)", None),
        ("1234 * Number(foo) * 1 * Number(bar)", None),
        ("1234 * parseInt(foo) * 1 * Number(bar)", None),
        ("1234 * parseFloat(foo) * 1 * parseInt(bar)", None),
        ("1234 * parseFloat(foo) * 1 * Number(bar)", None),
        ("(- -1234) * (parseFloat(foo) - 0) * (Number(bar) - 0)", None),
        ("1234*foo*1", None),
        ("1234*1*foo", None),
        ("1234*bar*1*foo", None),
        ("1234*1*foo*bar", None),
        ("1234*1*foo*Number(bar)", None),
        ("1234*1*Number(foo)*bar", None),
        ("1234*1*parseInt(foo)*bar", None),
        ("0 + foo", None),
        ("~foo.bar()", None),
        ("foo + 'bar'", None),
        ("foo + `${bar}`", None),
        ("!!foo", Some(serde_json::json!([{ "boolean": false }]))),
        ("~foo.indexOf(1)", Some(serde_json::json!([{ "boolean": false }]))),
        ("~foo.lastIndexOf(1)", Some(serde_json::json!([{ "boolean": false }]))),
        ("+foo", Some(serde_json::json!([{ "number": false }]))),
        ("-(-foo)", Some(serde_json::json!([{ "number": false }]))),
        ("foo - 0", Some(serde_json::json!([{ "number": false }]))),
        ("1*foo", Some(serde_json::json!([{ "number": false }]))),
        (r#"""+foo"#, Some(serde_json::json!([{ "string": false }]))),
        (r#"foo += """#, Some(serde_json::json!([{ "string": false }]))),
        ("var a = !!foo", Some(serde_json::json!([{ "boolean": true, "allow": ["!!"] }]))),
        ("var a = ~foo.indexOf(1)", Some(serde_json::json!([{ "boolean": true, "allow": ["~"] }]))),
        (
            "var a = ~foo.lastIndexOf(1)",
            Some(serde_json::json!([{ "boolean": true, "allow": ["~"] }])),
        ),
        ("var a = ~foo", Some(serde_json::json!([{ "boolean": true }]))),
        ("var a = 1 * foo", Some(serde_json::json!([{ "boolean": true, "allow": ["*"] }]))),
        ("- -foo", Some(serde_json::json!([{ "number": true, "allow": ["- -"] }]))),
        ("foo - 0", Some(serde_json::json!([{ "number": true, "allow": ["-"] }]))),
        ("var a = +foo", Some(serde_json::json!([{ "boolean": true, "allow": ["+"] }]))),
        (
            r#"var a = "" + foo"#,
            Some(serde_json::json!([{ "boolean": true, "string": true, "allow": ["+"] }])),
        ),
        ("'' + 'foo'", None),
        ("`` + 'foo'", None),
        ("'' + `${foo}`", None),
        ("'foo' + ''", None),
        ("'foo' + ``", None),
        ("`${foo}` + ''", None),
        ("foo += 'bar'", None),
        ("foo += `${bar}`", None),
        ("`a${foo}`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        ("`${foo}b`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        ("`${foo}${bar}`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        ("tag`${foo}`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        ("`${foo}`", None),
        ("`${foo}`", Some(serde_json::json!([{}]))),
        ("`${foo}`", Some(serde_json::json!([{ "disallowTemplateShorthand": false }]))),
        ("+42", None),
        ("'' + String(foo)", None),
        ("String(foo) + ''", None),
        ("`` + String(foo)", None),
        ("String(foo) + ``", None),
        ("`${'foo'}`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        ("`${`foo`}`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        ("`${String(foo)}`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        ("console.log(Math.PI * 1/4)", None),
        ("a * 1 / 2", None),
        ("a * 1 / b", None),
    ];

    let fail = vec![
        ("!!foo", None),
        ("!!(foo + bar)", None),
        ("!!(foo + bar); var Boolean = null", None),
        ("!!(foo + bar)", None),
        ("~foo.indexOf(1)", None),
        ("~foo.bar.indexOf(2)", None),
        ("~foo.lastIndexOf(1)", None),
        ("~foo.bar.lastIndexOf(2)", None),
        ("~foo?.lastIndexOf(1)", None),
        ("+foo", None),
        ("-(-foo)", None),
        ("+foo.bar", None),
        ("1*foo", None),
        ("foo*1", None),
        ("1*foo.bar", None),
        ("foo.bar-0", None),
        (r#"""+foo"#, None),
        ("``+foo", None),
        (r#"foo+"""#, None),
        ("foo+``", None),
        (r#"""+foo.bar"#, None),
        ("``+foo.bar", None),
        (r#"foo.bar+"""#, None),
        ("foo.bar+``", None),
        ("`${foo}`", Some(serde_json::json!([{ "disallowTemplateShorthand": true }]))),
        (
            r"`\\
			${foo}`",
            Some(serde_json::json!([{ "disallowTemplateShorthand": true }])),
        ),
        (
            r"`${foo}\\
			`",
            Some(serde_json::json!([{ "disallowTemplateShorthand": true }])),
        ),
        (r#"foo += """#, None),
        ("foo += ``", None),
        ("var a = !!foo", Some(serde_json::json!([{ "boolean": true, "allow": ["~"] }]))),
        (
            "var a = ~foo.indexOf(1)",
            Some(serde_json::json!([{ "boolean": true, "allow": ["!!"] }])),
        ),
        ("var a = 1 * foo", Some(serde_json::json!([{ "boolean": true, "allow": ["+"] }]))),
        ("var a = +foo", Some(serde_json::json!([{ "boolean": true, "allow": ["*"] }]))),
        (r#"var a = "" + foo"#, Some(serde_json::json!([{ "boolean": true, "allow": ["*"] }]))),
        ("var a = `` + foo", Some(serde_json::json!([{ "boolean": true, "allow": ["*"] }]))),
        ("typeof+foo", None),
        ("typeof +foo", None),
        ("let x ='' + 1n;", None),
        ("~foo?.indexOf(1)", None),
        ("~(foo?.indexOf)(1)", None),
        ("1 * a / 2", None),
        ("(a * 1) / 2", None),
        ("a * 1 / (b * 1)", None),
        ("a * 1 + 2", None),
    ];

    let fix = vec![
        ("!!foo", "Boolean(foo)", None),
        ("!!(foo + bar)", "Boolean(foo + bar)", None),
        (
            "var a = !!foo",
            "var a = Boolean(foo)",
            Some(serde_json::json!([{ "boolean": true, "allow": ["~"] }])),
        ),
    ];
    Tester::new(NoImplicitCoercion::NAME, NoImplicitCoercion::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
