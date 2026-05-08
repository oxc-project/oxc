use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;
use oxc_str::static_ident;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_regexp_callee,
};

fn unexpected_regexp_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use a regular expression literal instead of the `RegExp` constructor.")
        .with_label(span)
}

fn unexpected_redundant_regexp_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Regular expression literal is unnecessarily wrapped within a `RegExp` constructor.",
    )
    .with_label(span)
}

fn unexpected_redundant_regexp_with_flags_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Use regular expression literal with flags instead of the `RegExp` constructor.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct PreferRegexLiteralsConfig {
    /// By default, this rule doesn’t check when a regex literal is unnecessarily wrapped in a `RegExp` constructor call.
    /// When the option `disallowRedundantWrapping` is set to `true`, the rule will also disallow such unnecessary patterns.
    ///
    /// Examples of **incorrect** code for `{ "disallowRedundantWrapping": true }`:
    /// ```js
    /// new RegExp(/abc/);
    /// new RegExp(/abc/, 'u');
    /// ```
    ///
    /// Examples of **correct** code for `{ "disallowRedundantWrapping": true }`:
    /// ```js
    /// /abc/;
    /// /abc/u;
    /// new RegExp(/abc/, flags);
    /// ```
    disallow_redundant_wrapping: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreferRegexLiterals(PreferRegexLiteralsConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow use of the RegExp constructor in favor of regular expression literals.
    ///
    /// ### Why is this bad?
    ///
    /// There are two ways to create a regular expression:
    ///
    /// * Regular expression literals, e.g., `/abc/u`.
    /// * The `RegExp` constructor function, e.g., `new RegExp("abc", "u")` or `RegExp("abc", "u")`.
    ///
    /// The constructor function is particularly useful when you want to dynamically generate the pattern,
    /// because it takes string arguments.
    ///
    /// When using the constructor function with string literals, don't forget that the string escaping rules still apply.
    /// If you want to put a backslash in the pattern, you need to escape it in the string literal.
    /// Thus, the following are equivalent:
    ///
    /// ```js
    /// new RegExp("^\\d\\.$");
    ///
    /// /^\d\.$/;
    ///
    /// // matches "0.", "1.", "2." ... "9."
    /// ```
    ///
    /// In the above example, the regular expression literal is easier to read and reason about.
    /// Also, it's a common mistake to omit the extra `\` in the string literal, which would produce a completely different regular expression:
    ///
    /// ```js
    /// new RegExp("^\d\.$");
    ///
    /// // equivalent to /^d.$/, matches "d1", "d2", "da", "db" ...
    /// ```
    ///
    /// When a regular expression is known in advance, it is considered a best practice to avoid the string literal notation on top
    /// of the regular expression notation, and use regular expression literals instead of the constructor function.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new RegExp("abc");
    /// new RegExp("abc", "u");
    /// RegExp("abc");
    /// RegExp("abc", "u");
    /// new RegExp("\\d\\d\\.\\d\\d\\.\\d\\d\\d\\d");
    /// RegExp(`^\\d\\.$`);
    /// new RegExp(String.raw`^\d\.$`);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /abc/;
    /// /abc/u;
    /// /\d\d\.\d\d\.\d\d\d\d/;
    /// /^\d\.$/;
    /// // RegExp constructor is allowed for dynamically generated regular expressions
    /// new RegExp(pattern);
    /// RegExp("abc", flags);
    /// new RegExp(prefix + "abc");
    /// RegExp(`${prefix}abc`);
    /// new RegExp(String.raw`^\d\. ${suffix}`);
    /// ```
    PreferRegexLiterals,
    eslint,
    style,
    pending,
    config = PreferRegexLiterals,
    version = "next",
);

impl Rule for PreferRegexLiterals {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (callee, arguments, span) = match node.kind() {
            AstKind::NewExpression(new_expr) => {
                (&new_expr.callee, &new_expr.arguments, new_expr.span)
            }
            AstKind::CallExpression(call_expr) => {
                (&call_expr.callee, &call_expr.arguments, call_expr.span)
            }
            _ => return,
        };

        if !is_regexp_callee(callee, ctx) {
            return;
        }

        if self.0.disallow_redundant_wrapping
            && let Some(has_flags_argument) = is_unnecessarily_wrapped_regex_literal(arguments, ctx)
        {
            if has_flags_argument {
                ctx.diagnostic(unexpected_redundant_regexp_with_flags_diagnostic(span));
            } else {
                ctx.diagnostic(unexpected_redundant_regexp_diagnostic(span));
            }
        } else if has_only_static_string_arguments(arguments, ctx) {
            ctx.diagnostic(unexpected_regexp_diagnostic(span));
        }
    }
}

