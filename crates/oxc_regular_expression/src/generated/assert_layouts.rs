// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::ast::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 0 bytes
    assert!(size_of::<Pattern>() == 72);
    assert!(align_of::<Pattern>() == 8);
    assert!(offset_of!(Pattern, span) == 0);
    assert!(offset_of!(Pattern, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Disjunction>() == 48);
    assert!(align_of::<Disjunction>() == 8);
    assert!(offset_of!(Disjunction, span) == 0);
    assert!(offset_of!(Disjunction, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Alternative>() == 48);
    assert!(align_of::<Alternative>() == 8);
    assert!(offset_of!(Alternative, span) == 0);
    assert!(offset_of!(Alternative, body) == 24);

    assert!(size_of::<Term>() == 32);
    assert!(align_of::<Term>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<BoundaryAssertion>() == 32);
    assert!(align_of::<BoundaryAssertion>() == 8);
    assert!(offset_of!(BoundaryAssertion, span) == 0);
    assert!(offset_of!(BoundaryAssertion, kind) == 24);

    assert!(size_of::<BoundaryAssertionKind>() == 1);
    assert!(align_of::<BoundaryAssertionKind>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<LookAroundAssertion>() == 80);
    assert!(align_of::<LookAroundAssertion>() == 8);
    assert!(offset_of!(LookAroundAssertion, span) == 0);
    assert!(offset_of!(LookAroundAssertion, kind) == 72);
    assert!(offset_of!(LookAroundAssertion, body) == 24);

    assert!(size_of::<LookAroundAssertionKind>() == 1);
    assert!(align_of::<LookAroundAssertionKind>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<Quantifier>() == 88);
    assert!(align_of::<Quantifier>() == 8);
    assert!(offset_of!(Quantifier, span) == 0);
    assert!(offset_of!(Quantifier, min) == 24);
    assert!(offset_of!(Quantifier, max) == 32);
    assert!(offset_of!(Quantifier, greedy) == 80);
    assert!(offset_of!(Quantifier, body) == 48);

    // Padding: 3 bytes
    assert!(size_of::<Character>() == 32);
    assert!(align_of::<Character>() == 8);
    assert!(offset_of!(Character, span) == 0);
    assert!(offset_of!(Character, kind) == 28);
    assert!(offset_of!(Character, value) == 24);

    assert!(size_of::<CharacterKind>() == 1);
    assert!(align_of::<CharacterKind>() == 1);

    // Padding: 7 bytes
    assert!(size_of::<CharacterClassEscape>() == 32);
    assert!(align_of::<CharacterClassEscape>() == 8);
    assert!(offset_of!(CharacterClassEscape, span) == 0);
    assert!(offset_of!(CharacterClassEscape, kind) == 24);

    assert!(size_of::<CharacterClassEscapeKind>() == 1);
    assert!(align_of::<CharacterClassEscapeKind>() == 1);

    // Padding: 6 bytes
    assert!(size_of::<UnicodePropertyEscape>() == 64);
    assert!(align_of::<UnicodePropertyEscape>() == 8);
    assert!(offset_of!(UnicodePropertyEscape, span) == 0);
    assert!(offset_of!(UnicodePropertyEscape, negative) == 56);
    assert!(offset_of!(UnicodePropertyEscape, strings) == 57);
    assert!(offset_of!(UnicodePropertyEscape, name) == 24);
    assert!(offset_of!(UnicodePropertyEscape, value) == 40);

    // Padding: 0 bytes
    assert!(size_of::<Dot>() == 24);
    assert!(align_of::<Dot>() == 8);
    assert!(offset_of!(Dot, span) == 0);

    // Padding: 5 bytes
    assert!(size_of::<CharacterClass>() == 56);
    assert!(align_of::<CharacterClass>() == 8);
    assert!(offset_of!(CharacterClass, span) == 0);
    assert!(offset_of!(CharacterClass, negative) == 48);
    assert!(offset_of!(CharacterClass, strings) == 49);
    assert!(offset_of!(CharacterClass, kind) == 50);
    assert!(offset_of!(CharacterClass, body) == 24);

    assert!(size_of::<CharacterClassContentsKind>() == 1);
    assert!(align_of::<CharacterClassContentsKind>() == 1);

    assert!(size_of::<CharacterClassContents>() == 16);
    assert!(align_of::<CharacterClassContents>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<CharacterClassRange>() == 88);
    assert!(align_of::<CharacterClassRange>() == 8);
    assert!(offset_of!(CharacterClassRange, span) == 0);
    assert!(offset_of!(CharacterClassRange, min) == 24);
    assert!(offset_of!(CharacterClassRange, max) == 56);

    // Padding: 7 bytes
    assert!(size_of::<ClassStringDisjunction>() == 56);
    assert!(align_of::<ClassStringDisjunction>() == 8);
    assert!(offset_of!(ClassStringDisjunction, span) == 0);
    assert!(offset_of!(ClassStringDisjunction, strings) == 48);
    assert!(offset_of!(ClassStringDisjunction, body) == 24);

    // Padding: 7 bytes
    assert!(size_of::<ClassString>() == 56);
    assert!(align_of::<ClassString>() == 8);
    assert!(offset_of!(ClassString, span) == 0);
    assert!(offset_of!(ClassString, strings) == 48);
    assert!(offset_of!(ClassString, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<CapturingGroup>() == 88);
    assert!(align_of::<CapturingGroup>() == 8);
    assert!(offset_of!(CapturingGroup, span) == 0);
    assert!(offset_of!(CapturingGroup, name) == 24);
    assert!(offset_of!(CapturingGroup, body) == 40);

    // Padding: 0 bytes
    assert!(size_of::<IgnoreGroup>() == 104);
    assert!(align_of::<IgnoreGroup>() == 8);
    assert!(offset_of!(IgnoreGroup, span) == 0);
    assert!(offset_of!(IgnoreGroup, modifiers) == 24);
    assert!(offset_of!(IgnoreGroup, body) == 56);

    // Padding: 6 bytes
    assert!(size_of::<Modifiers>() == 32);
    assert!(align_of::<Modifiers>() == 8);
    assert!(offset_of!(Modifiers, span) == 0);
    assert!(offset_of!(Modifiers, enabling) == 24);
    assert!(offset_of!(Modifiers, disabling) == 25);

    // Padding: 0 bytes
    assert!(size_of::<Modifier>() == 1);
    assert!(align_of::<Modifier>() == 1);

    // Padding: 4 bytes
    assert!(size_of::<IndexedReference>() == 32);
    assert!(align_of::<IndexedReference>() == 8);
    assert!(offset_of!(IndexedReference, span) == 0);
    assert!(offset_of!(IndexedReference, index) == 24);

    // Padding: 0 bytes
    assert!(size_of::<NamedReference>() == 40);
    assert!(align_of::<NamedReference>() == 8);
    assert!(offset_of!(NamedReference, span) == 0);
    assert!(offset_of!(NamedReference, name) == 24);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    // Padding: 0 bytes
    assert!(size_of::<Pattern>() == 64);
    assert!(align_of::<Pattern>() == 4);
    assert!(offset_of!(Pattern, span) == 0);
    assert!(offset_of!(Pattern, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Disjunction>() == 40);
    assert!(align_of::<Disjunction>() == 4);
    assert!(offset_of!(Disjunction, span) == 0);
    assert!(offset_of!(Disjunction, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<Alternative>() == 40);
    assert!(align_of::<Alternative>() == 4);
    assert!(offset_of!(Alternative, span) == 0);
    assert!(offset_of!(Alternative, body) == 24);

    assert!(size_of::<Term>() == 28);
    assert!(align_of::<Term>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<BoundaryAssertion>() == 28);
    assert!(align_of::<BoundaryAssertion>() == 4);
    assert!(offset_of!(BoundaryAssertion, span) == 0);
    assert!(offset_of!(BoundaryAssertion, kind) == 24);

    assert!(size_of::<BoundaryAssertionKind>() == 1);
    assert!(align_of::<BoundaryAssertionKind>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<LookAroundAssertion>() == 68);
    assert!(align_of::<LookAroundAssertion>() == 4);
    assert!(offset_of!(LookAroundAssertion, span) == 0);
    assert!(offset_of!(LookAroundAssertion, kind) == 64);
    assert!(offset_of!(LookAroundAssertion, body) == 24);

    assert!(size_of::<LookAroundAssertionKind>() == 1);
    assert!(align_of::<LookAroundAssertionKind>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<Quantifier>() == 80);
    assert!(align_of::<Quantifier>() == 8);
    assert!(offset_of!(Quantifier, span) == 0);
    assert!(offset_of!(Quantifier, min) == 24);
    assert!(offset_of!(Quantifier, max) == 32);
    assert!(offset_of!(Quantifier, greedy) == 76);
    assert!(offset_of!(Quantifier, body) == 48);

    // Padding: 3 bytes
    assert!(size_of::<Character>() == 32);
    assert!(align_of::<Character>() == 4);
    assert!(offset_of!(Character, span) == 0);
    assert!(offset_of!(Character, kind) == 28);
    assert!(offset_of!(Character, value) == 24);

    assert!(size_of::<CharacterKind>() == 1);
    assert!(align_of::<CharacterKind>() == 1);

    // Padding: 3 bytes
    assert!(size_of::<CharacterClassEscape>() == 28);
    assert!(align_of::<CharacterClassEscape>() == 4);
    assert!(offset_of!(CharacterClassEscape, span) == 0);
    assert!(offset_of!(CharacterClassEscape, kind) == 24);

    assert!(size_of::<CharacterClassEscapeKind>() == 1);
    assert!(align_of::<CharacterClassEscapeKind>() == 1);

    // Padding: 2 bytes
    assert!(size_of::<UnicodePropertyEscape>() == 44);
    assert!(align_of::<UnicodePropertyEscape>() == 4);
    assert!(offset_of!(UnicodePropertyEscape, span) == 0);
    assert!(offset_of!(UnicodePropertyEscape, negative) == 40);
    assert!(offset_of!(UnicodePropertyEscape, strings) == 41);
    assert!(offset_of!(UnicodePropertyEscape, name) == 24);
    assert!(offset_of!(UnicodePropertyEscape, value) == 32);

    // Padding: 0 bytes
    assert!(size_of::<Dot>() == 24);
    assert!(align_of::<Dot>() == 4);
    assert!(offset_of!(Dot, span) == 0);

    // Padding: 1 bytes
    assert!(size_of::<CharacterClass>() == 44);
    assert!(align_of::<CharacterClass>() == 4);
    assert!(offset_of!(CharacterClass, span) == 0);
    assert!(offset_of!(CharacterClass, negative) == 40);
    assert!(offset_of!(CharacterClass, strings) == 41);
    assert!(offset_of!(CharacterClass, kind) == 42);
    assert!(offset_of!(CharacterClass, body) == 24);

    assert!(size_of::<CharacterClassContentsKind>() == 1);
    assert!(align_of::<CharacterClassContentsKind>() == 1);

    assert!(size_of::<CharacterClassContents>() == 8);
    assert!(align_of::<CharacterClassContents>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<CharacterClassRange>() == 88);
    assert!(align_of::<CharacterClassRange>() == 4);
    assert!(offset_of!(CharacterClassRange, span) == 0);
    assert!(offset_of!(CharacterClassRange, min) == 24);
    assert!(offset_of!(CharacterClassRange, max) == 56);

    // Padding: 3 bytes
    assert!(size_of::<ClassStringDisjunction>() == 44);
    assert!(align_of::<ClassStringDisjunction>() == 4);
    assert!(offset_of!(ClassStringDisjunction, span) == 0);
    assert!(offset_of!(ClassStringDisjunction, strings) == 40);
    assert!(offset_of!(ClassStringDisjunction, body) == 24);

    // Padding: 3 bytes
    assert!(size_of::<ClassString>() == 44);
    assert!(align_of::<ClassString>() == 4);
    assert!(offset_of!(ClassString, span) == 0);
    assert!(offset_of!(ClassString, strings) == 40);
    assert!(offset_of!(ClassString, body) == 24);

    // Padding: 0 bytes
    assert!(size_of::<CapturingGroup>() == 72);
    assert!(align_of::<CapturingGroup>() == 4);
    assert!(offset_of!(CapturingGroup, span) == 0);
    assert!(offset_of!(CapturingGroup, name) == 24);
    assert!(offset_of!(CapturingGroup, body) == 32);

    // Padding: 0 bytes
    assert!(size_of::<IgnoreGroup>() == 92);
    assert!(align_of::<IgnoreGroup>() == 4);
    assert!(offset_of!(IgnoreGroup, span) == 0);
    assert!(offset_of!(IgnoreGroup, modifiers) == 24);
    assert!(offset_of!(IgnoreGroup, body) == 52);

    // Padding: 2 bytes
    assert!(size_of::<Modifiers>() == 28);
    assert!(align_of::<Modifiers>() == 4);
    assert!(offset_of!(Modifiers, span) == 0);
    assert!(offset_of!(Modifiers, enabling) == 24);
    assert!(offset_of!(Modifiers, disabling) == 25);

    // Padding: 0 bytes
    assert!(size_of::<Modifier>() == 1);
    assert!(align_of::<Modifier>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<IndexedReference>() == 28);
    assert!(align_of::<IndexedReference>() == 4);
    assert!(offset_of!(IndexedReference, span) == 0);
    assert!(offset_of!(IndexedReference, index) == 24);

    // Padding: 0 bytes
    assert!(size_of::<NamedReference>() == 32);
    assert!(align_of::<NamedReference>() == 4);
    assert!(offset_of!(NamedReference, span) == 0);
    assert!(offset_of!(NamedReference, name) == 24);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
