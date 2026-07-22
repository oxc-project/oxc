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
        names: FxHashSet<&'a str>,
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
            if !self.names.insert(name.as_str()) {
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
    use oxc_allocator::Allocator;

    use super::*;
    use crate::{LiteralParser, Options};

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
    }
}
