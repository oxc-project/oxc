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

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum RegexGroup {
    CaptureGroup(u8),
    NamedCaptureGroup(u8, String),
    NonCaptureGroup(u8),
    LookAheadGroup(u8),
    LookBehindGroup(u8),
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
    for char in iter.by_ref() {
        // ToDO: backslash?
        if char == '>' {
            break;
        }

        group_name.push(char);
    }

    group_name
}

/**
 * get the Regex Group Type, char must be "(".
 *
 * it will move the iterator to the start of the regex group expression
 *
 * Lookaheads assertion: (?=...), (?!...)
 * Lookbehind assertion: (?<=...), (?<!...)
 * Non-capturing group: (?:...)
 * Named capture group: (<foo>...)
 * Capture group: (...)
 */
fn get_regex_group_by_open_bracet(
    char: char,
    iter: &mut Peekable<std::str::Chars<'_>>,
    group_counter: u8,
) -> RegexGroup {
    assert!(char == '(');

    if let Some(next_char) = iter.peek() {
        if *next_char == '?' {
            iter.next(); // pointer is now on ?

            if let Some(next_char) = iter.peek() {
                let owned_next_char = next_char.to_owned();

                if owned_next_char == '=' || owned_next_char == '!' {
                    return RegexGroup::LookAheadGroup(group_counter);
                }

                if owned_next_char == ':' {
                    return RegexGroup::NonCaptureGroup(group_counter);
                }

                if owned_next_char == '<' {
                    iter.next(); // pointer is now on <

                    if let Some(next_char) = iter.peek() {
                        let owned_char = next_char.to_owned();

                        if owned_char == '=' || owned_char == '!' {
                            return RegexGroup::LookBehindGroup(group_counter);
                        }

                        let group_name = get_name_reference(owned_next_char, iter);

                        return RegexGroup::NamedCaptureGroup(group_counter, group_name);
                    }
                }
            }
        }
    }

    RegexGroup::CaptureGroup(group_counter)
}

fn get_capture_groups(
    chars: &mut Peekable<std::str::Chars<'_>>,
    group_count: u8,
) -> Vec<RegexGroup> {
    let mut backslash_started = false;
    let mut alternative_found = false;
    let mut inside_group: u8 = 0;
    let mut inside_character_class_count: u8 = 0;
    let mut result: Vec<RegexGroup> = vec![];

    while let Some(char) = chars.next() {
        if alternative_found {
            break;
        }

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
                    let group_type = get_regex_group_by_open_bracet(char, chars, group_count);
                    result.push(group_type);

                    inside_group += 1;
                }
                ')' => {
                    inside_group -= 1;
                }
                '|' if inside_group == 0 && inside_character_class_count == 0 => {
                    alternative_found = true;
                }
                _ => {}
            }
        }

        if char != '\\' {
            backslash_started = false;
        }
    }

    result
}

fn has_string_invalid_back_reference(regex: &str) -> bool {
    let mut chars = regex.chars().peekable();

    println!("regex: {regex}");
    has_peekable_invalid_back_reference(&mut chars, 0)
}

