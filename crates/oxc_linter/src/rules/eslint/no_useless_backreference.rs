use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::{
    ast::{Argument, CallExpression, NewExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use std::iter::Peekable;

#[derive(Debug, Default, Clone)]
pub struct NoUselessBackreference;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq)]
enum RegexGroup {
    CaptureGroup(u8),
    NamedCaptureGroup(String),
    NonCaptureGroup(),
    LookAheadGroup(),
    LookBehindGroup(),
}

#[derive(Debug, Clone)]
struct RegexBackReference {
    span: Span,
    regex_group: RegexGroup,
}

#[derive(Debug)]
enum LookBehindReference {
    BackReference(RegexBackReference),
    RegexGroup(RegexGroup),
}

#[derive(Debug)]
struct LookBehindContext {
    inside_counter: u8,
    timeline: Vec<LookBehindReference>,
}

#[derive(Debug, Clone)]
struct CaptureGroupContext<'a> {
    iterator: Peekable<std::str::Chars<'a>>,
    current_counter: u8,
    finished_groups: Vec<RegexGroup>,
    all_groups: Vec<RegexGroup>,
    current_index: u32,
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

fn get_name_reference(context: &mut CaptureGroupContext) -> String {
    let mut group_name = String::new();

    for char in context.iterator.by_ref() {
        context.current_index += 1;

        // ToDO: backslash?
        if char == '>' {
            break;
        }

        group_name.push(char);
    }

    group_name
}

