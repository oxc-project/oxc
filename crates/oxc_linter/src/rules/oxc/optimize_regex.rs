use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::ast::{
    Alternative, CapturingGroup, CharacterClass, CharacterClassContents,
    CharacterClassContentsKind, Disjunction, IgnoreGroup, LookAroundAssertion, Pattern, Quantifier,
    Term,
};
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn optimize_regex_diagnostic(span: Span, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("This regular expression can be simplified.")
        .with_help(format!("Replace this fragment with `{replacement}`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct OptimizeRegex;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Simplifies a narrow set of regular-expression fragments that are longer
    /// than equivalent built-in forms.
    ///
    /// ### Why is this bad?
    ///
    /// Shorter regular expressions are easier to scan and usually make the
    /// intended character set or repetition clearer to readers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const digits = /[0-9]+/;
    /// const word = /[A-Z_a-z0-9]+/;
    /// const repeated = /aaaaaa/;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const digits = /\d+/;
    /// const word = /\w+/;
    /// const repeated = /a{6}/;
    /// ```
    OptimizeRegex,
    oxc,
    style,
    fix
);

impl Rule for OptimizeRegex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::RegExpLiteral(regex) = node.kind() else {
            return;
        };

        let Some(pattern) = regex.regex.pattern.pattern.as_deref() else {
            return;
        };

        let mut replacements = Vec::new();
        collect_pattern_replacements(pattern, ctx, &mut replacements);

        for replacement in replacements {
            ctx.diagnostic_with_fix(
                optimize_regex_diagnostic(replacement.span, &replacement.text),
                |fixer| fixer.replace(replacement.span, replacement.text.clone()),
            );
        }
    }
}

#[derive(Debug, Clone)]
struct Replacement {
    span: Span,
    text: String,
}

fn collect_pattern_replacements(
    pattern: &Pattern<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    collect_disjunction_replacements(&pattern.body, ctx, replacements);
}

fn collect_disjunction_replacements(
    disjunction: &Disjunction<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    for alternative in &disjunction.body {
        collect_alternative_replacements(alternative, ctx, replacements);
    }
}

fn collect_alternative_replacements(
    alternative: &Alternative<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    for term in &alternative.body {
        collect_term_replacements(term, ctx, replacements);
    }

    let source_text = ctx.source_text();
    let mut index = 0usize;

    while index < alternative.body.len() {
        let Some(raw_atom) = repeatable_atom_source(&alternative.body[index], source_text) else {
            index += 1;
            continue;
        };

        let mut end_index = index + 1;
        while end_index < alternative.body.len()
            && repeatable_atom_source(&alternative.body[end_index], source_text) == Some(raw_atom)
        {
            end_index += 1;
        }

        let count = end_index - index;
        if count >= 2 {
            let replacement = format!("{raw_atom}{{{count}}}");
            if replacement.len() < raw_atom.len() * count {
                let span = Span::new(
                    alternative.body[index].span().start,
                    alternative.body[end_index - 1].span().end,
                );
                replacements.push(Replacement { span, text: replacement });
                index = end_index;
                continue;
            }
        }

        index += 1;
    }
}

fn collect_term_replacements(
    term: &Term<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    match term {
        Term::Quantifier(quantifier) => {
            collect_quantifier_replacements(quantifier, ctx, replacements)
        }
        Term::CharacterClass(class) => {
            if let Some(text) = character_class_replacement(class) {
                replacements.push(Replacement { span: class.span, text: text.to_string() });
            }
        }
        Term::CapturingGroup(group) => {
            collect_capturing_group_replacements(group, ctx, replacements)
        }
        Term::IgnoreGroup(group) => collect_ignore_group_replacements(group, ctx, replacements),
        Term::LookAroundAssertion(assertion) => {
            collect_lookaround_replacements(assertion, ctx, replacements);
        }
        _ => {}
    }
}

fn collect_quantifier_replacements(
    quantifier: &Quantifier<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    collect_term_replacements(&quantifier.body, ctx, replacements);
}

fn collect_capturing_group_replacements(
    group: &CapturingGroup<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    collect_disjunction_replacements(&group.body, ctx, replacements);
}

fn collect_ignore_group_replacements(
    group: &IgnoreGroup<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    collect_disjunction_replacements(&group.body, ctx, replacements);
}

