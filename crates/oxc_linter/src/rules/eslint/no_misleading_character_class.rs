use oxc_ast::ast::RegExpFlags;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::{Character, CharacterClassContents, CharacterKind},
    visit::{RegExpAstKind, Visit},
};
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::run_on_regex_node};

fn no_misleading_character_class_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoMisleadingCharacterClass {
    allow_escape: bool,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoMisleadingCharacterClass,
    eslint,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

#[derive(Debug)]
struct RegexCollector<'a> {
    classes: Vec<&'a CharacterClassContents<'a>>,
}

impl RegexCollector<'_> {
    fn new() -> Self {
        Self { classes: Vec::new() }
    }
}

impl<'ast> Visit<'ast> for RegexCollector<'ast> {
    fn enter_node(&mut self, kind: RegExpAstKind<'ast>) {
        if let RegExpAstKind::CharacterClassContents(class) = kind {
            self.classes.push(class);
        }
    }

    fn leave_node(&mut self, _kind: RegExpAstKind<'ast>) {}
}

// Rust translation of the JS iterateCharacterSequence function
fn iterate_character_sequence<'a>(
    nodes: &'a Vec<&'a CharacterClassContents<'a>>,
) -> Vec<Vec<&'a Character>> {
    let mut result = Vec::new();
    let mut seq = Vec::new();

    for node in nodes {
        match node {
            CharacterClassContents::Character(char) => {
                seq.push(char.as_ref());
            }
            CharacterClassContents::CharacterClassRange(range) => {
                seq.push(&range.min);
                result.push(seq);
                seq = vec![&range.max];
            }
            CharacterClassContents::ClassStringDisjunction(_) => {
                if !seq.is_empty() {
                    result.push(seq);
                    seq = Vec::new();
                }
            }
            _ => {}
        }
    }

    if !seq.is_empty() {
        result.push(seq);
    }

    result
}

impl Rule for NoMisleadingCharacterClass {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_escape = value
            .get(0)
            .and_then(|v| v.as_object())
            .and_then(|v| v.get("allowEscape"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();

        Self { allow_escape }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        run_on_regex_node(node, ctx, |pattern, _span| {
            let flags = RegExpFlags::empty();
            let mut collector = RegexCollector::new();
            collector.visit_pattern(pattern);

            for unfiltered_chars in iterate_character_sequence(&collector.classes) {
                if self.allow_escape {
                    let has_escape = unfiltered_chars.iter().any(|c| {
                        !matches!(
                            c.kind,
                            CharacterKind::Symbol
                                | CharacterKind::Identifier
                                | CharacterKind::SingleEscape
                        )
                    });
                    if has_escape {
                        continue;
                    }
                }

                // Always check for combining marks, regional indicator, ZWJ, and emoji modifier sequences
                let combining_class = combining_class_sequences(&unfiltered_chars);
                if !combining_class.is_empty() {
                    ctx.diagnostic(no_misleading_character_class_diagnostic(pattern.span));
                }
                let regional_indicator = regional_indicator_symbol_sequences(&unfiltered_chars);
                if !regional_indicator.is_empty() {
                    ctx.diagnostic(no_misleading_character_class_diagnostic(pattern.span));
                }
                let zwj = zwj_sequences(&unfiltered_chars);
                let emoji_modifier = emoji_modifier_sequences(&unfiltered_chars);
                if !zwj.is_empty() {
                    ctx.diagnostic(no_misleading_character_class_diagnostic(pattern.span));
                }
                if !emoji_modifier.is_empty() {
                    ctx.diagnostic(no_misleading_character_class_diagnostic(pattern.span));
                }

                // Only check for surrogate pairs (with or without Unicode escapes) in non-unicode/v mode
                if !(flags.contains(RegExpFlags::U) || flags.contains(RegExpFlags::V)) {
                    let surrogate_pairs = surrogate_pair_sequences(&unfiltered_chars);
                    if !surrogate_pairs.is_empty() {
                        ctx.diagnostic(no_misleading_character_class_diagnostic(pattern.span));
                    }

                    let surrogate_pairs_no_flag =
                        surrogate_pair_sequences_without_flag(&unfiltered_chars);
                    if !surrogate_pairs_no_flag.is_empty() {
                        ctx.diagnostic(no_misleading_character_class_diagnostic(pattern.span));
                    }
                }
            }
        });
    }
}

// Returns true if the code point is a regional indicator symbol (U+1F1E6 to U+1F1FF)
fn is_regional_indicator_symbol(value: u32) -> bool {
    (0x1F1E6..=0x1F1FF).contains(&value)
}

// Find regional indicator symbol pairs
fn regional_indicator_symbol_sequences<'a>(chars: &[&'a Character]) -> Vec<Vec<&'a Character>> {
    let mut result = Vec::new();
    for i in 1..chars.len() {
        let prev = chars[i - 1];
        let curr = chars[i];
        if is_regional_indicator_symbol(prev.value) && is_regional_indicator_symbol(curr.value) {
            result.push(vec![prev, curr]);
        }
    }
    result
}