fn get_regex_backreference(
    char: char,
    context: &mut CaptureGroupContext,
    capture_group_count: u8,
) -> Result<RegexBackReference, bool> {
    if char != '0' && char.is_ascii_digit() {
        let digit_result: u8; // can be only max 99
        let span_start: u32 = context.current_index - 1; // backslash is the start
        let mut span_end: u32 = context.current_index + 1;

        if let Some(next_char) = context.iterator.peek() {
            // next char is a digit, =>  9 > final number < 100
            // TODO: why format :/
            if next_char.is_ascii_digit() {
                digit_result = format!("{char}{next_char}").parse::<u8>().unwrap();
                span_end += 1;
            } else {
                digit_result = format!("{char}").parse::<u8>().unwrap();
            }
        } else {
            digit_result = format!("{char}").parse::<u8>().unwrap();
        }

        // we are trying to access a capture group and not an octal
        if digit_result <= capture_group_count {
            return Ok(RegexBackReference {
                span: Span::new(span_start, span_end),
                regex_group: RegexGroup::CaptureGroup(digit_result),
            });
        }
    }

    if char == 'k' {
        if let Some(next_char) = context.iterator.peek() {
            if *next_char == '<' {
                let span_start: u32 = context.current_index - 1; // backslash is the start

                context.iterator.next();
                context.current_index += 1;

                let group_name = get_name_reference(context);
                let span_end: u32 = context.current_index + 1;

                return Ok(RegexBackReference {
                    span: Span::new(span_start, span_end),
                    regex_group: RegexGroup::NamedCaptureGroup(group_name),
                });
            }
        }
    }

    Err(false)
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
fn get_regex_group_by_open_bracet(context: &mut CaptureGroupContext) -> RegexGroup {
    if let Some(next_char) = context.iterator.peek() {
        if *next_char == '?' {
            context.iterator.next(); // pointer is now on ?
            context.current_index += 1;

            if let Some(next_char) = context.iterator.peek() {
                if *next_char == '=' || *next_char == '!' {
                    return RegexGroup::LookAheadGroup();
                }

                if *next_char == ':' {
                    return RegexGroup::NonCaptureGroup();
                }

                if *next_char == '<' {
                    context.iterator.next(); // pointer is now on <
                    context.current_index += 1;

                    if let Some(next_char) = context.iterator.peek() {
                        if *next_char == '=' || *next_char == '!' {
                            return RegexGroup::LookBehindGroup();
                        }

                        let group_name = get_name_reference(context);

                        return RegexGroup::NamedCaptureGroup(group_name);
                    }
                }
            }
        }
    }

    RegexGroup::CaptureGroup(context.current_counter)
}

fn get_regex_groups(context: &mut CaptureGroupContext, with_alternatives: bool) -> Vec<RegexGroup> {
    let mut backslash_started = false;
    let mut inside_group: u8 = 0;
    let mut inside_character_class_count: u8 = 0;
    let mut result: Vec<RegexGroup> = vec![];

    while let Some(char) = context.iterator.next() {
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
                    let group_type = get_regex_group_by_open_bracet(context);

                    inside_group += 1;

                    match group_type {
                        RegexGroup::CaptureGroup(_) => {
                            context.current_counter += 1;
                        }
                        RegexGroup::NamedCaptureGroup(_) => {
                            result.push(RegexGroup::CaptureGroup(context.current_counter));
                            context.current_counter += 1;
                        }
                        _ => {}
                    }

                    result.push(group_type);
                }
                ')' => {
                    // we are in another alternative, let parent process do closing stuff,
                    // example: (a|b)
                    //              ^
                    if inside_group == 0 {
                        return result;
                    }
                    inside_group -= 1;
                }
                '|' if inside_character_class_count == 0 => {
                    if with_alternatives {
                        let mut sub_results = get_regex_groups(context, true);
                        result.append(&mut sub_results);
                    } else {
                        return result;
                    }
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

fn handle_closing_bracet(
    context: &mut CaptureGroupContext,
    open_groups: &mut Vec<RegexGroup>,
    look_behind_context: &mut LookBehindContext,
    inside_look_ahead: &mut u8,
) -> Result<Span, bool> {
    if let Some(group_type) = open_groups.pop() {
        match group_type {
            RegexGroup::LookBehindGroup() => {
                look_behind_context.inside_counter -= 1;

                println!(
                    "
                    context: {context:?}
                    look_behind_context: {look_behind_context:?}
                "
                );

                if look_behind_context.inside_counter == 0 {
                    let mut found_groups: Vec<RegexGroup> = vec![];

                    for lookbehind_reference in &look_behind_context.timeline {
                        match lookbehind_reference {
                            LookBehindReference::RegexGroup(regex_group) => {
                                // we dont capture any useless groups for this check
                                found_groups.push(regex_group.to_owned());
                            }
                            LookBehindReference::BackReference(reference) => {
                                if found_groups.contains(&reference.regex_group) {
                                    return Ok(reference.span);
                                }
                            }
                        }
                    }

                    // look_behind_context.timeline.clear();
                }
            }
            RegexGroup::LookAheadGroup() => {
                *inside_look_ahead -= 1;
            }
            RegexGroup::CaptureGroup(_) | RegexGroup::NamedCaptureGroup(_) => {
                context.finished_groups.push(group_type);
            }
            RegexGroup::NonCaptureGroup() => {}
        }

        return Err(true);
    }
    // we found a closing group of a parent regex
    // this can happen when the parent group has an alternative

    // println!("closing parent found inside )");
    Err(false)
}

fn report_invalid_back_reference(regex: &str, span_start_index: u32, ctx: &LintContext) {
    if let Ok(span) = get_string_invalid_back_reference(regex, span_start_index) {
        ctx.diagnostic(OxcDiagnostic::warn("no back reference").with_label(span));
    }
}

fn get_string_invalid_back_reference(regex: &str, span_start_index: u32) -> Result<Span, bool> {
    let chars = regex.chars();
    let peek = chars.peekable();

    let sub_context = &mut CaptureGroupContext {
        iterator: peek.clone(),
        current_counter: 1,
        finished_groups: vec![],
        all_groups: vec![],
        current_index: span_start_index,
    };

    // println!("regex: {regex}");

    get_peekable_invalid_back_references(&mut CaptureGroupContext {
        iterator: peek,
        current_counter: 1,
        finished_groups: vec![],
        all_groups: get_regex_groups(sub_context, true),
        current_index: span_start_index,
    })
}

fn get_peekable_invalid_back_references(context: &mut CaptureGroupContext) -> Result<Span, bool> {
    let capture_group_count = u8::try_from(
        context
            .all_groups
            .iter()
            .filter_map(|group| match group {
                RegexGroup::CaptureGroup(_) | RegexGroup::NamedCaptureGroup(_) => Some(true),
                _ => None,
            })
            .collect::<Vec<_>>()
            .len(),
    )
    .unwrap();

    let mut backslash_started = false;
    let mut open_groups: Vec<RegexGroup> = vec![];
    let mut inside_character_class_count: u8 = 0;
    let mut inside_look_ahead: u8 = 0;
    let current_found_group: Vec<RegexGroup> = get_regex_groups(&mut context.clone(), false);

    let mut look_behind_context = LookBehindContext { inside_counter: 0, timeline: vec![] };

    // println!("context: {context:?}");

    while let Some(char) = context.iterator.next() {
        // println!("{char}");
        context.current_index += 1;

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
                    let group_type = get_regex_group_by_open_bracet(context);

                    match group_type {
                        RegexGroup::LookBehindGroup() => {
                            look_behind_context.inside_counter += 1;
                        }
                        RegexGroup::LookAheadGroup() => {
                            inside_look_ahead += 1;
                        }
                        RegexGroup::CaptureGroup(_) | RegexGroup::NamedCaptureGroup(_) => {
                            context.current_counter += 1;

                            if look_behind_context.inside_counter != 0 {
                                look_behind_context
                                    .timeline
                                    .push(LookBehindReference::RegexGroup(group_type.clone()));
                            }
                        }
                        RegexGroup::NonCaptureGroup() => {}
                    }

                    open_groups.push(group_type);
                }
                ')' => {
                    let result = handle_closing_bracet(
                        context,
                        &mut open_groups,
                        &mut look_behind_context,
                        &mut inside_look_ahead,
                    );

                    if let Ok(span) = result {
                        return Ok(span);
                    } else if !result.err().unwrap() {
                        return Err(false);
                    }
                }

                '|' if inside_character_class_count == 0 => {
                    // println!(
                    //     "alternative
                    //     context:  {context:?}
                    //     inside_look_behind: {inside_look_behind}
                    //     open_groups: {open_groups:?}"
                    // );

                    if open_groups.is_empty() {
                        return get_peekable_invalid_back_references(context);
                    }

                    // we are iterating until we find the closing group
                    let result = get_peekable_invalid_back_references(context);

                    if result.is_ok() {
                        return result;
                    }

                    // the alternative did put the pointer to ")", we need to close the group now
                    let result = handle_closing_bracet(
                        context,
                        &mut open_groups,
                        &mut look_behind_context,
                        &mut inside_look_ahead,
                    );

                    if let Ok(span) = result {
                        return Ok(span);
                    } else if !result.err().unwrap() {
                        return Err(false);
                    }
                }
                _ => {}
            }
        }
        // starts with a backlash followed by a positive number or an k for named backreference
        // not inside a character class (example : [abc])
        else if inside_character_class_count == 0 {
            if let Ok(backreference) = get_regex_backreference(char, context, capture_group_count) {
                println!(
                    "backreference {backreference:?}
                    capture_group_count: {capture_group_count}
                    context: {context:?}
                    look_behind_context: {look_behind_context:?}
                    _inside_look_ahead: {inside_look_ahead}
                    open_groups: {open_groups:?}
                    current_found_group: {current_found_group:?}",
                );

                if look_behind_context.inside_counter != 0 {
                    look_behind_context
                        .timeline
                        .push(LookBehindReference::BackReference(backreference.clone()));
                }

                // not inside look behind: ToDo: check für better
                // the group needs to be finished and not currently open
                // cant be open
                if open_groups.contains(&backreference.regex_group) {
                    return Ok(backreference.span);
                // needs to be found in this alternative
                } else if !current_found_group.contains(&backreference.regex_group) {
                    return Ok(backreference.span);
                // there is a reference but we dont know from where
                } else if !context.finished_groups.contains(&backreference.regex_group)
                    && look_behind_context.inside_counter == 0
                {
                    return Ok(backreference.span);
                }
            }
        }

        if char != '\\' {
            backslash_started = false;
        }
    }

    Err(false)
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
            AstKind::RegExpLiteral(literal) => {
                report_invalid_back_reference(&literal.regex.pattern, literal.span.start, ctx);
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
                    Argument::StringLiteral(arg) => {
                        report_invalid_back_reference(&arg.value, arg.span.start, ctx);
                    }
                    _ => {}
                };
            }
            AstKind::CallExpression(expr) if is_regexp_call_expression(expr) => {
                let regex = &expr.arguments[0];

                if let Argument::StringLiteral(arg) = regex {
                    report_invalid_back_reference(&arg.value, arg.span.start, ctx);
                }
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
        // r"/\k<foo>(a)/", // without the 'u' flag and any named groups this isn't a syntax error, matches "k<foo>a"
        r"/^(a)\1\2$/", // \1 is a backreference, \2 is an octal escape sequence.
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
        // r"/(?<=(?=(a)\1))b/",
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
        r"/\k<foo>(?<foo>bar)/",
        // r"RegExp('(a|bc)|\\1')", -- ToDo: why is this invalid?
        r"new RegExp('(?!(?<foo>\\n))\\1')",
        // r"/(?<!(a)\1)b/",  -- ToDo
        // nested
        r"new RegExp('(\\1)')",
        r"/^(a\1)$/",
        r"/^((a)\1)$/",
        r"new RegExp('^(a\\1b)$')",
        r"RegExp('^((\\1))$')",
        r"/((\2))/",
        r"/a(?<foo>(.)b\1)/",
        r"/a(?<foo>\k<foo>)b/",
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
        r"/\k<foo>(?<foo>a)/",
        r"/(?:a(b)\2)(c)/",
        r"new RegExp('(a)(b)\\3(c)')",
        r"/\1(?<=(a))./",
        r"/\1(?<!(a))./",
        // r"/(?<=\1)(?<=(a))/",
        // r"/(?<!\1)(?<!(a))/",
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
        // r"/a(?!(b)).\1/",
        // r"/(?<!(a))b\1/",
        // r"/(?<!(a))(?:\1)/",
        // r"/.(?<!a|(b)).\1/",
        // r"/.(?!(a)).(?!\1)./",
        // r"/.(?<!(a)).(?<!\1)./",
        // r"/.(?=(?!(a))\1)./",
        // r"/.(?<!\1(?!(a)))/",
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
        // r"const r = RegExp, p = '\\1', s = '(a)'; new r(p + s);",
        // ES2024
        r"new RegExp('\\1([[A--B]])', 'v')",
        // ES2025
        // r"/\k<foo>((?<foo>bar)|(?<foo>baz))/",
        // r"/((?<foo>bar)|\k<foo>(?<foo>baz))/",
        r"/\k<foo>((?<foo>bar)|(?<foo>baz)|(?<foo>qux))/",
        // r"/((?<foo>bar)|\k<foo>(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|(?<foo>baz\k<foo>)|(?<foo>qux\k<foo>))/",
        // r"/(?<=((?<foo>bar)|(?<foo>baz))\k<foo>)/",
        // r"/((?!(?<foo>bar))|(?!(?<foo>baz)))\k<foo>/",
    ];

    Tester::new(NoUselessBackreference::NAME, pass, fail).test_and_snapshot();
}
