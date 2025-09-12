use crate::ast::{
    Alternative, CharacterClass, CharacterClassContents, Disjunction, LookAroundAssertionKind,
    Pattern, Term,
};

pub struct RegexUnsupportedPatterns {
    pub named_capture_groups: bool,
    pub unicode_property_escapes: bool,
    pub look_behind_assertions: bool,
    pub pattern_modifiers: bool,
}

/// Check if the regular expression contains any unsupported syntax.
///
/// Based on parsed regular expression pattern.
pub fn has_unsupported_regular_expression_pattern(
    pattern: &Pattern,
    unsupported: &RegexUnsupportedPatterns,
) -> bool {
    disjunction_contains_unsupported(&pattern.body, unsupported)
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
