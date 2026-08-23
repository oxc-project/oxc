use std::borrow::Cow;

use rustc_hash::FxHashSet;

use crate::ast::{
    Alternative, CharacterClass, CharacterClassContents, Disjunction, LookAroundAssertionKind,
    Pattern, Term,
};
use crate::ast_impl::visit::{RegExpAstKind, Visit};

#[derive(Debug, Default)]
pub struct RegexUnsupportedFlags {
    pub sticky: bool,
    pub unicode: bool,
    pub dot_all: bool,
    pub match_indices: bool,
    pub unicode_sets: bool,
}

pub struct RegexUnsupportedPatterns {
    pub named_capture_groups: bool,
    pub duplicate_named_capture_groups: bool,
    pub unicode_property_escapes: bool,
    pub look_behind_assertions: bool,
    pub pattern_modifiers: bool,
}

/// Get the string value of a `RegExpIdentifierName` while retaining the source spelling when it
/// contains no escapes.
pub fn normalize_group_name(name: &str) -> Cow<'_, str> {
    if !name.contains('\\') {
        return Cow::Borrowed(name);
    }

    let Some(normalized) = try_normalize_group_name(name) else {
        // The RegExp parser will report malformed escapes separately.
        return Cow::Borrowed(name);
    };
    Cow::Owned(normalized)
}

fn try_normalize_group_name(name: &str) -> Option<String> {
    let bytes = name.as_bytes();
    let mut normalized = String::with_capacity(name.len());
    let mut offset = 0;
    while offset < bytes.len() {
        if bytes[offset] == b'\\' {
            let (character, len) = decode_unicode_escape(&bytes[offset..])?;
            normalized.push(character);
            offset += len;
        } else {
            let character = name[offset..].chars().next()?;
            normalized.push(character);
            offset += character.len_utf8();
        }
    }
    Some(normalized)
}

fn decode_unicode_escape(bytes: &[u8]) -> Option<(char, usize)> {
    if !bytes.starts_with(br"\u") {
        return None;
    }

    if bytes.get(2) == Some(&b'{') {
        let end = bytes[3..].iter().position(|&byte| byte == b'}')? + 3;
        let value = parse_hex(&bytes[3..end])?;
        return char::from_u32(value).map(|character| (character, end + 1));
    }

    let first = parse_hex(bytes.get(2..6)?)?;
    if (0xD800..=0xDBFF).contains(&first) {
        let second_escape = bytes.get(6..12)?;
        if !second_escape.starts_with(br"\u") {
            return None;
        }
        let second = parse_hex(&second_escape[2..])?;
        if !(0xDC00..=0xDFFF).contains(&second) {
            return None;
        }
        let value = 0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00);
        return char::from_u32(value).map(|character| (character, 12));
    }

    char::from_u32(first).map(|character| (character, 6))
}

fn parse_hex(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    bytes.iter().try_fold(0_u32, |value, &byte| {
        let digit = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'a'..=b'f' => u32::from(byte - b'a' + 10),
            b'A'..=b'F' => u32::from(byte - b'A' + 10),
            _ => return None,
        };
        value.checked_mul(16)?.checked_add(digit)
    })
}

/// Check if the regular expression flags are invalid or contain unsupported flags.
pub fn has_unsupported_regular_expression_flags(
    flags: &str,
    unsupported: &RegexUnsupportedFlags,
) -> bool {
    const UNICODE: u8 = 1 << 4;
    const UNICODE_SETS: u8 = 1 << 7;

    let mut seen = 0_u8;
    for flag in flags.bytes() {
        let (bit, is_unsupported) = match flag {
            b'g' => (1 << 0, false),
            b'i' => (1 << 1, false),
            b'm' => (1 << 2, false),
            b'y' => (1 << 3, unsupported.sticky),
            b'u' => (UNICODE, unsupported.unicode),
            b's' => (1 << 5, unsupported.dot_all),
            b'd' => (1 << 6, unsupported.match_indices),
            b'v' => (UNICODE_SETS, unsupported.unicode_sets),
            _ => return true,
        };
        if is_unsupported || seen & bit != 0 {
            return true;
        }
        seen |= bit;
    }

    seen & UNICODE != 0 && seen & UNICODE_SETS != 0
}

/// Check if the regular expression contains any unsupported syntax.
///
/// Based on parsed regular expression pattern.
pub fn has_unsupported_regular_expression_pattern(
    pattern: &Pattern,
    unsupported: &RegexUnsupportedPatterns,
) -> bool {
    disjunction_contains_unsupported(&pattern.body, unsupported)
        || (unsupported.duplicate_named_capture_groups
            && has_duplicate_named_capture_groups(pattern))
}