fn has_peekable_invalid_back_reference(
    chars: &mut Peekable<std::str::Chars<'_>>,
    capture_group_start_count: u8,
) -> bool {
    let mut cloned_chars: Peekable<std::str::Chars> = chars.clone();
    let all_groups = get_capture_groups(&mut cloned_chars, capture_group_start_count);
    let capture_group_count = u8::try_from(
        all_groups
            .iter()
            .filter_map(|group| match group {
                RegexGroup::CaptureGroup(i) => Some(i),
                _ => None,
            })
            .collect::<Vec<_>>()
            .len(),
    )
    .unwrap();

    let mut backslash_started = false;

    let mut open_groups: Vec<RegexGroup> = vec![];

    // ToDo: can be all 8, atm for just simple programming
    let mut inside_character_class_count: u8 = 0;
    let mut inside_look_behind: u8 = 0;
    let mut inside_look_ahead: u8 = 0;
    let mut capture_group_started: Vec<u8>;

    if capture_group_count == 0 {
        capture_group_started = vec![];
    } else {
        capture_group_started = (0..capture_group_start_count).collect();
    }

    // ToDo: we can remove the entry
    let mut capture_group_finished = capture_group_started.clone();

    while let Some(char) = chars.next() {
        println!("{char}");
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
                    let group_type =
                        get_regex_group_by_open_bracet(char, chars, capture_group_count);

                    open_groups.push(group_type.clone());

                    match group_type {
                        RegexGroup::LookBehindGroup(_) => {
                            inside_look_behind += 1;
                        }
                        RegexGroup::LookAheadGroup(_) => {
                            inside_look_ahead += 1;
                        }
                        RegexGroup::CaptureGroup(i) => {
                            // ToDo: maybe a counter? before trying to access an hash len
                            capture_group_started
                                .push(u8::try_from(capture_group_started.len()).unwrap());
                        }
                        _ => {}
                    }
                }
                ')' => {
                    if let Some(group_type) = open_groups.pop() {
                        match group_type {
                            RegexGroup::LookBehindGroup(_) => {
                                inside_look_behind = inside_look_behind.saturating_sub(1);
                            }
                            RegexGroup::LookAheadGroup(_) => {
                                inside_look_ahead = inside_look_behind.saturating_sub(1);
                            }
                            RegexGroup::CaptureGroup(_) => {
                                capture_group_finished
                                    .push(u8::try_from(capture_group_started.len()).unwrap());
                            }
                            _ => {}
                        }
                    }
                }

                '|' => {
                    println!(
                        "alternative {:?}, {:?}, {inside_character_class_count}, {inside_look_behind}",
                        capture_group_started,
                        capture_group_finished,
                    );

                    if open_groups.is_empty() && inside_character_class_count == 0 {
                        return has_peekable_invalid_back_reference(chars, capture_group_count);
                    }
                }
                _ => {}
            }
        }
        // starts with a backlash followed by a positive number or an k for named backreference
        // not inside a character class (example : [abc])
        else if inside_character_class_count == 0 && char != '0' && char.is_ascii_digit()
        {
            let digit_result: u8; // ToDo: u8, can be only max 99

            if let Some(next_char) = chars.peek() {
                // next char is a digit, =>  9 > final number < 100
                // TODO: why format :/
                if next_char.is_ascii_digit() {
                    digit_result = format!("{char}{next_char}").parse::<u8>().unwrap();
                } else {
                    digit_result = format!("{char}").parse::<u8>().unwrap();
                }
            } else {
                digit_result = format!("{char}").parse::<u8>().unwrap();
            }

            println!(
                "backreference {digit_result}, {capture_group_count}, {:?}, {:?}, {capture_group_start_count}, {inside_look_behind}, {inside_look_ahead}",
                capture_group_finished,
                open_groups,
            );

            // we are trying to access a capture group and not an octal
            if digit_result <= capture_group_count {
                // this capture group did not end

                // capture group did not finished in this regex, but maybe one
                if inside_look_behind == 0 && !capture_group_finished.contains(&digit_result) {
                    // not in look behind, so this in invalid
                    return true;
                }
            }
            // are there more future groups?
            else if inside_look_ahead != 0 && digit_result > capture_group_count {
                // return true;
            }

        } else if inside_character_class_count == 0 && char == '<' {
            // let group_name = get_name_reference(char,  chars);
            //
            // if !named_group_finished.contains(&group_name) {
            //     return true;
            // }
        }

        if char != '\\' {
            backslash_started = false;
        }
    }

    false
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
                if has_string_invalid_back_reference(&literal.regex.pattern) =>
            {
                ctx.diagnostic(OxcDiagnostic::warn("no back reference").with_label(literal.span));
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
                    Argument::StringLiteral(arg)
                        if has_string_invalid_back_reference(&arg.value) =>
                    {
                        ctx.diagnostic(
                            OxcDiagnostic::warn("no back reference").with_label(arg.span),
                        );
                    }
                    _ => {}
                };
            }
            AstKind::CallExpression(expr) if is_regexp_call_expression(expr) => {
                let regex = &expr.arguments[0];

                match regex {
                    Argument::StringLiteral(arg)
                        if has_string_invalid_back_reference(&arg.value) =>
                    {
                        ctx.diagnostic(
                            OxcDiagnostic::warn("no back reference").with_label(arg.span),
                        );
                    }
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
        r"'\1(a)'",
        r"regExp('\\1(a)')",
        r"new Regexp('\\1(a)', 'u')",
        r"RegExp.foo('\\1(a)', 'u')",
        r"new foo.RegExp('\\1(a)')",
        // unknown pattern
        r"RegExp(p)",
        r"new RegExp(p, 'u')",
        r"RegExp('\\1(a)' + suffix)",
        // r#"new RegExp(`${prefix}\\\\1(a)"#,

        // not the global RegExp
        // ToDo: Do we really need this checks?
        // r#"let RegExp; new RegExp('\\1(a)');"#,
        // r#"function foo() { var RegExp; RegExp('\\1(a)', 'u'); }"#,
        // r#"function foo(RegExp) { new RegExp('\\1(a)'); }"#,
        // r#"if (foo) { const RegExp = bar; RegExp('\\1(a)'); }"#,

        // no capturing groups
        r"/(?:)/",
        r"/(?:a)/",
        r"new RegExp('')",
        r"RegExp('(?:a)|(?:b)*')",
        r"/^ab|[cd].\n$/",
        // no backreferences
        r"/(a)/",
        r"RegExp('(a)|(b)')",
        r"new RegExp('\\n\\d(a)')",
        r"/\0(a)/",
        r"/\0(a)/u",
        r"/(?<=(a))(b)(?=(c))/",
        r"/(?<!(a))(b)(?!(c))/",
        r"/(?<foo>a)/",
        // not really a backreference
        r"RegExp('\1(a)')",        // string octal escape
        r"RegExp('\\\\1(a)')",     // escaped backslash
        r"/\\1(a)/",               // escaped backslash
        r"/\1/",                   // group 1 doesn't exist, this is a regex octal escape
        r"/^\1$/",                 // group 1 doesn't exist, this is a regex octal escape
        r"/\2(a)/",                // group 2 doesn't exist, this is a regex octal escape
        r"/\1(?:a)/",              // group 1 doesn't exist, this is a regex octal escape
        r"/\1(?=a)/",              // group 1 doesn't exist, this is a regex octal escape
        r"/\1(?!a)/",              // group 1 doesn't exist, this is a regex octal escape
        r"/^[\1](a)$/",            // \N in a character class is a regex octal escape
        r"new RegExp('[\\1](a)')", // \N in a character class is a regex octal escape
        r"/\11(a)/",               // regex octal escape \11, regex matches "\x09a"
        r"/\k<foo>(a)/", // without the 'u' flag and any named groups this isn't a syntax error, matches "k<foo>a"
        r"/^(a)\1\2$/",  // \1 is a backreference, \2 is an octal escape sequence.
        // Valid backreferences: correct position, after the group
        r"/(a)\1/",
        r"/(a).\1/",
        r"RegExp('(a)\\1(b)')",
        r"/(a)(b)\2(c)/",
        r"/(?<foo>a)\k<foo>/",
        r"new RegExp('(.)\\1')",
        r"RegExp('(a)\\1(?:b)')",
        r"/(a)b\1/",
        r"/((a)\2)/",
        r"/((a)b\2c)/",
        r"/^(?:(a)\1)$/",
        r"/^((a)\2)$/",
        r"/^(((a)\3))|b$/",
        r"/a(?<foo>(.)b\2)/",
        r"/(a)?(b)*(\1)(c)/",
        r"/(a)?(b)*(\2)(c)/",
        r"/(?<=(a))b\1/",
        r"/(?<=(?=(a)\1))b/",
        // Valid backreferences: correct position before the group when they're both in the same lookbehind
        r"/(?<!\1(a))b/",
        r"/(?<=\1(a))b/",
        r"/(?<!\1.(a))b/",
        r"/(?<=\1.(a))b/",
        r"/(?<=(?:\1.(a)))b/",
        r"/(?<!(?:\1)((a)))b/",
        r"/(?<!(?:\2)((a)))b/",
        r"/(?=(?<=\1(a)))b/",
        r"/(?=(?<!\1(a)))b/",
        r"/(.)(?<=\2(a))b/",
        // Valid backreferences: not a reference into another alternative
        r"/^(a)\1|b/",
        r"/^a|(b)\1/",
        r"/^a|(b|c)\1/",
        r"/^(a)|(b)\2/",
        r"/^(?:(a)|(b)\2)$/",
        r"/^a|(?:.|(b)\1)/",
        r"/^a|(?:.|(b).(\1))/",
        r"/^a|(?:.|(?:(b)).(\1))/",
        r"/^a|(?:.|(?:(b)|c).(\1))/",
        r"/^a|(?:.|(?:(b)).(\1|c))/",
        r"/^a|(?:.|(?:(b)|c).(\1|d))/",
        // Valid backreferences: not a reference into a negative lookaround (reference from within the same lookaround is allowed)
        r"/.(?=(b))\1/",
        r"/.(?<=(b))\1/",
        r"/a(?!(b)\1)./",
        r"/a(?<!\1(b))./",
        r"/a(?!(b)(\1))./",
        r"/a(?!(?:(b)\1))./",
        r"/a(?!(?:(b))\1)./",
        r"/a(?<!(?:\1)(b))./",
        r"/a(?<!(?:(?:\1)(b)))./",
        r"/(?<!(a))(b)(?!(c))\2/",
        r"/a(?!(b|c)\1)./",
        // ignore regular expressions with syntax errors
        // ToDo: Implement u flag check
        // r#"RegExp('\\1(a)[')"#, // \1 would be an error, but the unterminated [ is a syntax error
        // r#"new RegExp('\\1(a){', 'u')"#, // \1 would be an error, but the unterminated { is a syntax error because of the 'u' flag
        // r#"new RegExp('\\1(a)\\2', 'ug')"#, // \1 would be an error, but \2 is syntax error because of the 'u' flag
        // r#"const flags = 'gus'; RegExp('\\1(a){', flags);"#, // \1 would be an error, but the rule is aware of the 'u' flag so this is a syntax error
        // r#"RegExp('\\1(a)\\k<foo>', 'u')"#, // \1 would be an error, but \k<foo> produces syntax error because of the u flag
        // r#"new RegExp('\\k<foo>(?<foo>a)\\k<bar>')"#, // \k<foo> would be an error, but \k<bar> produces syntax error because group <bar> doesn't exist

        // ES2024
        r"new RegExp('([[A--B]])\\1', 'v')",
        r"new RegExp('[[]\\1](a)', 'v')", // SyntaxError
        // ES2025
        r"/((?<foo>bar)\k<foo>|(?<foo>baz))/",
    ];

    let fail = vec![
        r"/(b)(\2a)/",
        // r#"/\k<foo>(?<foo>bar)/"#,
        // r"RegExp('(a|bc)|\\1')",
        // r"new RegExp('(?!(?<foo>\\n))\\1')",
        // r"/(?<!(a)\1)b/",  -- ToDo
        // nested
        r"new RegExp('(\\1)')",
        r"/^(a\1)$/",
        r"/^((a)\1)$/",
        r"new RegExp('^(a\\1b)$')",
        r"RegExp('^((\\1))$')",
        r"/((\2))/",
        // r"/a(?<foo>(.)b\1)/",
        // r"/a(?<foo>\k<foo>)b/",
        r"/^(\1)*$/",
        r"/^(?:a)(?:((?:\1)))*$/",
        r"/(?!(\1))/",
        r"/a|(b\1c)/",
        r"/(a|(\1))/",
        r"/(a|(\2))/",
        r"/(?:a|(\1))/",
        r"/(a)?(b)*(\3)/",
        // r"/(?<=(a\1))b/", -- ToDo
        // forward
        r"/\1(a)/",
        r"/\1.(a)/",
        r"/(?:\1)(?:(a))/",
        r"/(?:\1)(?:((a)))/",
        r"/(?:\2)(?:((a)))/",
        r"/(?:\1)(?:((?:a)))/",
        r"/(\2)(a)/",
        r"RegExp('(a)\\2(b)')",
        r"/(?:a)(b)\2(c)/",
        // r"/\k<foo>(?<foo>a)/",
        r"/(?:a(b)\2)(c)/",
        r"new RegExp('(a)(b)\\3(c)')",
        r"/\1(?<=(a))./",
        r"/\1(?<!(a))./",
        r"/(?<=\1)(?<=(a))/",
        r"/(?<!\1)(?<!(a))/",
        r"/(?=\1(a))./",
        r"/(?!\1(a))./",
        // backward in the same lookbehind
        r"/(?<=(a)\1)b/",
        r"/(?<!.(a).\1.)b/",
        r"/(.)(?<!(b|c)\2)d/",
        r"/(?<=(?:(a)\1))b/",
        r"/(?<=(?:(a))\1)b/",
        r"/(?<=(a)(?:\1))b/",
        r"/(?<!(?:(a))(?:\1))b/",
        r"/(?<!(?:(a))(?:\1)|.)b/",
        r"/.(?!(?<!(a)\1))./",
        r"/.(?=(?<!(a)\1))./",
        r"/.(?!(?<=(a)\1))./",
        r"/.(?=(?<=(a)\1))./",
        // into another alternative
        r"/(a)|\1b/",
        r"/^(?:(a)|\1b)$/",
        r"/^(?:(a)|b(?:c|\1))$/",
        r"/^(?:a|b(?:(c)|\1))$/",
        r"/^(?:(a(?!b))|\1b)+$/",
        r"/^(?:(?:(a)(?!b))|\1b)+$/",
        r"/^(?:(a(?=a))|\1b)+$/",
        r"/^(?:(a)(?=a)|\1b)+$/",
        r"/.(?:a|(b)).|(?:(\1)|c)./",
        r"/.(?!(a)|\1)./",
        r"/.(?<=\1|(a))./",
        // into a negative lookaround
        r"/a(?!(b)).\1/",
        r"/(?<!(a))b\1/",
        r"/(?<!(a))(?:\1)/",
        r"/.(?<!a|(b)).\1/",
        r"/.(?!(a)).(?!\1)./",
        r"/.(?<!(a)).(?<!\1)./",
        r"/.(?=(?!(a))\1)./",
        r"/.(?<!\1(?!(a)))/",
        // valid and invalid
        r"/\1(a)(b)\2/",
        r"/\1(a)\1/",
        // multiple invalid
        r"/\1(a)\2(b)/",
        r"/\1.(?<=(a)\1)/",
        r"/(?!\1(a)).\1/",
        r"/(a)\2(b)/; RegExp('(\\1)');",
        // when flags cannot be evaluated, it is assumed that the regex doesn't have 'u' flag set, so this will be a correct regex with a useless backreference
        r"RegExp('\\1(a){', flags);",
        r"const r = RegExp, p = '\\1', s = '(a)'; new r(p + s);",
        // ES2024
        r"new RegExp('\\1([[A--B]])', 'v')",
        // ES2025
        r"/\k<foo>((?<foo>bar)|(?<foo>baz))/",
        r"/((?<foo>bar)|\k<foo>(?<foo>baz))/",
        r"/\k<foo>((?<foo>bar)|(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|\k<foo>(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|(?<foo>baz\k<foo>)|(?<foo>qux\k<foo>))/",
        r"/(?<=((?<foo>bar)|(?<foo>baz))\k<foo>)/",
        r"/((?!(?<foo>bar))|(?!(?<foo>baz)))\k<foo>/",
    ];

    Tester::new(NoUselessBackreference::NAME, pass, fail).test();
}
