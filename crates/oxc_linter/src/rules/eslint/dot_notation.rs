use std::ops::Deref;

use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{ComputedMemberExpression, Expression, StaticMemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::is_identifier_name;
use schemars::JsonSchema;

use crate::{AstNode, context::LintContext, rule::Rule};

/// ES3 reserved keywords used by ESLint's dot-notation rule for allowKeywords option.
/// This list matches ESLint's lib/rules/utils/keywords.js
const ES3_KEYWORDS: &[&str] = &[
    "abstract",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "double",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "function",
    "goto",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "volatile",
    "while",
    "with",
];

fn is_es3_keyword(name: &str) -> bool {
    ES3_KEYWORDS.binary_search(&name).is_ok()
}

fn use_dot_notation_diagnostic(property: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("[{property}] is better written in dot notation"))
        .with_help(format!("Use `.{property}` instead of `[\"{property}\"]`"))
        .with_label(span)
}

fn use_bracket_notation_diagnostic(property: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(".{property} is a reserved word"))
        .with_help(format!("Use `[\"{property}\"]` instead of `.{property}`"))
        .with_label(span)
}

#[derive(Debug, Clone, Default)]
pub struct DotNotation(Box<DotNotationConfig>);

impl Deref for DotNotation {
    type Target = DotNotationConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DotNotationConfig {
    /// A regex pattern that allows bracket notation for property names matching the pattern.
    #[schemars(with = "Option<String>")]
    allow_pattern: Option<Regex>,
    /// Set to `false` to require bracket notation for reserved words (ES3 compatibility).
    allow_keywords: bool,
}

impl Default for DotNotationConfig {
    fn default() -> Self {
        Self { allow_pattern: None, allow_keywords: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires the use of dot notation whenever possible, discouraging the use
    /// of square-bracket notation to access properties.
    ///
    /// ### Why is this bad?
    ///
    /// Dot notation is more concise and easier to read. Square-bracket notation
    /// should only be used when the property name is dynamic or contains special
    /// characters that would make dot notation invalid.
    ///
    /// ### Options
    ///
    /// - `allowKeywords` (default: `true`): Set to `false` to require bracket notation
    ///   for reserved words (ECMAScript 3 compatibility mode).
    /// - `allowPattern`: A regex pattern that allows bracket notation for property
    ///   names matching the pattern.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var x = foo["bar"];
    /// var y = obj["helloWorld"]; // valid identifier should use dot notation
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var x = foo.bar;
    /// var y = foo[bar]; // dynamic property
    /// var z = foo["hello-world"]; // invalid identifier character (hyphen)
    /// var w = obj["hello world"]; // invalid identifier (contains space)
    /// ```
    DotNotation,
    eslint,
    style,
    fix,
    config = DotNotationConfig,
);

impl Rule for DotNotation {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);
        let allow_keywords = config
            .and_then(|c| c.get("allowKeywords"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        // Note: Invalid regex patterns are silently ignored (pattern becomes None).
        // This matches the behavior of other oxc rules with regex config options.
        let allow_pattern = config
            .and_then(|c| c.get("allowPattern"))
            .and_then(serde_json::Value::as_str)
            .and_then(|s| Regex::new(s).ok());

        Self(Box::new(DotNotationConfig { allow_pattern, allow_keywords }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ComputedMemberExpression(expr) => {
                self.check_computed_member_expression(expr, ctx);
            }
            AstKind::StaticMemberExpression(expr) => {
                self.check_static_member_expression(expr, ctx);
            }
            _ => {}
        }
    }
}

impl DotNotation {
    fn check_computed_member_expression<'a>(
        &self,
        expr: &ComputedMemberExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        // Unwrap parenthesized expressions to get the actual expression
        let inner_expr = unwrap_parenthesized(&expr.expression);

        let property_name = match inner_expr {
            Expression::StringLiteral(lit) => Some(lit.value.as_str()),
            Expression::TemplateLiteral(lit) if lit.expressions.is_empty() => {
                // Use cooked value (interpreted) to match ESLint behavior.
                // cooked is None for invalid escape sequences in tagged templates.
                lit.quasis.first().and_then(|q| q.value.cooked.as_ref().map(|s| s.as_str()))
            }
            Expression::NullLiteral(_) => Some("null"),
            Expression::BooleanLiteral(lit) => Some(if lit.value { "true" } else { "false" }),
            _ => None,
        };

        let Some(property_name) = property_name else {
            return;
        };

        // Check if property name is a valid identifier
        if !is_identifier_name(property_name) {
            return;
        }

        // Check if it's an ES3 keyword and allowKeywords is false
        if !self.allow_keywords && is_es3_keyword(property_name) {
            return;
        }

        // Check if pattern allows this property
        if let Some(ref regex) = self.allow_pattern
            && regex.is_match(property_name)
        {
            return;
        }

        // Check for comments inside the bracket notation - don't auto-fix if present
        // to avoid destroying user comments (matches ESLint behavior)
        let bracket_span = Span::new(expr.object.span().end, expr.span.end);
        if ctx.has_comments_between(bracket_span) {
            ctx.diagnostic(use_dot_notation_diagnostic(property_name, expr.span));
            return;
        }

        // Report: should use dot notation
        ctx.diagnostic_with_fix(use_dot_notation_diagnostic(property_name, expr.span), |fixer| {
            // Find the bracket position to preserve whitespace between object and bracket
            let source = ctx.source_text();
            let object_end = expr.object.span().end as usize;
            let expr_end = expr.span.end as usize;

            // Find the '[' or '?.[' position after the object
            let text_between = &source[object_end..expr_end];
            let bracket_offset = if expr.optional {
                // For optional chaining: obj?.['prop']
                // Find '?.' first, then '['
                text_between.find("?.[").map(|p| p + 2) // Skip '?.' to point at '['
            } else {
                text_between.find('[')
            };

            let Some(bracket_offset) = bracket_offset else {
                // Fallback: replace entire expression
                let object_text = ctx.source_range(expr.object.span());
                let needs_space_before =
                    !expr.optional && needs_space_before_dot(&expr.object, ctx);
                let needs_space_after = needs_space_after_property(expr.span, property_name, ctx);
                let operator = if expr.optional { "?." } else { "." };
                let space_before = if needs_space_before { " " } else { "" };
                let space_after = if needs_space_after { " " } else { "" };
                let fixed =
                    format!("{object_text}{space_before}{operator}{property_name}{space_after}");
                return fixer.replace(expr.span, fixed);
            };

            // Calculate positions
            let bracket_start = object_end + bracket_offset;

            // Check if we need space before dot (for numeric literals)
            // Only when there's no whitespace between object and bracket
            let has_whitespace_before =
                text_between[..bracket_offset].chars().any(char::is_whitespace);
            let needs_space_before = !has_whitespace_before
                && !expr.optional
                && needs_space_before_dot(&expr.object, ctx);
            let needs_space_after = needs_space_after_property(expr.span, property_name, ctx);

            let operator = if expr.optional { "" } else { "." }; // '?.' already in source for optional
            let space_before = if needs_space_before { " " } else { "" };
            let space_after = if needs_space_after { " " } else { "" };

            // Replace from '[' to end of expression with '.property'
            #[expect(clippy::cast_possible_truncation)]
            let replacement_span = Span::new(bracket_start as u32, expr.span.end);
            let fixed = format!("{space_before}{operator}{property_name}{space_after}");
            fixer.replace(replacement_span, fixed)
        });
    }

    fn check_static_member_expression<'a>(
        &self,
        expr: &StaticMemberExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        // Only check if allowKeywords is false
        if self.allow_keywords {
            return;
        }

        let property_name = expr.property.name.as_str();

        // Report if the property is an ES3 keyword
        if is_es3_keyword(property_name) {
            // Check for comments between object and property - don't auto-fix if present
            let member_span = Span::new(expr.object.span().end, expr.span.end);
            if ctx.has_comments_between(member_span) {
                ctx.diagnostic(use_bracket_notation_diagnostic(property_name, expr.span));
                return;
            }

            ctx.diagnostic_with_fix(
                use_bracket_notation_diagnostic(property_name, expr.span),
                |fixer| {
                    let object_text = ctx.source_range(expr.object.span());
                    let operator = if expr.optional { "?." } else { "" };

                    let fixed = format!("{object_text}{operator}[\"{property_name}\"]");
                    fixer.replace(expr.span, fixed)
                },
            );
        }
    }
}

/// Recursively unwrap parenthesized expressions to get the inner expression.
fn unwrap_parenthesized<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(paren) => unwrap_parenthesized(&paren.expression),
        _ => expr,
    }
}

