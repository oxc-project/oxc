use itertools::Itertools;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::{Character, CharacterClassContents, CharacterKind},
    visit::{RegExpAstKind, Visit},
};
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::run_on_regex_node,
};

fn surrogate_pair_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected surrogate pair in character class.").with_label(span)
}

fn combining_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected combining class in character class.").with_label(span)
}

fn emoji_modifiers_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected emoji modifier in character class.").with_label(span)
}

fn regional_indicator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected regional indicator in character class.").with_label(span)
}

fn zwj_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected joined character sequence in character class.").with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoMisleadingCharacterClass {
    /// When set to `true`, the rule allows any grouping of code points
    /// inside a character class as long as they are written using escape sequences.
    ///
    /// Examples of **incorrect** code for this rule with `{ "allowEscape": true }`:
    /// ```javascript
    /// /[\uD83D]/; // backslash can be omitted
    /// new RegExp("[\ud83d" + "\udc4d]");
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "allowEscape": true }`:
    /// ```javascript
    /// /[\ud83d\udc4d]/;
    /// /[\u00B7\u0300-\u036F]/u;
    /// /[👨\u200d👩]/u;
    /// new RegExp("[\x41\u0301]");
    /// new RegExp(`[\u{1F1EF}\u{1F1F5}]`, "u");
    /// new RegExp("[\\u{1F1EF}\\u{1F1F5}]", "u");
    /// ```
    allow_escape: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports regular expressions which include multiple code point characters in character class syntax. This includes:
    ///
    /// - Characters with combining marks (e.g., `Á` where `A` is followed by a combining acute accent)
    /// - Characters with emoji modifiers (e.g., `👶🏻`)
    /// - Pairs of regional indicator symbols (e.g., `🇯🇵`)
    /// - Characters joined by zero-width joiner (ZWJ) (e.g., `👨‍👩‍👦`)
    /// - Surrogate pairs without the Unicode flag (e.g., `/^[👍]$/`)
    ///
    /// ### Why is this bad?
    ///
    /// Unicode includes characters which are made by multiple code points.
    /// RegExp character class syntax (`/[abc]/`) cannot handle characters
    /// which are made by multiple code points as a character;
    /// those characters will be dissolved to each code point.
    /// For example, `❇️` is made by `❇` (`U+2747`) and VARIATION SELECTOR-16 (`U+FE0F`).
    /// If this character is in a RegExp character class,
    /// it will match either `❇` (`U+2747`) or VARIATION SELECTOR-16 (`U+FE0F`) rather than `❇️`.
    ///
    /// This can lead to regular expressions that do not match what the author intended,
    /// especially for emoji, regional indicators, and characters with combining marks.
    /// #### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /^[Á]$/u;
    /// /^[❇️]$/u;
    /// /^[👶🏻]$/u;
    /// /^[🇯🇵]$/u;
    /// /^[👨‍👩‍👦]$/u;
    /// /^[👍]$/;
    /// new RegExp("[🎵]");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /^[abc]$/;
    /// /^[👍]$/u;
    /// /[\u00B7\u0300-\u036F]/u;
    /// new RegExp("^[\u{1F1EF}\u{1F1F5}]", "u");
    /// ```
    NoMisleadingCharacterClass,
    eslint,
    nursery, // TODO: change category to `correctness`, after oxc-project/oxc#13660 and oxc-project/oxc#13436
    config = NoMisleadingCharacterClass,
);

#[derive(Debug)]
struct CharacterSequenceCollector<'a> {
    sequences: Vec<Vec<&'a Character>>,
    current_seq: Vec<&'a Character>,
}

impl CharacterSequenceCollector<'_> {
    fn new() -> Self {
        Self { sequences: Vec::new(), current_seq: Vec::new() }
    }
}

