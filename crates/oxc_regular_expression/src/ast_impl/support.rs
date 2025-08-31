use crate::ast::{CharacterClass, CharacterClassContents, LookAroundAssertionKind, Pattern, Term};

pub struct RegexUnsupportedPatterns {
    pub named_capture_groups: bool,
    pub unicode_property_escapes: bool,
    pub look_behind_assertions: bool,
}

/// Check if the regular expression contains any unsupported syntax.
///
/// Based on parsed regular expression pattern.
pub fn has_unsupported_regular_expression_pattern(
    pattern: &Pattern,
    unsupported: &RegexUnsupportedPatterns,
) -> bool {
    pattern.body.body.iter().any(|alternative| {
        alternative.body.iter().any(|term| term_contains_unsupported(term, unsupported))
    })
}

fn term_contains_unsupported(mut term: &Term, unsupported: &RegexUnsupportedPatterns) -> bool {
    // Loop because `Term::Quantifier` contains a nested `Term`
    loop {
        match term {
            Term::CapturingGroup(group) => {
                return group.name.is_some() && unsupported.named_capture_groups;
            }
            Term::UnicodePropertyEscape(_) => return unsupported.unicode_property_escapes,
            Term::CharacterClass(character_class) => {
                return unsupported.unicode_property_escapes
                    && character_class_has_unicode_property_escape(character_class);
            }
            Term::LookAroundAssertion(assertion) => {
                return unsupported.look_behind_assertions
                    && matches!(
                        assertion.kind,
                        LookAroundAssertionKind::Lookbehind
                            | LookAroundAssertionKind::NegativeLookbehind
                    );
            }
            Term::Quantifier(quantifier) => term = &quantifier.body,
            _ => return false,
        }
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
