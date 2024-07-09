use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::{
    ast::{Argument, CallExpression, NewExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use std::{collections::HashSet, iter::Peekable};

#[derive(Debug, Default, Clone)]
pub struct NoUselessBackreference;

enum RegexGroup {
    RegexCaptureGroup(),
    RegexNamedCaptureGroup(String),
    RegexNonCaptureGroup(),
    RegexLookAheadGroup(),
    RegexLookBehindGroup(),
}

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

fn get_name_reference(char: char, iter: &mut Peekable<std::str::Chars<'_>>) -> String {
    assert!(char == '<');

    let mut group_name = String::new();
    while let Some(char) = iter.next() {
        // ToDO: backslash?
        if char == '>' {
            break;
        }

        group_name.push(char);
    }

    return group_name;
}
/**
 * get the Regex Group Type, char must be "(".
 *
 * it will move the iterator to the start of the regex group expression
 *
 * Lookaheads assertion: (?=...), (?!...)
 * Lookbehind assertion: (?<=...), (?<!...)
 * Non-capturing group: (?:...)
 * Capture group: (...)
 */
fn get_group_type_by_open_bracet(
    char: char,
    iter: &mut Peekable<std::str::Chars<'_>>,
) -> RegexGroup {
    assert!(char == '(');

    let next_char = iter.peek();

    match next_char {
        Some(next_char) => {
            if next_char.to_owned() == '?' {
                iter.next(); // pointer is now on ?
                let next_char = iter.peek();

                match next_char {
                    Some(next_char) => {
                        let owned_next_char = next_char.to_owned();

                        if owned_next_char == '=' || owned_next_char == '!' {
                            return RegexGroup::RegexLookAheadGroup();
                        }

                        if owned_next_char == ':' {
                            return RegexGroup::RegexNonCaptureGroup();
                        }

                        if owned_next_char == '<' {
                            iter.next(); // pointer is now on <
                            let next_char = iter.peek();

                            match next_char {
                                Some(next_char) => {
                                    let owned_char = next_char.to_owned();

                                    if owned_char == '=' || owned_char == '!' {
                                        return RegexGroup::RegexLookBehindGroup();
                                    }
                                }
                                _ => {}
                            }

                            // we already called iter.next()
                            let group_name = get_name_reference(owned_next_char, iter);

                            return RegexGroup::RegexNamedCaptureGroup(group_name);
                        }
                    }
                    _ => {}
                };
            }
        }
        _ => {}
    }

    return RegexGroup::RegexCaptureGroup();
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

        if !backslash_started && char == '(' {
            let group_type = get_group_type_by_open_bracet(char, &mut chars);

            match group_type {
                RegexGroup::RegexCaptureGroup() => {
                    count += 1;
                }
                _ => {}
            }
        }

        if char != '\\' {
            backslash_started = false;
        }
    }

    return count;
}

fn has_invalid_back_reference(regex: &str) -> bool {
    let mut chars = regex.chars().peekable();
    let mut backslash_started = false;

    // ToDO: can be all 8, atm for just simple programming
    let mut inside_character_class_count: u32 = 0;
    let mut group_started: HashSet<usize> = HashSet::new();
    let mut group_finished: HashSet<usize> = HashSet::new();
    let mut named_group_started: HashSet<String> = HashSet::new();
    let mut named_group_finished: HashSet<String> = HashSet::new();
    let captures_groups_count: u32 = get_capture_group_count(regex);

    // fast accept: no captures groups? no back references!
    if captures_groups_count == 0 {
        return false;
    }

    while let Some(char) = chars.next() {
        // check for backslash
        if char == '\\' {
            backslash_started = !backslash_started;
            continue;
        }

        if !backslash_started {
            match char {
                '[' => {
                    inside_character_class_count += 1;
                }
                ']' => {
                    inside_character_class_count -= 1;
                }
                '(' => {
                    // pointer can be moved!
                    let group_type = get_group_type_by_open_bracet(char, &mut chars);

                    match group_type {
                        RegexGroup::RegexNonCaptureGroup() => {}
                        RegexGroup::RegexLookBehindGroup() => {}
                        RegexGroup::RegexLookAheadGroup() => {}
                        RegexGroup::RegexNamedCaptureGroup(name) => {
                            named_group_started.insert(name);
                        }
                        RegexGroup::RegexCaptureGroup() => {
                            // ToDo: maybe a counter? before trying to access an hash len
                            group_started.insert(group_started.len());
                        }
                    }
                }
                ')' => {
                    group_finished.insert(group_started.len());
                }

                // ToDo: disallow for alternative routes
                '|' => {}
                _ => {}
            }
        }
        // starts with a backlash followed by a positive number or an k for named backreference
        // not inside a character class (e. : [abc])
        else if inside_character_class_count == 0 && (char != '0' && char.is_ascii_digit()) {
            let next_char = chars.peek();
            let digit_result: u32; // ToDo: u8, can be only max 99

            match next_char {
                Some(next_char) => {
                    // next char is a digit, =>  9 > final number < 100
                    if next_char.is_ascii_digit() {
                        digit_result = format!("{}{}", char, next_char).parse::<u32>().unwrap();
                    } else {
                        digit_result = char.to_digit(10).unwrap();
                    }
                }
                None => {
                    digit_result = char.to_digit(10).unwrap();
                }
            }

            println!(
                "digits {} {}, {} {:?}",
                regex, digit_result, captures_groups_count, group_finished
            );

            // we are trying to access a capture group and not an octal
            if digit_result <= captures_groups_count {
                // this capture group did not end
                let digit_result_usize = digit_result as usize;

                if !group_finished.contains(&digit_result_usize) {
                    return true;
                }
            }
        } else if inside_character_class_count == 0 && char == '<' {
            let group_name = get_name_reference(char, &mut chars);

            if !named_group_finished.contains(&group_name) {
                return true;
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
                let regex: &Argument = &expr.arguments[0];

                match regex {
                    Argument::TemplateLiteral(args) => {
                        if args.expressions.len() == 0 && args.quasis.len() == 1 {
                            // let template_value = args.quasis[0].value.cooked.unwrap();

                            // if has_invalid_back_reference(&template_value) {
                            //     ctx.diagnostic(OxcDiagnostic::warn("no back reference").with_label(args.quasis[0].span))
                            // }
                        }
                    }
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
        // unknown pattern
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
        r#"RegExp('\1(a)')"#,        // string octal escape
        r#"RegExp('\\\\1(a)')"#,     // escaped backslash
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
        r#"/(?<!\1(a))b/"#,
        r#"/(?<=\1(a))b/"#,
        r#"/(?<!\1.(a))b/"#,
        r#"/(?<=\1.(a))b/"#,
        r#"/(?<=(?:\1.(a)))b/"#,
        r#"/(?<!(?:\1)((a)))b/"#,
        r#"/(?<!(?:\2)((a)))b/"#,
        r#"/(?=(?<=\1(a)))b/"#,
        r#"/(?=(?<!\1(a)))b/"#,
        r#"/(.)(?<=\2(a))b/"#,
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
        r#"/a(?<!\1(b))./"#,
        r#"/a(?!(b)(\1))./"#,
        r#"/a(?!(?:(b)\1))./"#,
        r#"/a(?!(?:(b))\1)./"#,
        r#"/a(?<!(?:\1)(b))./"#,
        r#"/a(?<!(?:(?:\1)(b)))./"#,
        r#"/(?<!(a))(b)(?!(c))\2/"#,
        r#"/a(?!(b|c)\1)./"#,
        // ignore regular expressions with syntax errors
        // ToDo: Check if really needed
        // r#"RegExp('\\1(a)[')"#, // \1 would be an error, but the unterminated [ is a syntax error
        // r#"new RegExp('\\1(a){', 'u')"#, // \1 would be an error, but the unterminated { is a syntax error because of the 'u' flag
        // r#"new RegExp('\\1(a)\\2', 'ug')"#, // \1 would be an error, but \2 is syntax error because of the 'u' flag
        // r#"const flags = 'gus'; RegExp('\\1(a){', flags);"#, // \1 would be an error, but the rule is aware of the 'u' flag so this is a syntax error
        // r#"RegExp('\\1(a)\\k<foo>', 'u')"#, // \1 would be an error, but \k<foo> produces syntax error because of the u flag
        // r#"new RegExp('\\k<foo>(?<foo>a)\\k<bar>')"#, // \k<foo> would be an error, but \k<bar> produces syntax error because group <bar> doesn't exist

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