fn has_duplicate_named_capture_groups(pattern: &Pattern<'_>) -> bool {
    struct DuplicateNamedCaptureGroups<'a> {
        names: FxHashSet<Cow<'a, str>>,
        found: bool,
    }

    impl<'a> Visit<'a> for DuplicateNamedCaptureGroups<'a> {
        fn enter_node(&mut self, kind: RegExpAstKind<'a>) {
            let RegExpAstKind::CapturingGroup(group) = kind else {
                return;
            };
            let Some(name) = &group.name else {
                return;
            };
            if !self.names.insert(normalize_group_name(name.as_str())) {
                self.found = true;
            }
        }
    }

    let mut visitor = DuplicateNamedCaptureGroups { names: FxHashSet::default(), found: false };
    visitor.visit_pattern(pattern);
    visitor.found
}

fn disjunction_contains_unsupported(
    disjunction: &Disjunction,
    unsupported: &RegexUnsupportedPatterns,
) -> bool {
    disjunction
        .body
        .iter()
        .any(|alternative| alternative_contains_unsupported(alternative, unsupported))
}

fn alternative_contains_unsupported(
    alternative: &Alternative,
    unsupported: &RegexUnsupportedPatterns,
) -> bool {
    alternative.body.iter().any(|term| term_contains_unsupported(term, unsupported))
}

fn term_contains_unsupported(term: &Term, unsupported: &RegexUnsupportedPatterns) -> bool {
    match term {
        Term::LookAroundAssertion(assertion) => {
            if unsupported.look_behind_assertions
                && matches!(
                    assertion.kind,
                    LookAroundAssertionKind::Lookbehind
                        | LookAroundAssertionKind::NegativeLookbehind
                )
            {
                return true;
            }
            disjunction_contains_unsupported(&assertion.body, unsupported)
        }
        Term::Quantifier(quantifier) => term_contains_unsupported(&quantifier.body, unsupported),
        Term::UnicodePropertyEscape(_) => unsupported.unicode_property_escapes,
        Term::CharacterClass(character_class) => {
            unsupported.unicode_property_escapes
                && character_class_has_unicode_property_escape(character_class)
        }
        Term::CapturingGroup(group) => {
            if group.name.is_some() && unsupported.named_capture_groups {
                return true;
            }
            disjunction_contains_unsupported(&group.body, unsupported)
        }
        Term::IgnoreGroup(group) => {
            if group.modifiers.is_some() && unsupported.pattern_modifiers {
                return true;
            }
            disjunction_contains_unsupported(&group.body, unsupported)
        }
        _ => false,
    }
}

fn character_class_has_unicode_property_escape(character_class: &CharacterClass) -> bool {
    character_class.body.iter().any(|element| match element {
        CharacterClassContents::UnicodePropertyEscape(_) => true,
        CharacterClassContents::NestedCharacterClass(character_class) => {
            character_class_has_unicode_property_escape(character_class)
        }
        _ => false,
    })
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use oxc_allocator::Allocator;

    use super::*;
    use crate::{LiteralParser, Options};

    #[test]
    fn group_name_normalization() {
        assert!(matches!(normalize_group_name("name"), Cow::Borrowed("name")));
        assert_eq!(normalize_group_name(r"\u0061"), "a");
        assert_eq!(normalize_group_name(r"\u{61}"), "a");
        assert_eq!(normalize_group_name(r"\uD835\uDC9C"), "𝒜");
        assert_eq!(normalize_group_name(r"\u{FFFFFFFFFFFFFFFF}"), r"\u{FFFFFFFFFFFFFFFF}");
    }

    #[test]
    fn unsupported_flags() {
        let supported = RegexUnsupportedFlags::default();
        assert!(!has_unsupported_regular_expression_flags("gim", &supported));
        assert!(has_unsupported_regular_expression_flags("gg", &supported));
        assert!(has_unsupported_regular_expression_flags("uv", &supported));
        assert!(has_unsupported_regular_expression_flags("x", &supported));

        let unsupported = RegexUnsupportedFlags {
            sticky: true,
            unicode: true,
            dot_all: true,
            match_indices: true,
            unicode_sets: true,
        };
        for flags in ["y", "u", "s", "d", "v"] {
            assert!(has_unsupported_regular_expression_flags(flags, &unsupported));
        }
    }

    #[test]
    fn duplicate_named_capture_groups() {
        let allocator = Allocator::default();
        let pattern =
            LiteralParser::new(&allocator, "(?<name>a)|(?<name>b)", None, Options::default())
                .parse()
                .unwrap();

        let supported = RegexUnsupportedPatterns {
            named_capture_groups: false,
            duplicate_named_capture_groups: false,
            unicode_property_escapes: false,
            look_behind_assertions: false,
            pattern_modifiers: false,
        };
        assert!(!has_unsupported_regular_expression_pattern(&pattern, &supported));

        let unsupported =
            RegexUnsupportedPatterns { duplicate_named_capture_groups: true, ..supported };
        assert!(has_unsupported_regular_expression_pattern(&pattern, &unsupported));

        let escaped_pattern =
            LiteralParser::new(&allocator, r"(?<a>x)|(?<\u0061>y)", None, Options::default())
                .parse()
                .unwrap();
        assert!(has_unsupported_regular_expression_pattern(&escaped_pattern, &unsupported));
    }
}