impl<'ast> Visit<'ast> for CharacterSequenceCollector<'ast> {
    fn enter_node(&mut self, kind: RegExpAstKind<'ast>) {
        let RegExpAstKind::CharacterClassContents(class) = kind else {
            return;
        };

        match class {
            CharacterClassContents::Character(char) => {
                self.current_seq.push(char.as_ref());
            }
            CharacterClassContents::CharacterClassRange(range) => {
                self.current_seq.push(&range.min);
                self.sequences.push(std::mem::take(&mut self.current_seq));
                self.current_seq.push(&range.max);
            }
            CharacterClassContents::ClassStringDisjunction(_) => {
                if !self.current_seq.is_empty() {
                    self.sequences.push(std::mem::take(&mut self.current_seq));
                }
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, _kind: RegExpAstKind<'ast>) {}
}

impl Rule for NoMisleadingCharacterClass {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoMisleadingCharacterClass>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        run_on_regex_node(node, ctx, |pattern, _span| {
            let mut collector = CharacterSequenceCollector::new();
            collector.visit_pattern(pattern);

            // Restore: push any remaining sequence after visiting
            if !collector.current_seq.is_empty() {
                collector.sequences.push(std::mem::take(&mut collector.current_seq));
            }

            for unfiltered_chars in &collector.sequences {
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
                if combining_class_sequences(unfiltered_chars) {
                    ctx.diagnostic(combining_class_diagnostic(pattern.span));
                }
                if regional_indicator_symbol_sequences(unfiltered_chars) {
                    ctx.diagnostic(regional_indicator_diagnostic(pattern.span));
                }
                if zwj_sequences(unfiltered_chars) {
                    ctx.diagnostic(zwj_diagnostic(pattern.span));
                }
                if emoji_modifier_sequences(unfiltered_chars) {
                    ctx.diagnostic(emoji_modifiers_diagnostic(pattern.span));
                }
                if surrogate_pair_sequences(unfiltered_chars) {
                    ctx.diagnostic(surrogate_pair_diagnostic(pattern.span));
                }
                if surrogate_pair_sequences_without_flag(unfiltered_chars) {
                    ctx.diagnostic(surrogate_pair_diagnostic(pattern.span));
                }
            }
        });
    }
}

// Returns true if the code point is a regional indicator symbol
//
// Regional Indicator Symbols:
// - Unicode characters in the range U+1F1E6 to U+1F1FF
// - Each one represents an uppercase Latin letter A–Z
// - Used in pairs to represent ISO 3166-1 alpha-2 country codes
// - Appear as flag emojis when combined in supported systems
fn is_regional_indicator_symbol(value: u32) -> bool {
    (0x1F1E6..=0x1F1FF).contains(&value)
}

// Find regional indicator symbol pairs
fn regional_indicator_symbol_sequences(chars: &[&Character]) -> bool {
    for (prev, curr) in chars.iter().tuple_windows() {
        if is_regional_indicator_symbol(prev.value) && is_regional_indicator_symbol(curr.value) {
            return true;
        }
    }
    false
}

// Returns true if the code point is a combining mark (Unicode category Mn, Mc, or Me)
//
// Unicode Combining Character Ranges
// A. General Combining Characters (Mn, Mc, Me)
// Combining characters are those whose General Category falls under:
// - Mn — Nonspacing Mark
// - Mc — Spacing Mark
// - Me — Enclosing Mark
//
// Some Unicode blocks dedicated (entirely or in part) to combining marks include :
// - Combining Diacritical Marks: U+0300–U+036F
// - Combining Diacritical Marks Extended: U+1AB0–U+1AFF
// - Combining Diacritical Marks Supplement: U+1DC0–U+1DFF
// - Combining Diacritical Marks for Symbols: U+20D0–U+20FF
// - Combining Half Marks: U+FE20–U+FE2F
//
// Additional combining characters exist within script-specific blocks
// (e.g., Devanagari signs, Hiragana/Katakana marks) and are not confined to a block,
// but share the Mn/Mc/Me categories
//
// B. Variation Selectors (also Combining)
//
// Variation selectors are also combining characters. There are two blocks:
// - Variation Selectors (Basic): U+FE00–U+FE0F
// - Variation Selectors Supplement: U+E0100–U+E01EF (256 total)
fn is_combining_character(value: u32) -> bool {
    // Covers Mn (Nonspacing_Mark), Mc (Spacing_Mark), Me (Enclosing_Mark), and variation selectors
    matches!(
        value,
        0x0300..=0x036F | // Combining Diacritical Marks
        0x1AB0..=0x1AFF | // Combining Diacritical Marks Extended
        0x1DC0..=0x1DFF | // Combining Diacritical Marks Supplement
        0x20D0..=0x20FF | // Combining Diacritical Marks for Symbols
        0xFE20..=0xFE2F | // Combining Half Marks
        0xFE00..=0xFE0F | // Variation Selectors (Basic)
        0xE0100..=0xE01EF  // Variation Selectors (Supplement)
    )
}

// Find combining mark sequences: previous is not combining, current is combining
fn combining_class_sequences(chars: &[&Character]) -> bool {
    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];
        if is_combining_character(char.value) && !is_combining_character(previous.value) {
            return true;
        }
    }
    false
}

// Returns true if a zero width joiner character is detected between two characters
fn zwj_sequences(chars: &[&Character]) -> bool {
    for (index, &char) in chars.iter().enumerate() {
        let previous = if index > 0 { Some(chars[index - 1]) } else { None };
        let next = chars.get(index + 1).copied();
        if let (Some(previous), Some(next)) = (previous, next)
            && char.value == 0x200D
            && previous.value != 0x200D
            && next.value != 0x200D
        {
            return true;
        }
    }
    false
}

