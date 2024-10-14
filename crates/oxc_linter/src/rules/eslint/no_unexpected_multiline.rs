use memchr::{memchr, memrchr};
use oxc_ast::{ast::BinaryOperator, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoUnexpectedMultiline;

enum DiagnosticKind {
    FunctionCall { open_paren_span: Span },
    PropertyAccess { open_bracket_span: Span },
    TaggedTemplate { backtick_span: Span },
    Division { slash_span: Span },
}

fn no_unexpected_multiline_diagnostic(kind: &DiagnosticKind) -> OxcDiagnostic {
    match kind {
        DiagnosticKind::FunctionCall { open_paren_span } => OxcDiagnostic::warn(
            "Unexpected newline between function name and open parenthesis of function call",
        )
        .with_label(
            open_paren_span.label("this is parsed as a function call, which may be unintentional"),
        )
        .with_help(
            "If you did not intend to make a function call, insert ';' before the parenthesis",
        ),
        DiagnosticKind::PropertyAccess { open_bracket_span } => OxcDiagnostic::warn(
            "Unexpected newline between object and open bracket of property access",
        )
        .with_label(
            open_bracket_span
                .label("this is parsed as a property access, which may be unintentional"),
        )
        .with_help("If you did not intend to access a property, insert ';' before the bracket"),
        DiagnosticKind::TaggedTemplate { backtick_span } => {
            OxcDiagnostic::warn(
                "Unexpected newline between template tag and template literal",
            )
            .with_label(backtick_span.label(
                "this is parsed as a tagged template, which may be unintentional",
            ))
            .with_help("If you did not intend for this to be a tagged template, insert ';' before the backtick")
        }
        DiagnosticKind::Division { slash_span } => {
            OxcDiagnostic::warn(
                "Unexpected newline between numerator and division operator",
            )
            .with_label(
                slash_span.label("this is parsed as division, which may be unintentional"),
            )
            .with_help("If you did not intend to divide, insert ';' before the slash")
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// In most cases, semicolons are not required in JavaScript in order for code to be parsed
    /// and executed as expected. Typically this occurs because semicolons are automatically
    /// inserted based on a fixed set of rules. This rule exists to detect those cases where a semicolon
    /// is NOT inserted automatically, and may be parsed differently than expected.
    ///
    /// ### Why is this bad?
    ///
    /// Code that has unexpected newlines may be parsed and executed differently than what the
    /// developer intended. This can lead to bugs that are difficult to track down.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var a = b
    /// (x || y).doSomething()
    ///
    /// var a = b
    /// [a, b, c].forEach(doSomething)
    ///
    /// let x = function() {}
    ///  `hello`
    ///
    ///	foo
    ///	/bar/g.test(baz)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var a = b;
    /// (x || y).doSomething()
    ///
    /// var a = b;
    /// [a, b, c].forEach(doSomething)
    ///
    /// let x = function() {};
    /// `hello`
    ///
    /// foo;
    /// /bar/g.test(baz)
    /// ```
    NoUnexpectedMultiline,
    suspicious,
    fix_dangerous
);

impl Rule for NoUnexpectedMultiline {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if call_expr.optional {
                    return;
                }
                if let Some(AstKind::ChainExpression(_)) = ctx.nodes().parent_kind(node.id()) {
                    return;
                }

                let start = if let Some(generics) = &call_expr.type_parameters {
                    generics.span.end
                } else {
                    call_expr.callee.span().end
                };

                let span = Span::new(start, call_expr.span.end);
                if let Some(open_paren_pos) = has_newline_before(ctx, span, b'(') {
                    let paren_span = Span::sized(span.start + open_paren_pos, 1);

                    ctx.diagnostic_with_dangerous_fix(
                        no_unexpected_multiline_diagnostic(&DiagnosticKind::FunctionCall {
                            open_paren_span: paren_span,
                        }),
                        |fixer| fixer.insert_text_before_range(paren_span, ";"),
                    );
                }
            }
            AstKind::MemberExpression(member_expr) => {
                if !member_expr.is_computed() || member_expr.optional() {
                    return;
                }

                let span = Span::new(member_expr.object().span().end, member_expr.span().end);
                if let Some(open_bracket_pos) = has_newline_before(ctx, span, b'[') {
                    let bracket_span = Span::sized(span.start + open_bracket_pos, 1);

                    ctx.diagnostic_with_dangerous_fix(
                        no_unexpected_multiline_diagnostic(&DiagnosticKind::PropertyAccess {
                            open_bracket_span: bracket_span,
                        }),
                        |fixer| fixer.insert_text_before_range(bracket_span, ";"),
                    );
                }
            }
            AstKind::TaggedTemplateExpression(tagged_template_expr) => {
                let start = if let Some(generics) = &tagged_template_expr.type_parameters {
                    generics.span.end
                } else {
                    tagged_template_expr.tag.span().end
                };

                let span = Span::new(start, tagged_template_expr.span.end);
                if let Some(backtick_pos) = has_newline_before(ctx, span, b'`') {
                    let backtick_span = Span::sized(span.start + backtick_pos, 1);

                    ctx.diagnostic_with_dangerous_fix(
                        no_unexpected_multiline_diagnostic(&DiagnosticKind::TaggedTemplate {
                            backtick_span,
                        }),
                        |fixer| fixer.insert_text_before_range(backtick_span, ";"),
                    );
                }
            }
            AstKind::BinaryExpression(binary_expr) => {
                if binary_expr.operator != BinaryOperator::Division {
                    return;
                }
                let Some(AstKind::BinaryExpression(parent_binary_expr)) =
                    ctx.nodes().parent_kind(node.id())
                else {
                    return;
                };
                if parent_binary_expr.operator != BinaryOperator::Division {
                    return;
                }
                let span = Span::new(binary_expr.left.span().end, parent_binary_expr.span().end);
                let src = ctx.source_range(span);

                let Some(newline) = memchr(b'\n', src.as_bytes()) else {
                    return;
                };
                let Some(first_slash) = memchr(b'/', src.as_bytes()) else {
                    return;
                };
                let Some(second_slash) = memrchr(b'/', src.as_bytes()) else {
                    return;
                };
                if first_slash == second_slash {
                    return;
                }

                // the "identifier" will be the characters immediately following the second slash
                // until we reach a non-identifier character
                let ident_name = src
                    .chars()
                    .skip(second_slash + 1)
                    // This is a rough approximation of "looks like an identifier"
                    .take_while(|c| {
                        !(c.is_whitespace() || c.is_ascii_punctuation()) || c == &'_' || c == &'$'
                    })
                    .collect::<String>();

                if newline < parent_binary_expr.left.span().start as usize
					// The identifier name should look like it was an attempt to use a regex
					&& is_regex_flag(ident_name.as_str())
					// if it was a regex attempt, the second slash should be before the identifier
                    && second_slash + (span.start as usize) + 1 == parent_binary_expr.right.span().start as usize
                {
                    let slash_span =
                        Span::sized(span.start + u32::try_from(first_slash).unwrap(), 1);

                    ctx.diagnostic_with_dangerous_fix(
                        no_unexpected_multiline_diagnostic(&DiagnosticKind::Division {
                            slash_span,
                        }),
                        |fixer| fixer.insert_text_before_range(slash_span, ";"),
                    );
                }
            }
            _ => {}
        }
    }
}