fn has_only_static_string_arguments(arguments: &[Argument], ctx: &LintContext) -> bool {
    matches!(arguments.len(), 1 | 2)
        && arguments.iter().all(|arg| is_static_string_argument(arg, ctx))
}

fn is_unnecessarily_wrapped_regex_literal(
    arguments: &[Argument],
    ctx: &LintContext,
) -> Option<bool> {
    match arguments {
        [first] if is_regex_literal_argument(first) => Some(false),
        [first, second]
            if is_regex_literal_argument(first) && is_static_string_argument(second, ctx) =>
        {
            Some(true)
        }
        _ => None,
    }
}

fn is_regex_literal_argument(argument: &Argument) -> bool {
    matches!(
        argument.as_expression().map(Expression::get_inner_expression),
        Some(Expression::RegExpLiteral(_))
    )
}

fn is_static_string_argument(argument: &Argument, ctx: &LintContext) -> bool {
    let Some(expr) = argument.as_expression().map(Expression::get_inner_expression) else {
        return false;
    };

    match expr {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.is_no_substitution_template(),
        Expression::TaggedTemplateExpression(tagged) => {
            tagged.quasi.is_no_substitution_template()
                && is_string_raw_member_expression(&tagged.tag, ctx)
        }
        _ => false,
    }
}