fn collect_lookaround_replacements(
    assertion: &LookAroundAssertion<'_>,
    ctx: &LintContext<'_>,
    replacements: &mut Vec<Replacement>,
) {
    collect_disjunction_replacements(&assertion.body, ctx, replacements);
}

fn repeatable_atom_source<'a>(term: &Term<'_>, source_text: &'a str) -> Option<&'a str> {
    match term {
        Term::Character(character) => Some(character.span.source_text(source_text)),
        Term::Dot(dot) => Some(dot.span.source_text(source_text)),
        _ => None,
    }
}

fn character_class_replacement(class: &CharacterClass<'_>) -> Option<&'static str> {
    if class.strings || class.kind != CharacterClassContentsKind::Union {
        return None;
    }

    if is_digits_class(class) {
        return Some(if class.negative { r"\D" } else { r"\d" });
    }

    if is_word_class(class) {
        return Some(if class.negative { r"\W" } else { r"\w" });
    }

    None
}

fn is_digits_class(class: &CharacterClass<'_>) -> bool {
    class.body.len() == 1
        && matches!(class.body[0], CharacterClassContents::CharacterClassRange(ref range) if is_range(range, '0', '9'))
}

fn is_word_class(class: &CharacterClass<'_>) -> bool {
    if class.body.len() != 4 {
        return false;
    }

    let mut seen_lower = false;
    let mut seen_upper = false;
    let mut seen_digits = false;
    let mut seen_underscore = false;

    for content in &class.body {
        match content {
            CharacterClassContents::CharacterClassRange(range) if is_range(range, 'a', 'z') => {
                if seen_lower {
                    return false;
                }
                seen_lower = true;
            }
            CharacterClassContents::CharacterClassRange(range) if is_range(range, 'A', 'Z') => {
                if seen_upper {
                    return false;
                }
                seen_upper = true;
            }
            CharacterClassContents::CharacterClassRange(range) if is_range(range, '0', '9') => {
                if seen_digits {
                    return false;
                }
                seen_digits = true;
            }
            CharacterClassContents::Character(character) if character.value == u32::from(b'_') => {
                if seen_underscore {
                    return false;
                }
                seen_underscore = true;
            }
            _ => return false,
        }
    }

    seen_lower && seen_upper && seen_digits && seen_underscore
}

fn is_range(
    range: &oxc_regular_expression::ast::CharacterClassRange,
    min: char,
    max: char,
) -> bool {
    range.min.value == min as u32 && range.max.value == max as u32
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<(&str, Option<serde_json::Value>, Option<serde_json::Value>)> = vec![
        (r"const digits = /\d+/;", None, None),
        (r"const word = /\w+/;", None, None),
        (r"const not_word = /\W{2}/;", None, None),
        (r"const hex = /[0-9a-f]+/;", None, None),
        (r"const almost = /aaaa/;", None, None),
        (r"const mixed = /(ab)(ab)/;", None, None),
    ];

    let fail: Vec<(&str, Option<serde_json::Value>, Option<serde_json::Value>)> = vec![
        (r"const digits = /[0-9]+/;", None, None),
        (r"const not_digits = /[^0-9]+/;", None, None),
        (r"const word = /[A-Z_a-z0-9]+/;", None, None),
        (r"const not_word = /[^a-zA-Z0-9_]+/;", None, None),
        (r"const repeated = /aaaaaa/;", None, None),
        (r"const escaped = /\.\.\./;", None, None),
        (r"const nested = /(?=[0-9])foo/;", None, None),
    ];

    let fix = vec![
        (r"const digits = /[0-9]+/;", r"const digits = /\d+/;", None),
        (r"const not_digits = /[^0-9]+/;", r"const not_digits = /\D+/;", None),
        (r"const word = /[A-Z_a-z0-9]+/;", r"const word = /\w+/;", None),
        (r"const not_word = /[^a-zA-Z0-9_]+/;", r"const not_word = /\W+/;", None),
        (r"const repeated = /aaaaaa/;", r"const repeated = /a{6}/;", None),
        (r"const escaped = /\.\.\./;", r"const escaped = /\.{3}/;", None),
        (r"const nested = /(?=[0-9])foo/;", r"const nested = /(?=\d)foo/;", None),
    ];

    Tester::new(OptimizeRegex::NAME, OptimizeRegex::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