// based on ESLint source: REGEX_FLAG_MATCHER = /^[gimsuy]+$/u;
fn is_regex_flag(str: &str) -> bool {
    // check if all characters in the string are in the set [gimsuy]
    for c in str.chars() {
        if !matches!(c, 'g' | 'i' | 'm' | 's' | 'u' | 'y') {
            return false;
        }
    }
    true
}

/// Check if there is a newline proceeding a target character within a snippet of source text.
/// Returns `None` if the character is not found at all or has no proceeding newline. Otherwise,
/// returns the byte offset of the target character with respect to the start of the span.
///
/// Newlines do not have to be directly before the target character, but can be anywhere before it.
fn has_newline_before(ctx: &LintContext, span: Span, c: u8) -> Option<u32> {
    let src = ctx.source_range(span).as_bytes();
    let target = memchr(c, src)?;
    let newline = memchr(b'\n', src)?;

    (newline < target).then(|| u32::try_from(target).unwrap())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "(x || y).aFunction()",
        "[a, b, c].forEach(doSomething)",
        "var a = b;\n(x || y).doSomething()",
        "var a = b\n;(x || y).doSomething()",
        "var a = b\nvoid (x || y).doSomething()",
        "var a = b;\n[1, 2, 3].forEach(console.log)",
        "var a = b\nvoid [1, 2, 3].forEach(console.log)",
        "\"abc\\\n(123)\"",
        "var a = (\n(123)\n)",
        "f(\n(x)\n)",
        "(\nfunction () {}\n)[1]",
        "let x = function() {};\n   `hello`", // { "ecmaVersion": 6 },
        "let x = function() {}\nx `hello`",   // { "ecmaVersion": 6 },
        "String.raw `Hi\n${2+3}!`;",          // { "ecmaVersion": 6 },
        "x\n.y\nz `Valid Test Case`",         // { "ecmaVersion": 6 },
        "f(x\n)`Valid Test Case`",            // { "ecmaVersion": 6 },
        "x.\ny `Valid Test Case`",            // { "ecmaVersion": 6 },
        "(x\n)`Valid Test Case`",             // { "ecmaVersion": 6 },
        "
			foo
			/ bar /2
		",
        "
			foo
			/ bar / mgy
		",
        "
			foo
			/ bar /
			gym
		",
        "
			foo
			/ bar
			/ ygm
		",
        "
			foo
			/ bar /GYM
		",
        "
			foo
			/ bar / baz
		",
        "foo /bar/g",
        "
			foo
			/denominator/
			2
		",
        "
			foo
			/ /abc/
		",
        "
			5 / (5
			/ 5)
		",
        "
			tag<generic>`
				multiline
			`;
		", // {                "parser": require("../../fixtures/parsers/typescript-parsers/tagged-template-with-generic/tagged-template-with-generic-1")            },
        "
			tag<
				generic
			>`
				multiline
			`;
		", // {                "parser": require("../../fixtures/parsers/typescript-parsers/tagged-template-with-generic/tagged-template-with-generic-2")            },
        "
			tag<
				generic
			>`multiline`;
		", // {                "parser": require("../../fixtures/parsers/typescript-parsers/tagged-template-with-generic/tagged-template-with-generic-3")            },
        "var a = b\n  ?.(x || y).doSomething()", // { "ecmaVersion": 2020 },
        "var a = b\n  ?.[a, b, c].forEach(doSomething)", // { "ecmaVersion": 2020 },
        "var a = b?.\n  (x || y).doSomething()", // { "ecmaVersion": 2020 },
        "var a = b?.\n  [a, b, c].forEach(doSomething)", // { "ecmaVersion": 2020 },
        "class C { field1\n[field2]; }",         // { "ecmaVersion": 2022 },
        "class C { field1\n*gen() {} }",         // { "ecmaVersion": 2022 },
        "class C { field1 = () => {}\n[field2]; }", // { "ecmaVersion": 2022 },
        "class C { field1 = () => {}\n*gen() {} }", // { "ecmaVersion": 2022 }
        "const foo = bar<{
            a: string
            b: number
        }>();",
        "const foo = bar<{
            [key: string | number]: string
        }>();",
    ];

    let fail = vec![
        "var a = b\n(x || y).doSomething()",
        "var a = (a || b)\n(x || y).doSomething()",
        "var a = (a || b)\n(x).doSomething()",
        "var a = b\n[a, b, c].forEach(doSomething)",
        "var a = b\n    (x || y).doSomething()",
        "var a = b\n  [a, b, c].forEach(doSomething)",
        "let x = function() {}\n `hello`", // { "ecmaVersion": 6 },
        "let x = function() {}\nx\n`hello`", // { "ecmaVersion": 6 },
        "x\n.y\nz\n`Invalid Test Case`",   // { "ecmaVersion": 6 },
        "
			foo
			/ bar /gym
		",
        "
			foo
			/ bar /g
		",
        "
			foo
			/ bar /g.test(baz)
		",
        "
			foo
			/bar/gimuygimuygimuy.test(baz)
		",
        "
			foo
			/bar/s.test(baz)
		",
        "const x = aaaa<\n  test\n>/*\ntest\n*/`foo`", // {                "parser": require("../../fixtures/parsers/typescript-parsers/tagged-template-with-generic/tagged-template-with-generic-and-comment")            },
        "class C { field1 = obj\n[field2]; }",         // { "ecmaVersion": 2022 },
        "class C { field1 = function() {}\n[field2]; }", // { "ecmaVersion": 2022 }
    ];

    // TODO: add more fixer tests
    let fix = vec![("var a = b\n(x || y).doSomething()", "var a = b\n;(x || y).doSomething()")];

    Tester::new(NoUnexpectedMultiline::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