// Returns true if the code point is a combining mark (Unicode category Mn, Mc, or Me)
fn is_combining_character(value: u32) -> bool {
    // Covers Mn (Nonspacing_Mark), Mc (Spacing_Mark), Me (Enclosing_Mark), and variation selectors
    matches!(
        value,
        0x0300..=0x036F | // Combining Diacritical Marks
        0x1AB0..=0x1AFF | // Combining Diacritical Marks Extended
        0x1DC0..=0x1DFF | // Combining Diacritical Marks Supplement
        0x20D0..=0x20FF | // Combining Diacritical Marks for Symbols
        0xFE20..=0xFE2F | // Combining Half Marks
        0xFE0E | 0xFE0F   // VARIATION SELECTOR-15, VARIATION SELECTOR-16
    )
    // For full Unicode coverage, consider using the 'unicode-segmentation' or 'unicode-ident' crate.
}

// Find combining mark sequences: previous is not combining, current is combining
fn combining_class_sequences<'a>(chars: &[&'a Character]) -> Vec<Vec<&'a Character>> {
    let mut result = Vec::new();
    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];
        if is_combining_character(char.value) && !is_combining_character(previous.value) {
            result.push(vec![previous, char]);
        }
    }
    result
}

fn zwj_sequences<'a>(chars: &[&'a Character]) -> Vec<Vec<&'a Character>> {
    let mut result = Vec::new();
    let mut sequence: Option<Vec<&'a Character>> = None;

    for (index, &char) in chars.iter().enumerate() {
        let previous = if index > 0 { Some(chars[index - 1]) } else { None };
        let next = chars.get(index + 1).copied();

        if let (Some(previous), Some(next)) = (previous, next) {
            if char.value == 0x200D && previous.value != 0x200D && next.value != 0x200D {
                if let Some(ref mut seq) = sequence {
                    if seq.last().unwrap().value == previous.value {
                        seq.push(char);
                        seq.push(next);
                    } else {
                        result.push(seq.clone());
                        sequence = Some(vec![previous, char, next]);
                    }
                } else {
                    sequence = Some(vec![previous, char, next]);
                }
            }
        }
    }

    if let Some(seq) = sequence {
        result.push(seq);
    }

    result
}

fn is_emoji_modifier(char: &Character) -> bool {
    char.value >= 0x1f3fb && char.value <= 0x1f3ff
}

fn emoji_modifier_sequences<'a>(chars: &[&'a Character]) -> Vec<Vec<&'a Character>> {
    let mut result = Vec::new();

    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];

        if is_emoji_modifier(char) && !is_emoji_modifier(previous) {
            result.push(vec![previous, char]);
        }
    }

    result
}

// Returns true if the two code units form a surrogate pair
fn is_surrogate_pair(hi: u32, lo: u32) -> bool {
    (0xD800..=0xDBFF).contains(&hi) && (0xDC00..=0xDFFF).contains(&lo)
}

// Returns true if the character was written as a Unicode code point escape (e.g., \u{1F44D})
fn is_unicode_code_point_escape(char: &Character) -> bool {
    matches!(char.kind, CharacterKind::UnicodeEscape)
}

// Find surrogate pairs where at least one is a Unicode code point escape
fn surrogate_pair_sequences_without_flag<'a>(chars: &[&'a Character]) -> Vec<Vec<&'a Character>> {
    let mut result = Vec::new();
    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];
        if is_surrogate_pair(previous.value, char.value)
            && !is_unicode_code_point_escape(previous)
            && !is_unicode_code_point_escape(char)
        {
            result.push(vec![previous, char]);
        }
    }
    result
}