/// Determine if a space is needed after the property name to avoid creating invalid syntax.
/// For example, `foo['bar']instanceof baz` should become `foo.bar instanceof baz`.
fn needs_space_after_property(span: Span, _property_name: &str, ctx: &LintContext<'_>) -> bool {
    let source = ctx.source_text();
    let end = span.end as usize;

    // Check what comes after the member expression
    if end >= source.len() {
        return false;
    }

    let rest = &source[end..];

    // If the next character starts an identifier-like token,
    // we need a space to avoid merging tokens
    let next_char = rest.chars().next();
    matches!(next_char, Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$')
}

/// Determine if a space is needed before the dot to avoid ambiguity with numeric literals.
/// For example, `5.prop` would be parsed as `5.` followed by `prop`, so we need `5 .prop`.
fn needs_space_before_dot<'a>(object: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let Expression::NumericLiteral(lit) = object else {
        return false;
    };

    let text = ctx.source_range(lit.span());

    // If the number already has a decimal point, we don't need a space
    if text.contains('.') {
        return false;
    }

    // Check if it's an octal or other special format that doesn't need a space
    // Binary (0b), Hex (0x), and Octal (0o) don't need space
    if text.starts_with("0b")
        || text.starts_with("0B")
        || text.starts_with("0x")
        || text.starts_with("0X")
        || text.starts_with("0o")
        || text.starts_with("0O")
    {
        return false;
    }

    // Legacy octal (starts with 0 followed by digits 0-7 only)
    if text.starts_with('0') && text.len() > 1 {
        let rest = &text[1..];
        // Check if it's purely octal digits (0-7)
        if rest.chars().all(|c| c.is_ascii_digit() && c < '8') {
            return false;
        }
    }

    // Integer literals need space to avoid parsing as decimal
    // But octal-like numbers that contain 8 or 9 (like 08, 09, 018) need a space
    // because they're treated as decimal
    if text.starts_with('0') && text.len() > 1 && text[1..].contains(['8', '9']) {
        return true;
    }

    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("a.b;", None),
        ("a.b.c;", None),
        ("a['12'];", None),
        ("a[b];", None),
        ("a[0];", None),
        ("a.b.c;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.arguments;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.let;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.yield;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.eval;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a[0];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['while'];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['true'];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['null'];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a[true];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a[null];", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a.true;", Some(serde_json::json!([{ "allowKeywords": true }]))),
        ("a.null;", Some(serde_json::json!([{ "allowKeywords": true }]))),
        ("a['snake_case'];", Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }]))),
        (
            "a['lots_of_snake_case'];",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        ("a[`time${range}`];", None), // { "ecmaVersion": 6 },
        ("a[`while`];", Some(serde_json::json!([{ "allowKeywords": false }]))), // { "ecmaVersion": 6 },
        ("a[`time range`];", None), // { "ecmaVersion": 6 },
        ("a.true;", None),
        ("a.null;", None),
        ("a[undefined];", None),
        ("a[void 0];", None),
        ("a[b()];", None),
        ("a[/(?<zero>0)/];", None), // { "ecmaVersion": 2018 },
        ("class C { foo() { this['#a'] } }", None), // { "ecmaVersion": 2022 },
        (
            "class C { #in; foo() { this.#in; } }",
            Some(serde_json::json!([{ "allowKeywords": false }])),
        ), // { "ecmaVersion": 2022 }
        // Empty template literal - empty string is not a valid identifier
        ("a[``];", None),
        // Invalid identifiers (whitespace in property name)
        ("a[`  prop  `];", None),
    ];

    let fail = vec![
        ("a.true;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['true'];", None),
        ("a[`time`];", None), // { "ecmaVersion": 6 },
        ("a[null];", None),
        ("a[true];", None),
        ("a[false];", None),
        ("a['b'];", None),
        ("a.b['c'];", None),
        ("a['_dangle'];", Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }]))),
        ("a['SHOUT_CASE'];", Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }]))),
        // Multiline SHOUT_CASE test
        (
            "a\n ['SHOUT_CASE'];",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        // Multiple errors in same expression
        (
            "getResource()\n .then(function(){})\n [\"catch\"](function(){})\n .then(function(){})\n [\"catch\"](function(){});",
            None,
        ),
        // foo.while without comment
        ("foo.while;", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("foo[ /* comment */ 'bar' ]", None),
        ("foo[ 'bar' /* comment */ ]", None),
        ("foo[    'bar'    ];", None),
        ("foo. /* comment */ while", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("foo[('bar')]", None),
        ("foo[(null)]", None),
        ("(foo)['bar']", None),
        ("1['toString']", None),
        ("foo['bar']instanceof baz", None),
        ("let.if()", Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("5['prop']", None),
        ("-5['prop']", None),
        ("01['prop']", None),
        ("01234567['prop']", None),
        ("08['prop']", None),
        ("090['prop']", None),
        ("018['prop']", None),
        ("5_000['prop']", None),       // { "ecmaVersion": 2021 },
        ("5_000_00['prop']", None),    // { "ecmaVersion": 2021 },
        ("5.000_000['prop']", None),   // { "ecmaVersion": 2021 },
        ("0b1010_1010['prop']", None), // { "ecmaVersion": 2021 },
        ("obj?.['prop']", None),       // { "ecmaVersion": 2020 },
        ("0?.['prop']", None),         // { "ecmaVersion": 2020 },
        ("obj?.true", Some(serde_json::json!([{ "allowKeywords": false }]))), // { "ecmaVersion": 2020 },
        ("let?.true", Some(serde_json::json!([{ "allowKeywords": false }]))), // { "ecmaVersion": 2020 }
        // Hex and octal literals
        ("0xFF['prop']", None),
        ("0o77['prop']", None),
        ("0O77['prop']", None),
        // BigInt literals
        ("5n['prop']", None),
        // Unicode identifiers ($ and _ are valid identifier starts)
        ("a['$']", None),
        ("a['_']", None),
        ("a['$$']", None),
        ("a['__proto__']", None),
    ];

    let fix = vec![
        ("a.true;", r#"a["true"];"#, Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("a['true'];", "a.true;", None),
        ("a[`time`];", "a.time;", None),
        ("a[null];", "a.null;", None),
        ("a[true];", "a.true;", None),
        ("a[false];", "a.false;", None),
        ("a['b'];", "a.b;", None),
        ("a.b['c'];", "a.b.c;", None),
        (
            "a['_dangle'];",
            "a._dangle;",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        (
            "a['SHOUT_CASE'];",
            "a.SHOUT_CASE;",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        // Multiline SHOUT_CASE fix
        (
            "a\n ['SHOUT_CASE'];",
            "a\n .SHOUT_CASE;",
            Some(serde_json::json!([{ "allowPattern": "^[a-z]+(_[a-z]+)+$" }])),
        ),
        // Multiple errors in same expression fix
        (
            "getResource()\n .then(function(){})\n [\"catch\"](function(){})\n .then(function(){})\n [\"catch\"](function(){});",
            "getResource()\n .then(function(){})\n .catch(function(){})\n .then(function(){})\n .catch(function(){});",
            None,
        ),
        // foo.while fix
        ("foo.while;", r#"foo["while"];"#, Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("foo[    'bar'    ];", "foo.bar;", None),
        ("foo[('bar')]", "foo.bar", None),
        ("foo[(null)]", "foo.null", None),
        ("(foo)['bar']", "(foo).bar", None),
        ("1['toString']", "1 .toString", None),
        ("foo['bar']instanceof baz", "foo.bar instanceof baz", None),
        ("5['prop']", "5 .prop", None),
        ("-5['prop']", "-5 .prop", None),
        ("01['prop']", "01.prop", None),
        ("01234567['prop']", "01234567.prop", None),
        ("08['prop']", "08 .prop", None),
        ("090['prop']", "090 .prop", None),
        ("018['prop']", "018 .prop", None),
        ("5_000['prop']", "5_000 .prop", None),
        ("5_000_00['prop']", "5_000_00 .prop", None),
        ("5.000_000['prop']", "5.000_000.prop", None),
        ("0b1010_1010['prop']", "0b1010_1010.prop", None),
        ("obj?.['prop']", "obj?.prop", None),
        ("0?.['prop']", "0?.prop", None),
        ("obj?.true", r#"obj?.["true"]"#, Some(serde_json::json!([{ "allowKeywords": false }]))),
        ("let?.true", r#"let?.["true"]"#, Some(serde_json::json!([{ "allowKeywords": false }]))),
        // Hex and octal literals (no space needed)
        ("0xFF['prop']", "0xFF.prop", None),
        ("0o77['prop']", "0o77.prop", None),
        ("0O77['prop']", "0O77.prop", None),
        // BigInt literals (no space needed - 5n.prop is valid syntax)
        ("5n['prop']", "5n.prop", None),
        // Unicode identifiers
        ("a['$']", "a.$", None),
        ("a['_']", "a._", None),
        ("a['$$']", "a.$$", None),
        ("a['__proto__']", "a.__proto__", None),
    ];
    Tester::new(DotNotation::NAME, DotNotation::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

/// Verify that ES3_KEYWORDS is sorted (required for binary_search to work correctly)
#[test]
fn es3_keywords_is_sorted() {
    for window in ES3_KEYWORDS.windows(2) {
        assert!(
            window[0] < window[1],
            "ES3_KEYWORDS must be sorted for binary_search: '{}' should come before '{}'",
            window[0],
            window[1]
        );
    }
}
