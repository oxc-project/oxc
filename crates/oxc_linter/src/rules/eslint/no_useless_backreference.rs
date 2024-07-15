use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::{
    ast::{Argument, CallExpression, NewExpression, RegExpFlags},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use std::iter::Peekable;

#[derive(Debug, Default, Clone)]
pub struct NoUselessBackreference;

#[derive(Debug, Clone, PartialEq)]
enum RegexGroup {
    Capture(u8),          // u8 = the number identifier
    NamedCapture(String), // String = the name identifier, there is also always a number identifier
    NonCapture(),
    LookAhead(bool),  // positive or negative look ahead
    LookBehind(bool), // positive or negative look behind
}

#[derive(Debug, Clone, PartialEq)]
struct RegexBackReference {
    span: Span,
    regex_group: RegexGroup,
}

#[derive(Debug, PartialEq)]
enum LookGroupReference {
    BackReference(RegexBackReference),
    RegexGroup(RegexGroup),
}

#[derive(Debug, PartialEq)]
enum LookGroupDirection {
    Forward(),
    Backward(),
}

#[derive(Debug)]
struct LookGroupContext {
    direction: LookGroupDirection,
    inside_counter: u8,
    timeline: Vec<LookGroupReference>,
}

#[derive(Debug, Clone)]
struct CaptureGroupContext<'a> {
    // because we need to escape the escape in a js string, we need to check if the original regex was a string,
    // /(a)\1/ vs new Regex("(a)\\1")
    escaped_char_escaped: bool,

    // the current capture group counter
    // this will be always the next capture group we will finish
    current_counter: u8,
    finished_groups: Vec<RegexGroup>,

    // calling groups inside a negative looking group is forbidden
    inside_negative_count: u8,
    disallowed_groups: Vec<RegexGroup>,

    // the current span (start) index
    // ToDo: can be inside the iterator
    current_index: u32,

    iterator: Peekable<std::str::Chars<'a>>, // ToDO: check if this is also the ToDo item of the next line

    // ToDo: this never change but we are mutating the complete struct
    all_groups: &'a Vec<RegexGroup>,
    unicode_mode: bool,
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
    let span_start: u32 = if context.escaped_char_escaped {
        context.current_index - 2 // two backslash at the start
    } else {
        context.current_index - 1 // backslash at the start
    };

    if char != '0' && char.is_ascii_digit() {
        let digit_result: u8; // can be only max 99
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
                regex_group: RegexGroup::Capture(digit_result),
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
                    regex_group: RegexGroup::NamedCapture(group_name),
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
                    return RegexGroup::LookAhead(*next_char != '!');
                }

                if *next_char == ':' {
                    return RegexGroup::NonCapture();
                }

                if *next_char == '<' {
                    context.iterator.next(); // pointer is now on <
                    context.current_index += 1;

                    if let Some(next_char) = context.iterator.peek() {
                        if *next_char == '=' || *next_char == '!' {
                            return RegexGroup::LookBehind(*next_char != '!');
                        }

                        let group_name = get_name_reference(context);

                        return RegexGroup::NamedCapture(group_name);
                    }
                }
            }
        }
    }

    RegexGroup::Capture(context.current_counter)
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
                        RegexGroup::Capture(_) => {
                            context.current_counter += 1;
                        }
                        RegexGroup::NamedCapture(_) => {
                            result.push(RegexGroup::Capture(context.current_counter));
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

fn handle_look_group_closing(
    context: &mut CaptureGroupContext,
    group_context: &mut LookGroupContext,
    is_positive: bool,
) -> Result<Span, bool> {
    let mut found_groups: Vec<RegexGroup> = vec![];

    // println!(
    //     "
    //     context: {context:?}
    //     group_context: {group_context:?}
    //     is_positive: {is_positive}
    //     found_groups: {found_groups:?}
    // "
    // );

    for reference in &group_context.timeline {
        match reference {
            LookGroupReference::RegexGroup(regex_group) => {
                found_groups.push(regex_group.to_owned());

                if !is_positive {
                    context.disallowed_groups.push(regex_group.to_owned());
                }
            }
            LookGroupReference::BackReference(reference) => {
                // it is defined in this look behind
                if found_groups.contains(&reference.regex_group)
                    && group_context.direction == LookGroupDirection::Backward()
                {
                    return Ok(reference.span);
                }

                // it is not defined in a look ahead
                if !found_groups.contains(&reference.regex_group)
                    && group_context.direction == LookGroupDirection::Forward()
                {
                    return Ok(reference.span);
                }

                // we did not find it already
                if !context.finished_groups.contains(&reference.regex_group) {
                    return Ok(reference.span);
                }

                if context.disallowed_groups.contains(&reference.regex_group) {
                    return Ok(reference.span);
                }
            }
        }
    }

    group_context.timeline.clear();

    Err(false)
}

fn handle_closing_bracet(
    context: &mut CaptureGroupContext,
    open_groups: &mut Vec<RegexGroup>,
    look_behind_context: &mut LookGroupContext,
    look_ahead_context: &mut LookGroupContext,
) -> Result<Span, bool> {
    if let Some(group_type) = open_groups.pop() {
        match group_type {
            RegexGroup::LookBehind(is_positive) => {
                look_behind_context.inside_counter -= 1;

                if !is_positive {
                    context.inside_negative_count -= 1;
                }

                // println!(
                //     "
                //     context: {context:?}
                //     look_behind_context: {look_behind_context:?}
                // "
                // );

                if look_behind_context.inside_counter == 0 {
                    if let Ok(span) =
                        handle_look_group_closing(context, look_behind_context, is_positive)
                    {
                        return Ok(span);
                    }
                }
            }
            RegexGroup::Capture(_) | RegexGroup::NamedCapture(_) => {
                if context.inside_negative_count != 0
                    && !look_behind_context
                        .timeline
                        .contains(&LookGroupReference::RegexGroup(group_type.clone()))
                    && !look_behind_context
                        .timeline
                        .contains(&LookGroupReference::RegexGroup(group_type.clone()))
                {
                    context.disallowed_groups.push(group_type.clone());
                }

                context.finished_groups.push(group_type);
            }
            RegexGroup::LookAhead(is_positive) => {
                look_ahead_context.inside_counter -= 1;

                if !is_positive {
                    context.inside_negative_count -= 1;
                }

                if look_ahead_context.inside_counter == 0 {
                    if let Ok(span) =
                        handle_look_group_closing(context, look_behind_context, is_positive)
                    {
                        return Ok(span);
                    }
                }
            }
            RegexGroup::NonCapture() => {}
        }

        return Err(true);
    }
    // we found a closing group of a parent regex
    // this can happen when the parent group has an alternative

    // println!("closing parent found inside )");
    Err(false)
}

fn report_invalid_back_reference(
    regex: &str,
    escaped_char_escaped: bool,
    unicode_mode: bool,
    span_start_index: u32,
    ctx: &LintContext,
) {
    if let Ok(span) = get_invalid_back_reference_by_regex(
        regex,
        escaped_char_escaped,
        unicode_mode,
        span_start_index,
    ) {
        ctx.diagnostic(OxcDiagnostic::warn("no back reference").with_label(span));
    }
}

fn get_invalid_back_reference_by_regex(
    regex: &str,
    escaped_char_escaped: bool,
    unicode_mode: bool,
    span_start_index: u32,
) -> Result<Span, bool> {
    let chars = regex.chars();
    let peek: Peekable<std::str::Chars> = chars.peekable();

    let sub_context = &mut CaptureGroupContext {
        escaped_char_escaped,

        iterator: peek.clone(),
        current_counter: 1,
        finished_groups: vec![],

        inside_negative_count: 0,
        disallowed_groups: vec![],

        current_index: span_start_index,

        all_groups: &vec![],
        unicode_mode,
    };
    let all_groups = get_regex_groups(sub_context, true);


    get_peekable_invalid_back_references(&mut CaptureGroupContext {
        escaped_char_escaped,
        iterator: peek,
        current_counter: 1,
        finished_groups: vec![],

        inside_negative_count: 0,
        disallowed_groups: vec![],

        current_index: span_start_index,

        all_groups: &all_groups,
        unicode_mode,
    })
}

fn get_peekable_invalid_back_references(context: &mut CaptureGroupContext) -> Result<Span, bool> {
    let capture_group_count = u8::try_from(
        context
            .all_groups
            .iter()
            .filter_map(|group| match group {
                RegexGroup::Capture(_) | RegexGroup::NamedCapture(_) => Some(true),
                _ => None,
            })
            .collect::<Vec<_>>()
            .len(),
    )
    .unwrap();

    let mut backslash_started = false;
    // to know which groups is closed, we are tracking the open groups and pop() from it
    let mut open_groups: Vec<RegexGroup> = vec![];
    let mut inside_character_class_count: u8 = 0;
    let current_found_group: Vec<RegexGroup> = get_regex_groups(&mut context.clone(), false);
    let mut look_behind_context = LookGroupContext {
        inside_counter: 0,
        timeline: vec![],
        direction: LookGroupDirection::Backward(),
    };
    let mut look_ahead_context = LookGroupContext {
        inside_counter: 0,
        timeline: vec![],
        direction: LookGroupDirection::Forward(),
    };

    // println!("context: {context:?}");

    while let Some(char) = context.iterator.next() {
        context.current_index += 1;
        
        // check for backslash
        if char == '\\' {
            backslash_started = !backslash_started;

            if context.escaped_char_escaped {
                context.current_index += 1;
            }
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
                        RegexGroup::LookBehind(is_positive) => {
                            look_behind_context.inside_counter += 1;

                            if !is_positive {
                                context.inside_negative_count += 1;
                            }
                        }
                        RegexGroup::Capture(_) | RegexGroup::NamedCapture(_) => {
                            context.current_counter += 1;

                            if look_behind_context.inside_counter != 0 {
                                look_behind_context
                                    .timeline
                                    .push(LookGroupReference::RegexGroup(group_type.clone()));
                            }

                            if look_ahead_context.inside_counter != 0 {
                                look_ahead_context
                                    .timeline
                                    .push(LookGroupReference::RegexGroup(group_type.clone()));
                            }
                        }
                        RegexGroup::LookAhead(is_positive) => {
                            look_ahead_context.inside_counter += 1;

                            if !is_positive {
                                context.inside_negative_count += 1;
                            }
                        }
                        RegexGroup::NonCapture() => {}
                    }

                    open_groups.push(group_type);
                }
                ')' => {
                    let result = handle_closing_bracet(
                        context,
                        &mut open_groups,
                        &mut look_behind_context,
                        &mut look_ahead_context,
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
                        &mut look_ahead_context,
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
                // println!(
                //     "backreference {backreference:?}
                //     capture_group_count: {capture_group_count}
                //     context: {context:?}
                //     look_behind_context: {look_behind_context:?}
                //     look_ahead_context: {look_ahead_context:?}
                //     open_groups: {open_groups:?}
                //     current_found_group: {current_found_group:?}",
                // );

                if look_behind_context.inside_counter != 0 {
                    look_behind_context
                        .timeline
                        .push(LookGroupReference::BackReference(backreference.clone()));
                }

                if look_ahead_context.inside_counter != 0 {
                    look_ahead_context
                        .timeline
                        .push(LookGroupReference::BackReference(backreference.clone()));
                }

                let mut ignore_named_caputures = false;

                match backreference.regex_group {
                    RegexGroup::NamedCapture(_)
                        if !context.unicode_mode
                            && !context.all_groups.contains(&backreference.regex_group) =>
                    {
                        ignore_named_caputures = true;
                    }
                    _ => {}
                }

                // in unicode mode we dont really care if the name does not exists
                if !ignore_named_caputures {
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
                    } else if context.disallowed_groups.contains(&backreference.regex_group)
                        && look_ahead_context.inside_counter == 0
                    {
                        return Ok(backreference.span);
                    }
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

// fn get_regex_expression(expr: &str) -> (String, Vec<char>) {
//     let mut flags: Vec<char> = vec![];
//
//     let iterator = expr.chars().rev();
//     let mut delimiter: char;
//
//     while let Some(char) = iterator.next() {
//         // check for /, ", and other delimiters
//         if !char.is_alphabetic() {
//             delimiter = char;
//             break;
//         }
//
//         flags.push(char);
//     }
//
//     return (
//         delimiter,
//         flags
//     }
// }

impl Rule for NoUselessBackreference {
    fn run(&self, node: &AstNode, ctx: &LintContext) {
        match node.kind() {
            AstKind::RegExpLiteral(literal) => {
                report_invalid_back_reference(
                    &literal.regex.pattern,
                    false,
                    literal.regex.flags.contains(RegExpFlags::U),
                    literal.span.start,
                    ctx,
                );
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
                        report_invalid_back_reference(&arg.value, true, false, arg.span.start, ctx);
                    }
                    _ => {}
                };
            }
            AstKind::CallExpression(expr) if is_regexp_call_expression(expr) => {
                let regex = &expr.arguments[0];

                if let Argument::StringLiteral(arg) = regex {
                    report_invalid_back_reference(&arg.value, true, false, arg.span.start, ctx);
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
        // r"/(?<=(?=(a)\1))b/", -- ToDo
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
        // oxc: we dont check for syntax error, we are linting here

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
        r"RegExp('(a|bc)|\\1')",
        r"new RegExp('(?!(?<foo>\\n))\\1')",
        r"/(?<!(a)\1)b/",
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
        r"/(?<=(a\1))b/",
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
        // r"/.(?!(a)).(?!\1)./",
        r"/.(?<!(a)).(?<!\1)./",
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
        r"/\k<foo>((?<foo>bar)|(?<foo>baz))/",
        // r"/((?<foo>bar)|\k<foo>(?<foo>baz))/",
        r"/\k<foo>((?<foo>bar)|(?<foo>baz)|(?<foo>qux))/",
        // r"/((?<foo>bar)|\k<foo>(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz))/",
        r"/((?<foo>bar)|\k<foo>|(?<foo>baz)|(?<foo>qux))/",
        r"/((?<foo>bar)|(?<foo>baz\k<foo>)|(?<foo>qux\k<foo>))/",
        r"/(?<=((?<foo>bar)|(?<foo>baz))\k<foo>)/",
        // r"/((?!(?<foo>bar))|(?!(?<foo>baz)))\k<foo>/",
    ];

    Tester::new(NoUselessBackreference::NAME, pass, fail).test_and_snapshot();
}
