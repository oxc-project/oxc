use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_allocator::Vec;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, IdentifierReference, RegExpFlags, TemplateLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};

use crate::{
    AstNode,
    ast_util::get_declaration_of_variable,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_regexp_callee,
};

fn require_unicode_regexp_diagnostic(
    span: Span,
    require_flag: Option<&RequireFlag>,
) -> OxcDiagnostic {
    let (msg, help_msg) = if matches!(require_flag, Some(RequireFlag::V)) {
        ("Use the 'v' flag.", "Add the 'v' flag.")
    } else {
        ("Use the 'u' flag.", "Add the 'u' flag.")
    };

    OxcDiagnostic::warn(msg)
        .with_note("The 'u' and 'v' flags enable Unicode-aware regular expression behavior and stricter pattern parsing.")
        .with_help(help_msg)
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct RequireUnicodeRegexpConfig {
    /// The `u` flag may be preferred in environments that do not support the `v` flag.
    ///
    /// Examples of **incorrect** code for this rule with the `{ "requireFlag": "u" }` option:
    /// ```js
    /// const fooEmpty = /foo/;
    /// const fooEmptyRegexp = new RegExp('foo');
    /// const foo = /foo/v;
    /// const fooRegexp = new RegExp('foo', 'v');
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "requireFlag": "u" }` option:
    /// ```js
    /// const foo = /foo/u;
    /// const fooRegexp = new RegExp('foo', 'u');
    /// ```
    ///
    /// The `v` flag may be a better choice when it is supported because it has more features than the `u` flag (e.g., the ability to test Unicode properties of strings).
    /// It does have a stricter syntax, however (e.g., the need to escape certain characters within character classes).
    ///
    /// Examples of **incorrect** code for this rule with the `{ "requireFlag": "v" }` option:
    /// ```js
    /// const fooEmpty = /foo/;
    /// const fooEmptyRegexp = new RegExp('foo');
    /// const foo = /foo/u;
    /// const fooRegexp = new RegExp('foo', 'u');
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "requireFlag": "v" }` option:
    /// ```js
    /// const foo = /foo/v;
    /// const fooRegexp = new RegExp('foo', 'v');
    /// ```
    require_flag: Option<RequireFlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum RequireFlag {
    U,
    V,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RequireUnicodeRegexp(RequireUnicodeRegexpConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the use of `u` or `v` flag on regular expressions.
    ///
    /// ### Why is this bad?
    ///
    /// RegExp `u` flag has two effects:
    ///  1. Make the regular expression handling UTF-16 surrogate pairs correctly.
    /// ```js
    /// /^[👍]$/.test("👍") //→ false
    /// /^[👍]$/u.test("👍") //→ true
    /// ```
    ///
    /// 2. Make the regular expression throwing syntax errors early as disabling [Annex B extensions](https://262.ecma-international.org/6.0/#sec-regular-expressions-patterns).
    /// Because of historical reason, JavaScript regular expressions are tolerant of syntax errors.
    /// For example, `/\w{1, 2/` is a syntax error, but JavaScript doesn’t throw the error. It matches strings such as `"a{1, 2"` instead. Such a recovering logic is defined in Annex B.
    ///
    /// The RegExp `v` flag, introduced in ECMAScript 2024, is a superset of the `u` flag, and offers two more features:
    /// 1. Unicode properties of strings
    /// ```js
    /// const re = /^\p{RGI_Emoji}$/v;
    ///
    /// // Match an emoji that consists of just 1 code point:
    /// re.test('⚽'); // '\u26BD'
    /// // → true ✅
    ///
    /// // Match an emoji that consists of multiple code points:
    /// re.test('👨🏾‍⚕️'); // '\u{1F468}\u{1F3FE}\u200D\u2695\uFE0F'
    /// // → true ✅
    /// ```
    ///
    /// 2. Set notation
    /// It allows for set operations between character classes:
    /// ```js
    /// const re = /[\p{White_Space}&&\p{ASCII}]/v;
    /// re.test('\n'); // → true
    /// re.test('\u2028'); // → false
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const a = /aaa/
    /// const b = /bbb/gi
    /// const c = new RegExp("ccc")
    /// const d = new RegExp("ddd", "gi")
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = /aaa/u
    /// const b = /bbb/giu
    /// const c = new RegExp("ccc", "u")
    /// const d = new RegExp("ddd", "giu")
    ///
    /// const e = /aaa/v
    /// const f = /bbb/giv
    /// const g = new RegExp("ccc", "v")
    /// const h = new RegExp("ddd", "gv")
    ///
    /// // This rule ignores RegExp calls if the flags could not be evaluated to a static value.
    /// function i(flags) {
    ///     return new RegExp("eee", flags)
    /// }
    /// ```
    RequireUnicodeRegexp,
    eslint,
    pedantic,
    pending,
    config = RequireUnicodeRegexp,
    version = "1.63.0",
);

impl Rule for RequireUnicodeRegexp {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::RegExpLiteral(regexp_lit) => {
                let flags = regexp_lit.regex.flags;

                let is_missing_flag = self.check_flags(flags);

                if is_missing_flag {
                    ctx.diagnostic(require_unicode_regexp_diagnostic(
                        node.span(),
                        self.0.require_flag.as_ref(),
                    ));
                }
            }
            AstKind::NewExpression(new_expr) if is_regexp_callee(&new_expr.callee, ctx) => {
                if let Some(flags) = extract_regex_flags(&new_expr.arguments, ctx)
                    && self.check_flags(flags)
                {
                    ctx.diagnostic(require_unicode_regexp_diagnostic(
                        node.span(),
                        self.0.require_flag.as_ref(),
                    ));
                }
            }
            AstKind::CallExpression(call_expr) if is_regexp_callee(&call_expr.callee, ctx) => {
                if let Some(flags) = extract_regex_flags(&call_expr.arguments, ctx)
                    && self.check_flags(flags)
                {
                    ctx.diagnostic(require_unicode_regexp_diagnostic(
                        node.span(),
                        self.0.require_flag.as_ref(),
                    ));
                }
            }
            _ => {}
        }
    }
}

