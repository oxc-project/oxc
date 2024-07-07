use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::{
    ast::{Argument, CallExpression, NewExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use std::iter::Peekable;

#[derive(Debug, Default, Clone)]
pub struct NoUselessBackreference;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUselessBackreference,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

fn is_open_bracet_part_of_group_capture(char: char, iter: &mut Peekable<std::str::Chars<'_>>) -> bool {
    assert!(char == '(');

    let next_char = iter.peek();

    match next_char {
        Some(next_char) => {
            if next_char.to_owned() == '?' {
                iter.next(); // pointer is now on ?
                let next_char = iter.peek();

                match next_char {
                    Some(next_char) => {
                        let owned_char = next_char.to_owned();

                        // no non-capture-groups, lookaheads and negative lookaheads
                        if owned_char != ':' && owned_char != '=' && owned_char != '!' {
                            return true;
                        }
                    }
                    _ => {}
                };
            }
        }
        _ => {}
    }

    return false;
}

fn get_capture_group_count(regex: &str) -> u32 {
    let mut backslash_started = false;
    let mut count = 0;
    let mut chars = regex.chars().peekable();

    while let Some(char) = chars.next() {
        if char == '\\' {
            backslash_started = !backslash_started;
            continue;
        }

        if !backslash_started && char == '(' && is_open_bracet_part_of_group_capture(char, &mut chars) {
            count += 1;
        }

        if char != '\\' {
            backslash_started = false;
        }
    }

    return count;
}

fn has_invalid_back_reference(regex: &str) -> bool {
    let mut backslash_started = false;
    let mut inside_non_caputure_group: bool = false;
    let mut captures_ended: u32 = 0; // ToDO: can be 8, atm for just simple programming
    let captures_groups: u32 = get_capture_group_count(regex); // ToDO: can be 8, atm for just simple programming

    for char in regex.chars() {
        // check for backslash
        if char == '\\' {
            backslash_started = !backslash_started;
            continue;
        }

        if !backslash_started && is_open_bracet_part_of_group_capture(char, )

        if !backslash_started && char == ')' {
            if !inside_non_caputure_group {
                captures_ended += 1;
            }

            inside_non_caputure_group = false;
        }

        // starts with a backlash followed by a positive number
        if backslash_started && char != '0' && char.is_ascii_digit() {
            let digit_result = char.to_digit(10);

            match digit_result {
                Some(digit) => {
                    println!("digits {} {}, {}, {}", regex, digit, captures_groups, captures_ended);

                    // we are trying to access a capture group and not an octal
                    if digit <= captures_groups {
                        // this capture group did not end
                        if digit > captures_ended {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }

        if char != '\\' {
            backslash_started = false;
        }
    }

    return false;
}

fn is_regexp_new_expression(expr: &NewExpression<'_>) -> bool {
    expr.callee.is_specific_id("RegExp") && expr.arguments.len() > 0
}

fn is_regexp_call_expression(expr: &CallExpression<'_>) -> bool {
    expr.callee.is_specific_id("RegExp") && expr.arguments.len() > 0
}

impl Rule for NoUselessBackreference {
    fn run(&self, node: &AstNode, ctx: &LintContext) {
        match node.kind() {
            AstKind::RegExpLiteral(literal)
                if has_invalid_back_reference(&literal.regex.pattern) =>
            {
                ctx.diagnostic(OxcDiagnostic::warn("no back reference").with_label(literal.span))
            }
            AstKind::NewExpression(expr) if is_regexp_new_expression(expr) => {
                let regex = &expr.arguments[0];

                match regex {
                    Argument::StringLiteral(arg) if has_invalid_back_reference(&arg.value) => ctx
                        .diagnostic(OxcDiagnostic::warn("no back reference").with_label(arg.span)),
                    _ => {}
                };
            }
            AstKind::CallExpression(expr) if is_regexp_call_expression(expr) => {
                let regex = &expr.arguments[0];

                match regex {
                    Argument::StringLiteral(arg) if has_invalid_back_reference(&arg.value) => ctx
                        .diagnostic(OxcDiagnostic::warn("no back reference").with_label(arg.span)),
                    _ => {}
                };
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // not a regular expression
        r#"'\1(a)'"#,
        r#"regExp('\\1(a)')"#,
        r#"new Regexp('\\1(a)', 'u')"#,
        r#"RegExp.foo('\\1(a)', 'u')"#,
        r#"new foo.RegExp('\\1(a)')"#,
        // unkown pattern
        r#"RegExp(p)"#,
        r#"new RegExp(p, 'u')"#,
        r#"RegExp('\\1(a)' + suffix)"#,
        // r#"new RegExp(`${prefix}\\\\1(a)"#,

        // not the global RegExp
        // ToDo: Do we really need this checks?
        // r#"let RegExp; new RegExp('\\1(a)');"#,
        // r#"function foo() { var RegExp; RegExp('\\1(a)', 'u'); }"#,
        // r#"function foo(RegExp) { new RegExp('\\1(a)'); }"#,
        // r#"if (foo) { const RegExp = bar; RegExp('\\1(a)'); }"#,

        // no capturing groups
        r#"/(?:)/"#,
        r#"/(?:a)/"#,
        r#"new RegExp('')"#,
        r#"RegExp('(?:a)|(?:b)*')"#,
        r#"/^ab|[cd].\n$/"#,
        // no backreferences
        r#"/(a)/"#,
        r#"RegExp('(a)|(b)')"#,
        r#"new RegExp('\\n\\d(a)')"#,
        r#"/\0(a)/"#,
        r#"/\0(a)/u"#,
        r#"/(?<=(a))(b)(?=(c))/"#,
        r#"/(?<!(a))(b)(?!(c))/"#,
        r#"/(?<foo>a)/"#,
        // not really a backreference
        r#"RegExp('\1(a)')"#,    // string octal escape
        r#"RegExp('\\\\1(a)')"#, // escaped backslash
        r#"/\\1(a)/"#,               // escaped backslash
        r#"/\1/"#,                   // group 1 doesn't exist, this is a regex octal escape
        r#"/^\1$/"#,                 // group 1 doesn't exist, this is a regex octal escape
        r#"/\2(a)/"#,                // group 2 doesn't exist, this is a regex octal escape
        r#"/\1(?:a)/"#,              // group 1 doesn't exist, this is a regex octal escape
        r#"/\1(?=a)/"#,              // group 1 doesn't exist, this is a regex octal escape
        r#"/\1(?!a)/"#,              // group 1 doesn't exist, this is a regex octal escape
        r#"/^[\1](a)$/"#,            // \N in a character class is a regex octal escape
        r#"new RegExp('[\\1](a)')"#, // \N in a character class is a regex octal escape
        r#"/\11(a)/"#,               // regex octal escape \11, regex matches "\x09a"
        r#"/\k<foo>(a)/"#, // without the 'u' flag and any named groups this isn't a syntax error, matches "k<foo>a"
        r#"/^(a)\1\2$/"#,  // \1 is a backreference, \2 is an octal escape sequence.
        // Valid backreferences: correct position, after the group
        r#"/(a)\1/"#,
        r#"/(a).\1/"#,
        r#"RegExp('(a)\\1(b)')"#,
        r#"/(a)(b)\2(c)/"#,
        r#"/(?<foo>a)\k<foo>/"#,
        r#"new RegExp('(.)\\1')"#,
        r#"RegExp('(a)\\1(?:b)')"#,
        r#"/(a)b\1/"#,
        r#"/((a)\2)/"#,
        r#"/((a)b\2c)/"#,
        r#"/^(?:(a)\1)$/"#,
        r#"/^((a)\2)$/"#,
        r#"/^(((a)\3))|b$/"#,
        r#"/a(?<foo>(.)b\2)/"#,
        r#"/(a)?(b)*(\1)(c)/"#,
        r#"/(a)?(b)*(\2)(c)/"#,
        r#"/(?<=(a))b\1/"#,
        r#"/(?<=(?=(a)\1))b/"#,
        // Valid backreferences: correct position before the group when they're both in the same lookbehind
        // r#"/(?<!\1(a))b/"#,
        // r#"/(?<=\1(a))b/"#,
        // r#"/(?<!\1.(a))b/"#,
        // r#"/(?<=\1.(a))b/"#,
        // r#"/(?<=(?:\1.(a)))b/"#,
        // r#"/(?<!(?:\1)((a)))b/"#,
        // r#"/(?<!(?:\2)((a)))b/"#,
        // r#"/(?=(?<=\1(a)))b/"#,
        // r#"/(?=(?<!\1(a)))b/"#,
        // r#"/(.)(?<=\2(a))b/"#,
        // Valid backreferences: not a reference into another alternative
        r#"/^(a)\1|b/"#,
        r#"/^a|(b)\1/"#,
        r#"/^a|(b|c)\1/"#,
        r#"/^(a)|(b)\2/"#,
        r#"/^(?:(a)|(b)\2)$/"#,
        r#"/^a|(?:.|(b)\1)/"#,
        r#"/^a|(?:.|(b).(\1))/"#,
        r#"/^a|(?:.|(?:(b)).(\1))/"#,
        r#"/^a|(?:.|(?:(b)|c).(\1))/"#,
        r#"/^a|(?:.|(?:(b)).(\1|c))/"#,
        r#"/^a|(?:.|(?:(b)|c).(\1|d))/"#,
        // Valid backreferences: not a reference into a negative lookaround (reference from within the same lookaround is allowed)
        r#"/.(?=(b))\1/"#,
        r#"/.(?<=(b))\1/"#,
        r#"/a(?!(b)\1)./"#,
        // r#"/a(?<!\1(b))./"#,
        r#"/a(?!(b)(\1))./"#,
        r#"/a(?!(?:(b)\1))./"#,
        r#"/a(?!(?:(b))\1)./"#,
        // r#"/a(?<!(?:\1)(b))./"#,
        // r#"/a(?<!(?:(?:\1)(b)))./"#,
        r#"/(?<!(a))(b)(?!(c))\2/"#,
        r#"/a(?!(b|c)\1)./"#,
        // ignore regular expressions with syntax errors
        r#"RegExp('\\1(a)[')"#, // \1 would be an error, but the unterminated [ is a syntax error
        r#"new RegExp('\\1(a){', 'u')"#, // \1 would be an error, but the unterminated { is a syntax error because of the 'u' flag
        r#"new RegExp('\\1(a)\\2', 'ug')"#, // \1 would be an error, but \2 is syntax error because of the 'u' flag
        r#"const flags = 'gus'; RegExp('\\1(a){', flags);"#, // \1 would be an error, but the rule is aware of the 'u' flag so this is a syntax error
        r#"RegExp('\\1(a)\\k<foo>', 'u')"#, // \1 would be an error, but \k<foo> produces syntax error because of the u flag
        r#"new RegExp('\\k<foo>(?<foo>a)\\k<bar>')"#, // \k<foo> would be an error, but \k<bar> produces syntax error because group <bar> doesn't exist
        // ES2024
        r#"new RegExp('([[A--B]])\\1', 'v')"#,
        r#"new RegExp('[[]\\1](a)', 'v')"#, // SyntaxError
        // ES2025
        r#"/((?<foo>bar)\k<foo>|(?<foo>baz))/"#,
    ];

    let fail = vec![
        r#"/(b)(\2a)/"#,
        r#"/\k<foo>(?<foo>bar)/"#,
        r#"RegExp('(a|bc)|\\1')"#,
        r#"new RegExp('(?!(?<foo>\\n))\\1')"#,
        r#"/(?<!(a)\1)b/"#,
        // nested
        r#"new RegExp('(\\1)')"#,
        r#"/^(a\1)$/"#,
        r#"/^((a)\1)$/"#,
        r#"new RegExp('^(a\\1b)$')"#,
        r#"RegExp('^((\\1))$')"#,
        r#"/((\2))/"#,
        r#"/a(?<foo>(.)b\1)/"#,
        r#"/a(?<foo>\k<foo>)b/"#,
        r#"/^(\1)*$/"#,
        r#"/^(?:a)(?:((?:\1)))*$/"#,
        r#"/(?!(\1))/"#,
        r#"/a|(b\1c)/"#,
        r#"/(a|(\1))/"#,
        r#"/(a|(\2))/"#,
        r#"/(?:a|(\1))/"#,
        r#"/(a)?(b)*(\3)/"#,
        r#"/(?<=(a\1))b/"#,
        // forward
        r#"/\1(a)/"#,
        r#"/\1.(a)/"#,
        r#"/(?:\1)(?:(a))/"#,
        r#"/(?:\1)(?:((a)))/"#,
        r#"/(?:\2)(?:((a)))/"#,
        r#"/(?:\1)(?:((?:a)))/"#,
        r#"/(\2)(a)/"#,
        r#"RegExp('(a)\\2(b)')"#,
        r#"/(?:a)(b)\2(c)/"#,
        r#"/\k<foo>(?<foo>a)/"#,
        r#"/(?:a(b)\2)(c)/"#,
        r#"new RegExp('(a)(b)\\3(c)')"#,
        r#"/\1(?<=(a))./"#,
        r#"/\1(?<!(a))./"#,
        r#"/(?<=\1)(?<=(a))/"#,
        r#"/(?<!\1)(?<!(a))/"#,
        r#"/(?=\1(a))./"#,
        r#"/(?!\1(a))./"#,
        // backward in the same lookbehind
        r#"/(?<=(a)\1)b/"#,
        r#"/(?<!.(a).\1.)b/"#,
        r#"/(.)(?<!(b|c)\2)d/"#,
        r#"/(?<=(?:(a)\1))b/"#,
        r#"/(?<=(?:(a))\1)b/"#,
        r#"/(?<=(a)(?:\1))b/"#,
        r#"/(?<!(?:(a))(?:\1))b/"#,
        r#"/(?<!(?:(a))(?:\1)|.)b/"#,
        r#"/.(?!(?<!(a)\1))./"#,
        r#"/.(?=(?<!(a)\1))./"#,
        r#"/.(?!(?<=(a)\1))./"#,
        r#"/.(?=(?<=(a)\1))./"#,
        // into another alternative
        r#"/(a)|\1b/"#,
        r#"/^(?:(a)|\1b)$/"#,
        r#"/^(?:(a)|b(?:c|\1))$/"#,
        r#"/^(?:a|b(?:(c)|\1))$/"#,
        r#"/^(?:(a(?!b))|\1b)+$/"#,
        r#"/^(?:(?:(a)(?!b))|\1b)+$/"#,
        r#"/^(?:(a(?=a))|\1b)+$/"#,
        r#"/^(?:(a)(?=a)|\1b)+$/"#,
        r#"/.(?:a|(b)).|(?:(\1)|c)./"#,
        r#"/.(?!(a)|\1)./"#,
        r#"/.(?<=\1|(a))./"#,
        // into a negative lookaround
        r#"/a(?!(b)).\1/"#,
        r#"/(?<!(a))b\1/"#,
        r#"/(?<!(a))(?:\1)/"#,
        r#"/.(?<!a|(b)).\1/"#,
        r#"/.(?!(a)).(?!\1)./"#,
        r#"/.(?<!(a)).(?<!\1)./"#,
        r#"/.(?=(?!(a))\1)./"#,
        r#"/.(?<!\1(?!(a)))/"#,
        // valid and invalid
        r#"/\1(a)(b)\2/"#,
        r#"/\1(a)\1/"#,
        // multiple invalid
        r#"/\1(a)\2(b)/"#,
        r#"/\1.(?<=(a)\1)/"#,
        r#"/(?!\1(a)).\1/"#,
        r#"/(a)\2(b)/; RegExp('(\\1)');"#,
        // when flags cannot be evaluated, it is assumed that the regex doesn't have 'u' flag set, so this will be a correct regex with a useless backreference
        r#"RegExp('\\1(a){', flags);"#,
        r#"const r = RegExp, p = '\\1', s = '(a)'; new r(p + s);"#,
        // ES2024
        r#"new RegExp('\\1([[A--B]])', 'v')"#,
        // ES2025
        r#"/\k<foo>((?<foo>bar)|(?<foo>baz))/"#,
        r#"/((?<foo>bar)|\k<foo>(?<foo>baz))/"#,
        r#"/\k<foo>((?<foo>bar)|(?<foo>baz)|(?<foo>qux))/"#,
        r#"/((?<foo>bar)|\k<foo>(?<foo>baz)|(?<foo>qux))/"#,
        r#"/((?<foo>bar)|\k<foo>|(?<foo>baz))/"#,
        r#"/((?<foo>bar)|\k<foo>|(?<foo>baz)|(?<foo>qux))/"#,
        r#"/((?<foo>bar)|(?<foo>baz\k<foo>)|(?<foo>qux\k<foo>))/"#,
        r#"/(?<=((?<foo>bar)|(?<foo>baz))\k<foo>)/"#,
        r#"/((?!(?<foo>bar))|(?!(?<foo>baz)))\k<foo>/"#,
    ];

    Tester::new(NoUselessBackreference::NAME, pass, fail).test();
}