fn is_emoji_modifier(char: &Character) -> bool {
    char.value >= 0x1f3fb && char.value <= 0x1f3ff
}

// Returns true if a emoji modifier sequence is detected
//
// Emoji modifiers are special Unicode characters used to modify the appearance of other emojis, such as:
// - Skin tone
// - Gender
// - Hair style
// - etc.
//
// They’re combined with base emojis (like people or body parts) to create variant emoji sequences.
fn emoji_modifier_sequences(chars: &[&Character]) -> bool {
    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];

        if is_emoji_modifier(char) && !is_emoji_modifier(previous) {
            return true;
        }
    }

    false
}

// Returns true if the two code units form a surrogate pair
//
// Structure of a Surrogate Pair
// - A high surrogate (also called lead surrogate)
// A low surrogate (also called trail surrogate)
//
// High Surrogate (Lead)
// - Range: U+D800 to U+DBFF
//
// Low Surrogate (Trail)
// - Range: U+DC00 to U+DFFF
fn is_surrogate_pair(hi: u32, lo: u32) -> bool {
    (0xD800..=0xDBFF).contains(&hi) && (0xDC00..=0xDFFF).contains(&lo)
}

// Returns true if the character was written as a Unicode code point escape (e.g., \u{1F44D})
fn is_unicode_code_point_escape(char: &Character) -> bool {
    matches!(char.kind, CharacterKind::UnicodeEscape)
}

// Find surrogate pairs where at least one is a Unicode code point escape
fn surrogate_pair_sequences_without_flag(chars: &[&Character]) -> bool {
    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];
        if is_surrogate_pair(previous.value, char.value)
            && !is_unicode_code_point_escape(previous)
            && !is_unicode_code_point_escape(char)
        {
            return true;
        }
    }
    false
}

// Find surrogate pairs where at least one is a Unicode code point escape
fn surrogate_pair_sequences(chars: &[&Character]) -> bool {
    for (index, &char) in chars.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let previous = chars[index - 1];
        if is_surrogate_pair(previous.value, char.value)
            && (is_unicode_code_point_escape(previous) || is_unicode_code_point_escape(char))
        {
            return true;
        }
    }
    false
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
        (r"/[\ud83d\udc4d]/", Some(serde_json::json!([{ "allowEscape": true }]))),
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
        (r#"var r = new RegExp(("[🇯🇵]"))"#, None),
        (r#"var r = new RegExp((("[🇯🇵]")))"#, None),
        (r#"var r = new RegExp(("[🇯🇵]"),)"#, None), // { "ecmaVersion": 2017 },
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
        // complex template literal
        // (r#"new RegExp(`${"[👍🇯🇵]"}[😊]`);"#, None),
        // references from variables
        // (r#"const pattern = "[👍]"; new RegExp(pattern);"#, None),
        // flag overrides, see oxc-project/oxc#13436
        // ("RegExp(/[a👍z]/u, '');", None),
        ("RegExp(/[👍]/)", None),
        ("RegExp(/[👍]/, 'i');", None),
        ("RegExp(/[👍]/, 'g');", None), // { "globals": { "RegExp": "off" } },
        (
            r#"new RegExp("\x5B \\ufe0f\u005D")"#, // "[" and "]" escaped as "\x5B" and "\u005D"
            None,
        ),
        (
            r#"new RegExp("[ \u{5c}ufe0f]")"#, // backslash escaped as "\\u{5c}"
            None,
        ),
        (
            r#"new RegExp("[ \\ufe\60f]")"#, // "0" escaped as "\60"
            None,
        ), // { "sourceType": "script" },
        (
            r#"new RegExp("[ \\uf\e0f]")"#, // "e" escaped as "\e"
            None,
        ),
        // (
        //     r#"new RegExp('[ \\u\\\r\nfe0f]')"#, // line continuation: backslash + <CR> + <LF>
        //     None
        // ),
        (
            r"new RegExp(`[.\\u200D.]`)", // just a backslash escaped as "\\"
            None,
        ),
        (
            r"new RegExp(`[.\\\x75200D.]`)", // "u" escaped as "\x75"
            None,
        ),
        ("var r = /[[👶🏻]]/v", None), // { "ecmaVersion": 2024 },
        // flag overrides, see oxc-project/oxc#13436
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
        // references from variables
        // (
        //     r#"const pattern = "[\x41\u0301]"; RegExp(pattern);"#,
        //     Some(serde_json::json!([{ "allowEscape": true }])),
        // ),
    ];

    Tester::new(NoMisleadingCharacterClass::NAME, NoMisleadingCharacterClass::PLUGIN, pass, fail)
        .test_and_snapshot();
}