impl RequireUnicodeRegexp {
    fn check_flags(&self, flags: RegExpFlags) -> bool {
        match &self.0.require_flag {
            Some(flag) => {
                let regexp_flag = match flag {
                    RequireFlag::U => RegExpFlags::U,
                    RequireFlag::V => RegExpFlags::V,
                };

                !flags.contains(regexp_flag)
            }
            None => !flags.contains(RegExpFlags::U) && !flags.contains(RegExpFlags::V),
        }
    }
}

fn extract_regex_flags<'a>(
    args: &'a Vec<'a, Argument<'a>>,
    ctx: &LintContext<'a>,
) -> Option<RegExpFlags> {
    if args.iter().any(oxc_ast::ast::Argument::is_spread) {
        return None;
    }
    if args.len() <= 1 {
        return Some(RegExpFlags::empty());
    }

    let flags_arg = args[1].as_expression()?.get_inner_expression();

    resolve_flags(flags_arg, ctx, true, 0)
}

fn parse_flags(flags_text: &str) -> RegExpFlags {
    flags_text.chars().filter_map(|ch| RegExpFlags::try_from(ch).ok()).collect()
}

const MAX_RESOLVE_DEPTH: usize = 8;

fn resolve_flags<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'a>,
    follow_identifier: bool,
    depth: usize,
) -> Option<RegExpFlags> {
    if depth > MAX_RESOLVE_DEPTH {
        return None;
    }

    let next_depth = depth + 1;
    let expr = expr.get_inner_expression();
    if follow_identifier && let Expression::Identifier(ident) = expr {
        return resolve_flags(resolve_const_initializer(ident, ctx)?, ctx, true, next_depth);
    }

    match expr {
        Expression::StringLiteral(lit) => Some(parse_flags(lit.value.as_str())),
        Expression::TemplateLiteral(template) => resolve_template_flags(template, ctx, next_depth),
        Expression::BooleanLiteral(lit) => {
            Some(if lit.value { RegExpFlags::U } else { RegExpFlags::empty() })
        }
        Expression::NullLiteral(_) => Some(RegExpFlags::U),
        Expression::NumericLiteral(_) => Some(RegExpFlags::empty()),
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = resolve_flags(&binary.left, ctx, true, next_depth)?;
            let right = resolve_flags(&binary.right, ctx, true, next_depth)?;
            Some(left | right)
        }
        Expression::ComputedMemberExpression(member) => {
            let object = resolve_static_string(&member.object, ctx)?;
            let index = resolve_static_index(&member.expression)?;
            let ch = object.chars().nth(index)?;
            Some(RegExpFlags::try_from(ch).unwrap_or_else(|_| RegExpFlags::empty()))
        }
        Expression::SequenceExpression(sequence) => {
            resolve_flags(sequence.expressions.last()?, ctx, true, next_depth)
        }
        Expression::AssignmentExpression(assignment)
            if assignment.operator == AssignmentOperator::Assign =>
        {
            resolve_flags(&assignment.right, ctx, true, next_depth)
        }
        _ => None,
    }
}