// Find surrogate pairs where at least one is a Unicode code point escape
fn surrogate_pair_sequences<'a>(chars: &[&'a Character]) -> Vec<Vec<&'a Character>> {
    let mut result = Vec::new();
    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];
        if is_surrogate_pair(previous.value, char.value)
            && (is_unicode_code_point_escape(previous) || is_unicode_code_point_escape(char))
        {
            result.push(vec![previous, char]);
        }
    }
    result
}

#[test]
#[expect(clippy::unicode_not_nfc)]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var r = /[👍]/u", None),
        (r"var r = /[\uD83D\uDC4D]/u", None),
        (r"var r = /[\u{1F44D}]/u", None),
        ("var r = /❇️/", None),
        ("var r = /Á/", None),
        ("var r = /[❇]/", None),
        ("var r = /👶🏻/", None),
        ("var r = /[👶]/u", None),
        ("var r = /🇯🇵/", None),
        ("var r = /[JP]/", None),
        ("var r = /👨‍👩‍👦/", None),
        ("new RegExp()", None),
        ("var r = RegExp(/[👍]/u)", None),
        ("const regex = /[👍]/u; new RegExp(regex);", None),
        // ("new RegExp('[👍]')", None), // { "globals": { "RegExp": "off" } },
        // Ignore solo lead/tail surrogate.
        (r"var r = /[\uD83D]/", None),
        (r"var r = /[\uDC4D]/", None),
        (r"var r = /[\uD83D]/u", None),
        (r"var r = /[\uDC4D]/u", None),
        // Ignore solo combining char.
        (r"var r = /[\u0301]/", None),
        (r"var r = /[\uFE0F]/", None),
        (r"var r = /[\u0301]/u", None),
        (r"var r = /[\uFE0F]/u", None),
        // Ignore solo emoji modifier.
        (r"var r = /[\u{1F3FB}]/u", None),
        ("var r = /[🏻]/u", None),
        // Ignore solo regional indicator symbol.
        ("var r = /[🇯]/u", None),
        ("var r = /[🇵]/u", None),
        // Ignore solo ZWJ.
        (r"var r = /[\u200D]/", None),
        (r"var r = /[\u200D]/u", None),
        // don't report and don't crash on invalid regex
        ("new RegExp('[Á] [ ');", None),
        ("var r = new RegExp('[Á] [ ');", None),
        ("var r = RegExp('{ [Á]', 'u');", None),
        ("var r = new globalThis.RegExp('[Á] [ ');", None), // { "ecmaVersion": 2020 },
        ("var r = globalThis.RegExp('{ [Á]', 'u');", None), // { "ecmaVersion": 2020 },
        // don't report on templates with expressions
        ("var r = RegExp(`${x}[👍]`)", None),
        // don't report on unknown flags
        ("var r = new RegExp('[🇯🇵]', `${foo}`)", None),
        // (r#"var r = new RegExp("[👍]", flags)"#, None),
        // don't report on spread arguments
        ("const args = ['[👍]', 'i']; new RegExp(...args);", None),
        ("var r = /[👍]/v", None),         // { "ecmaVersion": 2024 },
        (r"var r = /^[\q{👶🏻}]$/v", None),  // { "ecmaVersion": 2024 },
        (r"var r = /[🇯\q{abc}🇵]/v", None), // { "ecmaVersion": 2024 },
        ("var r = /[🇯[A]🇵]/v", None),      // { "ecmaVersion": 2024 },
        ("var r = /[🇯[A--B]🇵]/v", None),   // { "ecmaVersion": 2024 },
        // (r#"/[\ud83d\udc4d]/"#, Some(serde_json::json!([{ "allowEscape": true }]))),
        (
            r#"/[�d83d\udc4d]/u // U+D83D + Backslash + "udc4d""#,
            Some(serde_json::json!([{ "allowEscape": true }])),
        ),
        (r"/[A\u0301]/", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"/[👶\u{1f3fb}]/u", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"/[\u{1F1EF}\u{1F1F5}]/u", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"/[👨\u200d👩\u200d👦]/u", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"/[\u00B7\u0300-\u036F]/u", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"/[\n\u0305]/", Some(serde_json::json!([{ "allowEscape": true }]))),
        // (r#"RegExp("[\uD83D\uDC4D]")"#, Some(serde_json::json!([{ "allowEscape": true }]))),
        // (r#"RegExp("[A\u0301]")"#, Some(serde_json::json!([{ "allowEscape": true }]))),
        (r#"RegExp("[\x41\\u0301]")"#, Some(serde_json::json!([{ "allowEscape": true }]))),
        // (
        //     r#"RegExp(`[\uD83D\uDC4D]`) // Backslash + "uD83D" + Backslash + "uDC4D""#,
        //     Some(serde_json::json!([{ "allowEscape": true }])),
        // ),
    ];

    let fail = vec![
        ("var r = /[👍]/", None),
        (r"var r = /[\uD83D\uDC4D]/", None),
        (r"var r = /[\uD83D\uDC4D-\uffff]/", None), // { "ecmaVersion": 3, "sourceType": "script" },
        ("var r = /[👍]/", None),                   // { "ecmaVersion": 3, "sourceType": "script" },
        (r"var r = /before[\uD83D\uDC4D]after/", None),
        (r"var r = /[before\uD83D\uDC4Dafter]/", None),
        (r"var r = /\uDC4D[\uD83D\uDC4D]/", None),
        ("var r = /[👍]/", None), // { "ecmaVersion": 5, "sourceType": "script" },
        (r"var r = /[👍]\a/", None),
        (r"var r = /\a[👍]\a/", None),
        ("var r = /(?<=[👍])/", None), // { "ecmaVersion": 9 },
        ("var r = /(?<=[👍])/", None), // { "ecmaVersion": 2018 },
        ("var r = /[Á]/", None),
        ("var r = /[Á]/u", None),
        (r"var r = /[\u0041\u0301]/", None),
        (r"var r = /[\u0041\u0301]/u", None),
        (r"var r = /[\u{41}\u{301}]/u", None),
        ("var r = /[❇️]/", None),
        ("var r = /[❇️]/u", None),
        (r"var r = /[\u2747\uFE0F]/", None),
        (r"var r = /[\u2747\uFE0F]/u", None),
        (r"var r = /[\u{2747}\u{FE0F}]/u", None),
        ("var r = /[👶🏻]/", None),
        ("var r = /[👶🏻]/u", None),
        (r"var r = /[a\uD83C\uDFFB]/u", None),
        (r"var r = /[\uD83D\uDC76\uD83C\uDFFB]/u", None),
        (r"var r = /[\u{1F476}\u{1F3FB}]/u", None),
        ("var r = /[🇯🇵]/", None),
        ("var r = /[🇯🇵]/i", None),
        ("var r = /[🇯🇵]/u", None),
        (r"var r = /[\uD83C\uDDEF\uD83C\uDDF5]/u", None),
        (r"var r = /[\u{1F1EF}\u{1F1F5}]/u", None),
        ("var r = /[👨‍👩‍👦]/", None),
        ("var r = /[👨‍👩‍👦]/u", None),
        ("var r = /[👩‍👦]/u", None),
        ("var r = /[👩‍👦][👩‍👦]/u", None),
        ("var r = /[👨‍👩‍👦]foo[👨‍👩‍👦]/u", None),
        ("var r = /[👨‍👩‍👦👩‍👦]/u", None),
        (r"var r = /[\uD83D\uDC68\u200D\uD83D\uDC69\u200D\uD83D\uDC66]/u", None),
        (r"var r = /[\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466}]/u", None),
        (r"var r = /[\uD83D\uDC68\u200D\uD83D\uDC69]/u", None),
        (r"var r = /[\u{1F468}\u{200D}\u{1F469}]/u", None),
        (
            r"var r = /[\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466}]foo[\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466}]/u",
            None,
        ),
        (r#"var r = RegExp("[👍]", "")"#, None),
        (r#"var r = new RegExp("[👍]", "")"#, None),
        ("var r = new RegExp('[👍]', ``)", None),
        (
            "var r = new RegExp(`
			                [👍]`)",
            None,
        ),
        (
            "var r = new RegExp(`
			                [❇️]`)",
            None,
        ),
        (
            "var r = new RegExp(`
			[❇️]`)",
            None,
        ),
        (r#"const flags = ""; var r = new RegExp("[👍]", flags)"#, None),
        (r#"var r = RegExp("[\\uD83D\\uDC4D]", "")"#, None),
        (r#"var r = RegExp("before[\\uD83D\\uDC4D]after", "")"#, None),
        (r#"var r = RegExp("[before\\uD83D\\uDC4Dafter]", "")"#, None),
        (r#"var r = RegExp("\t\t\t👍[👍]")"#, None),
        (r#"var r = new RegExp("\u1234[\\uD83D\\uDC4D]")"#, None),
        (r#"var r = new RegExp("\\u1234\\u5678👎[👍]")"#, None),
        (r#"var r = new RegExp("\\u1234\\u5678👍[👍]")"#, None),
        (r#"var r = new RegExp("[👍]", "")"#, None), // { "ecmaVersion": 3, "sourceType": "script" },
        (r#"var r = new RegExp("[👍]", "")"#, None), // { "ecmaVersion": 5, "sourceType": "script" },
        (r#"var r = new RegExp("[👍]\\a", "")"#, None),
        (r#"var r = new RegExp("/(?<=[👍])", "")"#, None), // { "ecmaVersion": 9 },
        (r#"var r = new RegExp("/(?<=[👍])", "")"#, None), // { "ecmaVersion": 2018 },
        (r#"var r = new RegExp("[Á]", "")"#, None),
        (r#"var r = new RegExp("[Á]", "u")"#, None),
        (r#"var r = new RegExp("[\\u0041\\u0301]", "")"#, None),
        (r#"var r = new RegExp("[\\u0041\\u0301]", "u")"#, None),
        (r#"var r = new RegExp("[\\u{41}\\u{301}]", "u")"#, None),
        (r#"var r = new RegExp("[❇️]", "")"#, None),
        (r#"var r = new RegExp("[❇️]", "u")"#, None),
        (r#"new RegExp("[ \\ufe0f]", "")"#, None),
        (r#"new RegExp("[ \\ufe0f]", "u")"#, None),
        (r#"new RegExp("[ \\ufe0f][ \\ufe0f]")"#, None),
        (r#"var r = new RegExp("[\\u2747\\uFE0F]", "")"#, None),
        (r#"var r = new RegExp("[\\u2747\\uFE0F]", "u")"#, None),
        (r#"var r = new RegExp("[\\u{2747}\\u{FE0F}]", "u")"#, None),
        (r#"var r = new RegExp("[👶🏻]", "")"#, None),
        (r#"var r = new RegExp("[👶🏻]", "u")"#, None),
        (r#"var r = new RegExp("[\\uD83D\\uDC76\\uD83C\\uDFFB]", "u")"#, None),
        (r#"var r = new RegExp("[\\u{1F476}\\u{1F3FB}]", "u")"#, None),
        ("var r = RegExp(`			👍[👍]`)", None),
        (r"var r = RegExp(`\t\t\t👍[👍]`)", None),
        (r#"var r = new RegExp("[🇯🇵]", "")"#, None),
        (r#"var r = new RegExp("[🇯🇵]", "i")"#, None),
        ("var r = new RegExp('[🇯🇵]', `i`)", None),
        (r#"var r = new RegExp("[🇯🇵]")"#, None),
        (r#"var r = new RegExp("[🇯🇵]",)"#, None), // { "ecmaVersion": 2017 },
        // parentheses
        // (r#"var r = new RegExp(("[🇯🇵]"))"#, None),
        // (r#"var r = new RegExp((("[🇯🇵]")))"#, None),
        // (r#"var r = new RegExp(("[🇯🇵]"),)"#, None), // { "ecmaVersion": 2017 },
        (r#"var r = new RegExp("[🇯🇵]", "u")"#, None),
        (r#"var r = new RegExp("[\\uD83C\\uDDEF\\uD83C\\uDDF5]", "u")"#, None),
        (r#"var r = new RegExp("[\\u{1F1EF}\\u{1F1F5}]", "u")"#, None),
        (r#"var r = new RegExp("[👨‍👩‍👦]", "")"#, None),
        (r#"var r = new RegExp("[👨‍👩‍👦]", "u")"#, None),
        (r#"var r = new RegExp("[👩‍👦]", "u")"#, None),
        (r#"var r = new RegExp("[👩‍👦][👩‍👦]", "u")"#, None),
        (r#"var r = new RegExp("[👨‍👩‍👦]foo[👨‍👩‍👦]", "u")"#, None),
        (r#"var r = new RegExp("[👨‍👩‍👦👩‍👦]", "u")"#, None),
        (
            r#"var r = new RegExp("[\\uD83D\\uDC68\\u200D\\uD83D\\uDC69\\u200D\\uD83D\\uDC66]", "u")"#,
            None,
        ),
        (r#"var r = new RegExp("[\\u{1F468}\\u{200D}\\u{1F469}\\u{200D}\\u{1F466}]", "u")"#, None),
        (r#"var r = new globalThis.RegExp("[❇️]", "")"#, None), // { "ecmaVersion": 2020 },
        (r#"var r = new globalThis.RegExp("[👶🏻]", "u")"#, None), // { "ecmaVersion": 2020 },
        (r#"var r = new globalThis.RegExp("[🇯🇵]", "")"#, None), // { "ecmaVersion": 2020 },
        (
            r#"var r = new globalThis.RegExp("[\\u{1F468}\\u{200D}\\u{1F469}\\u{200D}\\u{1F466}]", "u")"#,
            None,
        ), // { "ecmaVersion": 2020 },
        (r"/[\ud83d\u{dc4d}]/u", None),
        (r"/[\u{d83d}\udc4d]/u", None),
        (r"/[\u{d83d}\u{dc4d}]/u", None),
        (r"/[\uD83D\u{DC4d}]/u", None),
        // (r#"new RegExp(`${"[👍🇯🇵]"}[😊]`);"#, None),
        // (r#"const pattern = "[👍]"; new RegExp(pattern);"#, None),
        // ("RegExp(/[a👍z]/u, '');", None),
        ("RegExp(/[👍]/)", None),
        ("RegExp(/[👍]/, 'i');", None),
        ("RegExp(/[👍]/, 'g');", None), // { "globals": { "RegExp": "off" } },
        (
            r##"r#\"

			            // \"[\" and \"]\" escaped as \"\\x5B\" and \"\\u005D\"
			            new RegExp(\"\\x5B \\\\ufe0f\\u005D\")

			            \"#"##,
            None,
        ),
        (
            r##"r#\"

			            // backslash escaped as \"\\u{5c}\"
			            new RegExp(\"[ \\u{5c}ufe0f]\")

			            \"#"##,
            None,
        ),
        (
            r##"r#\"

			            // \"0\" escaped as \"\\60\"
			            new RegExp(\"[ \\\\ufe\\60f]\")

			            \"#"##,
            None,
        ), // { "sourceType": "script" },
        (
            r##"r#\"

			            // \"e\" escaped as \"\\e\"
			            new RegExp(\"[ \\\\uf\\e0f]\")

			            \"#"##,
            None,
        ),
        ("var r = /[[👶🏻]]/v", None), // { "ecmaVersion": 2024 },
        // ("new RegExp(/^[👍]$/v, '')", None), // {				"ecmaVersion": 2024,			},
        (r"/[Á]/", Some(serde_json::json!([{ "allowEscape": false }]))),
        (r"/[\\̶]/", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"/[\n̅]/", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"/[\👍]/", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"RegExp('[\è]')", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"RegExp('[\👍]')", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"RegExp('[\\👍]')", Some(serde_json::json!([{ "allowEscape": true }]))),
        (r"RegExp('[\❇️]')", Some(serde_json::json!([{ "allowEscape": true }]))),
        (
            r"RegExp(`[\👍]`) // Backslash + U+D83D + U+DC4D",
            Some(serde_json::json!([{ "allowEscape": true }])),
        ),
        // (
        //     r#"const pattern = "[\x41\u0301]"; RegExp(pattern);"#,
        //     Some(serde_json::json!([{ "allowEscape": true }])),
        // ),
    ];

    Tester::new(NoMisleadingCharacterClass::NAME, NoMisleadingCharacterClass::PLUGIN, pass, fail)
        .test_and_snapshot();
}