fn is_string_raw_member_expression(expr: &Expression, ctx: &LintContext) -> bool {
    let Some(member) = expr.get_member_expr() else {
        return false;
    };

    member.static_property_name() == Some("raw")
        && matches!(
            member.object().get_inner_expression(),
            Expression::Identifier(ident)
                if ident.is_global_reference_name(static_ident!("String"), ctx.scoping())
        )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("/abc/", None),
        ("/abc/g", None),
        ("new RegExp(pattern)", None),
        (r"new RegExp('\\p{Emoji_Presentation}\\P{Script_Extensions=Latin}' + '', `ug`)", None),
        (r"new RegExp('\\cA' + '')", None),
        ("RegExp(pattern, 'g')", None),
        ("new RegExp(f('a'))", None),
        ("RegExp(prefix + 'a')", None),
        ("new RegExp('a' + suffix)", None),
        ("RegExp(`a` + suffix);", None),
        ("new RegExp(String.raw`a` + suffix);", None),
        ("RegExp('a', flags)", None),
        ("const flags = 'gu';RegExp('a', flags)", None),
        ("RegExp('a', 'g' + flags)", None),
        ("new RegExp(String.raw`a`, flags);", None),
        ("RegExp(`${prefix}abc`)", None),
        ("new RegExp(`a${b}c`);", None),
        ("new RegExp(`a${''}c`);", None),
        ("new RegExp(String.raw`a${b}c`);", None),
        ("new RegExp(String.raw`a${''}c`);", None),
        ("new RegExp('a' + 'b')", None),
        ("RegExp(1)", None),
        (r"new RegExp('(\\p{Emoji_Presentation})\\1' + '', `ug`)", None),
        (r"RegExp(String.raw`\78\126` + '\\5934', '' + `g` + '')", None),
        (
            r"func(new RegExp(String.raw`a${''}c\d`, 'u'),new RegExp(String.raw`a${''}c\d`, 'u'))",
            None,
        ),
        (r#"new RegExp('\\[' + "b\\]")"#, None),
        (
            "new RegExp(/a/, flags);",
            Some(serde_json::json!([{ "disallowRedundantWrapping": true }])),
        ),
        (
            "new RegExp(/a/, `u${flags}`);",
            Some(serde_json::json!([{ "disallowRedundantWrapping": true }])),
        ),
        ("new RegExp(/a/);", Some(serde_json::json!([{}]))),
        ("new RegExp(/a/);", Some(serde_json::json!([{ "disallowRedundantWrapping": false }]))),
        ("new RegExp;", None),
        ("new RegExp();", None),
        ("RegExp();", None),
        ("new RegExp('a', 'g', 'b');", None),
        ("RegExp('a', 'g', 'b');", None),
        ("new RegExp(`a`, `g`, `b`);", None),
        ("RegExp(`a`, `g`, `b`);", None),
        ("new RegExp(String.raw`a`, String.raw`g`, String.raw`b`);", None),
        ("RegExp(String.raw`a`, String.raw`g`, String.raw`b`);", None),
        (
            "new RegExp(/a/, 'u', 'foo');",
            Some(serde_json::json!([{ "disallowRedundantWrapping": true }])),
        ),
        ("new RegExp(String`a`);", None),
        ("RegExp(raw`a`);", None),
        ("new RegExp(f(String.raw)`a`);", None),
        ("RegExp(string.raw`a`);", None),
        ("new RegExp(String.Raw`a`);", None),
        ("new RegExp(String[raw]`a`);", None),
        ("RegExp(String.raw.foo`a`);", None),
        ("new RegExp(String.foo.raw`a`);", None),
        ("RegExp(foo.String.raw`a`);", None),
        ("new RegExp(String.raw);", None),
        ("let String; new RegExp(String.raw`a`);", None),
        ("function foo() { var String; new RegExp(String.raw`a`); }", None),
        ("function foo(String) { RegExp(String.raw`a`); }", None),
        ("if (foo) { const String = bar; RegExp(String.raw`a`); }", None),
        ("new Regexp('abc');", None),
        ("Regexp(`a`);", None),
        ("new Regexp(String.raw`a`);", None),
        ("let RegExp; new RegExp('a');", None),
        ("function foo() { var RegExp; RegExp('a', 'g'); }", None),
        ("function foo(RegExp) { new RegExp(String.raw`a`); }", None),
        ("if (foo) { const RegExp = bar; RegExp('a'); }", None),
        ("class C { #RegExp; foo() { globalThis.#RegExp('a'); } }", None),
        ("new RegExp('[[A--B]]' + a, 'v')", None),
    ];

    let fail = vec![
        ("new RegExp('abc');", None),
        ("RegExp('abc');", None),
        ("new RegExp('abc', 'g');", None),
        ("RegExp('abc', 'g');", None),
        ("new RegExp(`abc`);", None),
        ("RegExp(`abc`);", None),
        ("new RegExp(`abc`, `g`);", None),
        ("RegExp(`abc`, `g`);", None),
        ("new RegExp(String.raw`abc`);", None),
        (
            "new RegExp(String.raw`abc
            abc`);",
            None,
        ),
        (
            "new RegExp(String.raw`	abc
            abc`);",
            None,
        ),
        ("RegExp(String.raw`abc`);", None),
        ("new RegExp(String.raw`abc`, String.raw`g`);", None),
        ("RegExp(String.raw`abc`, String.raw`g`);", None),
        ("new RegExp(String['raw']`a`);", None),
        ("new RegExp('');", None),
        ("RegExp('', '');", None),
        ("new RegExp(String.raw``);", None),
        ("new RegExp('a', `g`);", None),
        ("RegExp(`a`, 'g');", None),
        ("RegExp(String.raw`a`, 'g');", None),
        (r"new RegExp(String.raw`\d`, `g`);", None),
        (r"new RegExp(String.raw`\\d`, `g`);", None),
        (r"new RegExp(String['raw']`\\d`, `g`);", None),
        (r#"new RegExp(String["raw"]`\\d`, `g`);"#, None),
        ("RegExp('a', String.raw`g`);", None),
        ("new globalThis.RegExp('a');", None), // { "ecmaVersion": 2020 },
        ("globalThis.RegExp('a');", None),     // { "ecmaVersion": 2020 },
        ("new RegExp(/a/);", Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ]))),
        (
            "new RegExp(/a/, 'u');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/g, '');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/g, 'g');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/ig, 'g');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/g, 'ig');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/i, 'g');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/i, 'i');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/, `u`);",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/, `gi`);",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        ("new RegExp('a');", Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ]))),
        (
            "new RegExp(/a/, String.raw`u`);",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/ /* comment */);",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "new RegExp(/a/, 'd');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ), // { "ecmaVersion": 2021, },
        (
            "(a)
            new RegExp(/b/);",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        (
            "(a)
            new RegExp(/b/, 'g');",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        ("a/RegExp(/foo/);", Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ]))),
        (
            "RegExp(/foo/)in a;",
            Some(serde_json::json!([ { "disallowRedundantWrapping": true, }, ])),
        ),
        ("new RegExp((String?.raw)`a`);", None),
        ("new RegExp('+');", None),
        ("new RegExp('*');", None),
        ("RegExp('+');", None),
        ("RegExp('*');", None),
        ("new RegExp('+', 'g');", None),
        ("new RegExp('*', 'g');", None),
        ("RegExp('+', 'g');", None),
        ("RegExp('*', 'g');", None),
        ("RegExp('abc', 'u');", None), // { "ecmaVersion": 3, "sourceType": "script", },
        ("new RegExp('abc', 'd');", None), // { "ecmaVersion": 2021, },
        ("RegExp('abc', 'd');", None), // { "ecmaVersion": 2022, },
        (r"RegExp('\\\\', '');", None),
        (r"RegExp('\n', '');", None), // { "ecmaVersion": 2022, },
        (r"RegExp('\n\n', '');", None),
        (r"RegExp('\t', '');", None),
        (r"RegExp('\t\t', '');", None),
        (r"RegExp('\r\n', '');", None),
        (r"RegExp('\u1234', 'g')", None),
        (r"RegExp('\u{1234}', 'g')", None),
        (r"RegExp('\u{11111}', 'g')", None),
        (r"RegExp('\v', '');", None),
        (r"RegExp('\v\v', '');", None),
        (r"RegExp('\f', '');", None),
        (r"RegExp('\f\f', '');", None),
        (r"RegExp('\\b', '');", None),
        (r"RegExp('\\b\\b', '');", None),
        (r"new RegExp('\\B\\b', '');", None),
        (r"RegExp('\\w', '');", None),
        (r"new globalThis.RegExp('\\W', '');", None), // { "globals": { "globalThis": "readonly", }, },
        (r"RegExp('\\s', '');", None),
        (r"new RegExp('\\S', '')", None),
        (r"globalThis.RegExp('\\d', '');", None), // { "globals": { "globalThis": "readonly", }, },
        (r"globalThis.RegExp('\\D', '')", None),  // { "globals": { "globalThis": "readonly", }, },
        (r"globalThis.RegExp('\\\\\\D', '')", None), // { "globals": { "globalThis": "readonly", }, },
        (r"new RegExp('\\D\\D', '')", None),
        (r"new globalThis.RegExp('\\0\\0', '');", None), // { "globals": { "globalThis": "writable", }, },
        (r"new RegExp('\\0\\0', '');", None),
        (r"new RegExp('\0\0', 'g');", None),
        (r"RegExp('\\0\\0\\0', '')", None),
        (r"RegExp('\\78\\126\\5934', '')", None), // { "ecmaVersion": 2022, },
        (r"new window['RegExp']('\\x56\\x78\\x45', '');", None), // { "globals": { "window": "readonly", }, },
        ("a in(RegExp('abc'))", None),
        (
            r#"x = y
                        RegExp("foo").test(x) ? bar() : baz()"#,
            None,
        ),
        (r"func(new RegExp(String.raw`\w{1, 2`, 'u'),new RegExp(String.raw`\w{1, 2`, 'u'))", None),
        (
            r#"x = y;
                        RegExp("foo").test(x) ? bar() : baz()"#,
            None,
        ),
        (r#"typeof RegExp("foo")"#, None),
        (
            r#"RegExp("foo") instanceof RegExp(String.raw`blahblah`, 'g') ? typeof new RegExp('(\\p{Emoji_Presentation})\\1', `ug`) : false"#,
            None,
        ),
        ("[   new RegExp(`someregular`)]", None),
        (
            r#"const totallyValidatesEmails = new RegExp("\\\\S+@(\\\\S+\\\\.)+\\\\S+")
                        if (typeof totallyValidatesEmails === 'object') {
                            runSomethingThatExists(Regexp('stuff'))
                        }"#,
            None,
        ),
        (
            "!new RegExp('^Hey, ', 'u') && new RegExp('jk$') && ~new RegExp('^Sup, ') || new RegExp('hi') + new RegExp('person') === -new RegExp('hi again') ? 5 * new RegExp('abc') : 'notregbutstring'",
            None,
        ),
        (
            r#"#!/usr/bin/sh
                        RegExp("foo")"#,
            None,
        ),
        (r#"async function abc(){await new RegExp("foo")}"#, None), // { "ecmaVersion": 8, "sourceType": "module", },
        (r#"function* abc(){yield new RegExp("foo")}"#, None),
        (r#"function* abc(){yield* new RegExp("foo")}"#, None),
        ("console.log({ ...new RegExp('a') })", None),
        ("delete RegExp('a');", None),
        ("void RegExp('a');", None),
        (r#"new RegExp("\\S+@(\\S+\\.)+\\S+")**RegExp('a')"#, None),
        (r#"new RegExp("\\S+@(\\S+\\.)+\\S+")%RegExp('a')"#, None),
        ("a in RegExp('abc')", None),
        (
            "
                        /abc/ == new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ === new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ != new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ !== new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ > new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ < new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ >= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ <= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ << new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ >> new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ >>> new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ ^ new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ & new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        /abc/ | new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        null ?? new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc *= new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        console.log({a: new RegExp('sup')})
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        console.log(() => {new RegExp('sup')})
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        function abc() {new RegExp('sup')}
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        function abc() {return new RegExp('sup')}
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc <<= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc >>= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc >>>= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc ^= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc &= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc |= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc ??= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc &&= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc ||= new RegExp('cba');
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc **= new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc /= new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc += new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc -= new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        abc %= new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (
            "
                        () => new RegExp('blah')
                        ",
            None,
        ), // { "ecmaVersion": 2021, },
        (r#"a/RegExp("foo")in b"#, None),
        (r#"a/RegExp("foo")instanceof b"#, None),
        (
            r#"do RegExp("foo")
            while (true);"#,
            None,
        ),
        (
            "for(let i;i<5;i++) { break
            new RegExp('search')}",
            None,
        ),
        (
            "for(let i;i<5;i++) { continue
            new RegExp('search')}",
            None,
        ),
        (
            r#"
                        switch (value) {
                            case "possibility":
                                console.log('possibility matched')
                            case RegExp('myReg').toString():
                                console.log('matches a regexp\' toString value')
                                break;
                        }
                        "#,
            None,
        ),
        ("throw new RegExp('abcdefg') // fail with a regular expression", None),
        ("for (value of new RegExp('something being searched')) { console.log(value) }", None),
        (
            "(async function(){for await (value of new RegExp('something being searched')) { console.log(value) }})()",
            None,
        ), // { "ecmaVersion": 2018, },
        ("for (value in new RegExp('something being searched')) { console.log(value) }", None),
        ("if (condition1 && condition2) new RegExp('avalue').test(str);", None),
        (
            "debugger
            new RegExp('myReg')",
            None,
        ),
        (r#"RegExp("\\\n")"#, None),
        (r#"RegExp("\\\t")"#, None),
        (r#"RegExp("\\\f")"#, None),
        (r#"RegExp("\\\v")"#, None),
        (r#"RegExp("\\\r")"#, None),
        (r#"new RegExp("	")"#, None),
        (r#"new RegExp("/")"#, None),
        (r#"new RegExp("\.")"#, None),
        (r#"new RegExp("\\.")"#, None),
        (r#"new RegExp("\\\n\\\n")"#, None),
        (r#"new RegExp("\\\n\\\f\\\n")"#, None),
        (r#"new RegExp("\u000A\u000A");"#, None),
        ("new RegExp('mysafereg' /* comment explaining its safety */)", None),
        ("new RegExp('[[A--B]]', 'v')", None), // { "ecmaVersion": 2024 },
        ("new RegExp('[[A--B]]', 'v')", None), // { "ecmaVersion": 2023 },
        ("new RegExp('[[A&&&]]', 'v')", None), // { "ecmaVersion": 2024 },
        ("new RegExp('a', 'uv')", None),       // { "ecmaVersion": 2024 },
        ("new RegExp(/a/, 'v')", Some(serde_json::json!([{ "disallowRedundantWrapping": true }]))), // { "ecmaVersion": 2024 },
        ("new RegExp(/a/, 'v')", Some(serde_json::json!([{ "disallowRedundantWrapping": true }]))), // { "ecmaVersion": 2023 },
        ("new RegExp(/a/g, 'v')", Some(serde_json::json!([{ "disallowRedundantWrapping": true }]))), // { "ecmaVersion": 2024 },
        (
            "new RegExp(/[[A--B]]/v, 'g')",
            Some(serde_json::json!([{ "disallowRedundantWrapping": true }])),
        ), // { "ecmaVersion": 2024 },
        ("new RegExp(/a/u, 'v')", Some(serde_json::json!([{ "disallowRedundantWrapping": true }]))), // { "ecmaVersion": 2024 },
        ("new RegExp(/a/v, 'u')", Some(serde_json::json!([{ "disallowRedundantWrapping": true }]))), // { "ecmaVersion": 2024 },
        (
            "new RegExp(/[[A--B]]/v, 'u')",
            Some(serde_json::json!([{ "disallowRedundantWrapping": true }])),
        ), // { "ecmaVersion": 2024 },
        ("new RegExp('(?i:foo)bar')", None), // { "ecmaVersion": 2025 },
        ("new RegExp('(?i:foo)bar')", None), // { "ecmaVersion": 2024 },
        ("var regex = new RegExp('foo', 'u');", None), // { "ecmaVersion": 2015, }
    ];

    Tester::new(PreferRegexLiterals::NAME, PreferRegexLiterals::PLUGIN, pass, fail)
        .test_and_snapshot();
}