fn resolve_template_flags<'a>(
    template: &'a TemplateLiteral<'a>,
    ctx: &LintContext<'a>,
    depth: usize,
) -> Option<RegExpFlags> {
    let mut flags = RegExpFlags::empty();

    for (index, expression) in template.expressions.iter().enumerate() {
        flags |= parse_flags(template.quasis.get(index)?.value.cooked?.as_str());
        flags |= resolve_flags(expression, ctx, true, depth)?;
    }

    flags |= parse_flags(template.quasis.last()?.value.cooked?.as_str());
    Some(flags)
}

fn resolve_static_string<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>) -> Option<&'a str> {
    match expr.get_inner_expression() {
        Expression::StringLiteral(lit) => Some(lit.value.as_str()),
        Expression::TemplateLiteral(template) => Some(template.single_quasi()?.as_str()),
        Expression::Identifier(ident) => {
            match resolve_const_initializer(ident, ctx)?.get_inner_expression() {
                Expression::StringLiteral(lit) => Some(lit.value.as_str()),
                Expression::TemplateLiteral(template) => Some(template.single_quasi()?.as_str()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn resolve_const_initializer<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let declaration = get_declaration_of_variable(ident, ctx.semantic())?;
    let AstKind::VariableDeclarator(decl) = declaration.kind() else { return None };

    if !decl.kind.is_const() || !decl.id.is_binding_identifier() {
        return None;
    }

    decl.init.as_ref()
}

fn resolve_static_index(expr: &Expression) -> Option<usize> {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(lit)
            if lit.value.is_finite() && lit.value >= 0.0 && lit.value.fract() == 0.0 =>
        {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "value is checked to be finite non-negative and integral"
            )]
            Some(lit.value as usize)
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("/foo/u", None),
        ("/foo/gimuy", None),
        ("RegExp('', 'u')", None),
        ("RegExp('', `u`)", None),
        ("new RegExp('', 'u')", None),
        ("RegExp('', 'gimuy')", None),
        ("RegExp('', `gimuy`)", None),
        ("RegExp(...patternAndFlags)", None),
        ("new RegExp('', 'gimuy')", None),
        ("const flags = 'u'; new RegExp('', flags)", None),
        ("const flags = 'g'; new RegExp('', flags + 'u')", None),
        ("const flags = 'gimu'; new RegExp('foo', flags[3])", None),
        ("const flags = flags; new RegExp('foo', flags)", None),
        ("const flags = `${flags}`; new RegExp('foo', flags)", None),
        ("const flags = other; const other = flags; new RegExp('foo', flags)", None),
        ("new RegExp('', flags)", None),
        ("function f(flags) { return new RegExp('', flags) }", None),
        ("function f(RegExp) { return new RegExp('foo') }", None),
        ("function f(patternAndFlags) { return new RegExp(...patternAndFlags) }", None),
        ("new globalThis.RegExp('foo', 'u')", None), // { "ecmaVersion": 2020 },
        ("globalThis.RegExp('foo', 'u')", None),     // { "ecmaVersion": 2020 },
        ("const flags = 'u'; new globalThis.RegExp('', flags)", None), // { "ecmaVersion": 2020 },
        ("const flags = 'g'; new globalThis.RegExp('', flags + 'u')", None), // { "ecmaVersion": 2020 },
        ("const flags = 'gimu'; new globalThis.RegExp('foo', flags[3])", None), // { "ecmaVersion": 2020 },
        ("class C { #RegExp; foo() { new globalThis.#RegExp('foo') } }", None), // { "ecmaVersion": 2022 },
        ("/foo/u", Some(serde_json::json!([{ "requireFlag": "u" }]))),
        ("new RegExp('foo', 'u')", Some(serde_json::json!([{ "requireFlag": "u" }]))),
        ("/foo/v", None),                  // { "ecmaVersion": 2024 },
        ("/foo/gimvy", None),              // { "ecmaVersion": 2024 },
        ("RegExp('', 'v')", None),         // { "ecmaVersion": 2024 },
        ("RegExp('', `v`)", None),         // { "ecmaVersion": 2024 },
        ("new RegExp('', 'v')", None),     // { "ecmaVersion": 2024 },
        ("RegExp('', 'gimvy')", None),     // { "ecmaVersion": 2024 },
        ("RegExp('', `gimvy`)", None),     // { "ecmaVersion": 2024 },
        ("new RegExp('', 'gimvy')", None), // { "ecmaVersion": 2024 },
        ("const flags = 'v'; new RegExp('', flags)", None), // { "ecmaVersion": 2024 },
        ("const flags = 'g'; new RegExp('', flags + 'v')", None), // { "ecmaVersion": 2024 },
        ("const flags = 'gimv'; new RegExp('foo', flags[3])", None), // { "ecmaVersion": 2024 },
        ("/foo/v", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        ("new RegExp('foo', 'v')", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 }
    ];

    let fail = vec![
        (r"/\a/", None),
        ("/foo/", None),
        ("/foo/gimy", None),
        ("RegExp()", None),
        ("RegExp('foo')", None),
        (r"RegExp('\\a')", None),
        ("RegExp('foo', '')", None),
        ("RegExp('foo', 'gimy')", None),
        ("RegExp('foo', `gimy`)", None),
        ("new RegExp('foo')", None),
        ("new RegExp('foo',)", None), // { "ecmaVersion": 2017, },
        ("new RegExp('foo', false)", None),
        ("new RegExp('foo', 1)", None),
        ("new RegExp('foo', '')", None),
        ("new RegExp('foo', 'gimy')", None),
        ("new RegExp(('foo'))", None),
        ("new RegExp(('unrelated', 'foo'))", None),
        ("const flags = 'gi'; new RegExp('foo', flags)", None),
        ("const flags = 'gi'; new RegExp('foo', ('unrelated', flags))", None),
        ("let flags; new RegExp('foo', flags = 'g')", None),
        ("const flags = `gi`; new RegExp(`foo`, (`unrelated`, flags))", None),
        ("const flags = 'gimu'; new RegExp('foo', flags[0])", None),
        ("new window.RegExp('foo')", None), // { "globals": globals.browser },
        ("new global.RegExp('foo')", None), // { "sourceType": "commonjs" },
        ("new globalThis.RegExp('foo')", None), // { "ecmaVersion": 2020 },
        ("/foo/", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        ("/foo/u", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        ("/foo/u", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 6 },
        ("/[[a]/u", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        ("new RegExp('foo', 'u')", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        ("new RegExp('[[a]', 'u')", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        (r#"new RegExp("foo", "\u0067")"#, Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        (r#"new RegExp("foo", `\u0067`)"#, Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        (r#"new RegExp("foo", "\u0075")"#, Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        (r#"new RegExp("foo", `\u0075`)"#, Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        (
            r#"const regularFlags = "sm"; new RegExp("foo", `${regularFlags}g`)"#,
            Some(serde_json::json!([{ "requireFlag": "v" }])),
        ), // { "ecmaVersion": 2024 },
        (
            r#"const regularFlags = "smu"; new RegExp("foo", `${regularFlags}g`)"#,
            Some(serde_json::json!([{ "requireFlag": "v" }])),
        ), // { "ecmaVersion": 2024 },
        ("/foo/v", Some(serde_json::json!([{ "requireFlag": "u" }]))), // { "ecmaVersion": 2024 },
        ("new RegExp('foo')", Some(serde_json::json!([{ "requireFlag": "v" }]))), // { "ecmaVersion": 2024 },
        ("new RegExp('foo', 'v')", Some(serde_json::json!([{ "requireFlag": "u" }]))), // { "ecmaVersion": 2024 }
        ("const a = 'g'; const b = a; new RegExp('x', b)", None),
    ];

    Tester::new(RequireUnicodeRegexp::NAME, RequireUnicodeRegexp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
