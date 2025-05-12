// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::ast::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<Pattern>() == 48);
    assert!(align_of::<Pattern>() == 8);
    assert!(offset_of!(Pattern, span) == 0);
    assert!(offset_of!(Pattern, body) == 8);

    assert!(size_of::<Disjunction>() == 40);
    assert!(align_of::<Disjunction>() == 8);
    assert!(offset_of!(Disjunction, span) == 0);
    assert!(offset_of!(Disjunction, body) == 8);

    assert!(size_of::<Alternative>() == 40);
    assert!(align_of::<Alternative>() == 8);
    assert!(offset_of!(Alternative, span) == 0);
    assert!(offset_of!(Alternative, body) == 8);

    assert!(size_of::<Term>() == 16);
    assert!(align_of::<Term>() == 8);

    assert!(size_of::<BoundaryAssertion>() == 16);
    assert!(align_of::<BoundaryAssertion>() == 8);
    assert!(offset_of!(BoundaryAssertion, span) == 0);
    assert!(offset_of!(BoundaryAssertion, kind) == 8);

    assert!(size_of::<BoundaryAssertionKind>() == 1);
    assert!(align_of::<BoundaryAssertionKind>() == 1);

    assert!(size_of::<LookAroundAssertion>() == 56);
    assert!(align_of::<LookAroundAssertion>() == 8);
    assert!(offset_of!(LookAroundAssertion, span) == 0);
    assert!(offset_of!(LookAroundAssertion, kind) == 8);
    assert!(offset_of!(LookAroundAssertion, body) == 16);

    assert!(size_of::<LookAroundAssertionKind>() == 1);
    assert!(align_of::<LookAroundAssertionKind>() == 1);

    assert!(size_of::<Quantifier>() == 56);
    assert!(align_of::<Quantifier>() == 8);
    assert!(offset_of!(Quantifier, span) == 0);
    assert!(offset_of!(Quantifier, min) == 8);
    assert!(offset_of!(Quantifier, max) == 16);
    assert!(offset_of!(Quantifier, greedy) == 32);
    assert!(offset_of!(Quantifier, body) == 40);

    assert!(size_of::<Character>() == 16);
    assert!(align_of::<Character>() == 8);
    assert!(offset_of!(Character, span) == 0);
    assert!(offset_of!(Character, kind) == 8);
    assert!(offset_of!(Character, value) == 12);

    assert!(size_of::<CharacterKind>() == 1);
    assert!(align_of::<CharacterKind>() == 1);

    assert!(size_of::<CharacterClassEscape>() == 16);
    assert!(align_of::<CharacterClassEscape>() == 8);
    assert!(offset_of!(CharacterClassEscape, span) == 0);
    assert!(offset_of!(CharacterClassEscape, kind) == 8);

    assert!(size_of::<CharacterClassEscapeKind>() == 1);
    assert!(align_of::<CharacterClassEscapeKind>() == 1);

    assert!(size_of::<UnicodePropertyEscape>() == 48);
    assert!(align_of::<UnicodePropertyEscape>() == 8);
    assert!(offset_of!(UnicodePropertyEscape, span) == 0);
    assert!(offset_of!(UnicodePropertyEscape, negative) == 8);
    assert!(offset_of!(UnicodePropertyEscape, strings) == 9);
    assert!(offset_of!(UnicodePropertyEscape, name) == 16);
    assert!(offset_of!(UnicodePropertyEscape, value) == 32);

    assert!(size_of::<Dot>() == 8);
    assert!(align_of::<Dot>() == 8);
    assert!(offset_of!(Dot, span) == 0);

    assert!(size_of::<CharacterClass>() == 48);
    assert!(align_of::<CharacterClass>() == 8);
    assert!(offset_of!(CharacterClass, span) == 0);
    assert!(offset_of!(CharacterClass, negative) == 8);
    assert!(offset_of!(CharacterClass, strings) == 9);
    assert!(offset_of!(CharacterClass, kind) == 10);
    assert!(offset_of!(CharacterClass, body) == 16);

    assert!(size_of::<CharacterClassContentsKind>() == 1);
    assert!(align_of::<CharacterClassContentsKind>() == 1);

    assert!(size_of::<CharacterClassContents>() == 16);
    assert!(align_of::<CharacterClassContents>() == 8);

    assert!(size_of::<CharacterClassRange>() == 40);
    assert!(align_of::<CharacterClassRange>() == 8);
    assert!(offset_of!(CharacterClassRange, span) == 0);
    assert!(offset_of!(CharacterClassRange, min) == 8);
    assert!(offset_of!(CharacterClassRange, max) == 24);

    assert!(size_of::<ClassStringDisjunction>() == 48);
    assert!(align_of::<ClassStringDisjunction>() == 8);
    assert!(offset_of!(ClassStringDisjunction, span) == 0);
    assert!(offset_of!(ClassStringDisjunction, strings) == 8);
    assert!(offset_of!(ClassStringDisjunction, body) == 16);

    assert!(size_of::<ClassString>() == 48);
    assert!(align_of::<ClassString>() == 8);
    assert!(offset_of!(ClassString, span) == 0);
    assert!(offset_of!(ClassString, strings) == 8);
    assert!(offset_of!(ClassString, body) == 16);

    assert!(size_of::<CapturingGroup>() == 64);
    assert!(align_of::<CapturingGroup>() == 8);
    assert!(offset_of!(CapturingGroup, span) == 0);
    assert!(offset_of!(CapturingGroup, name) == 8);
    assert!(offset_of!(CapturingGroup, body) == 24);

    assert!(size_of::<IgnoreGroup>() == 64);
    assert!(align_of::<IgnoreGroup>() == 8);
    assert!(offset_of!(IgnoreGroup, span) == 0);
    assert!(offset_of!(IgnoreGroup, modifiers) == 8);
    assert!(offset_of!(IgnoreGroup, body) == 24);

    assert!(size_of::<Modifiers>() == 16);
    assert!(align_of::<Modifiers>() == 8);
    assert!(offset_of!(Modifiers, span) == 0);
    assert!(offset_of!(Modifiers, enabling) == 8);
    assert!(offset_of!(Modifiers, disabling) == 11);

    assert!(size_of::<Modifier>() == 3);
    assert!(align_of::<Modifier>() == 1);
    assert!(offset_of!(Modifier, ignore_case) == 0);
    assert!(offset_of!(Modifier, multiline) == 1);
    assert!(offset_of!(Modifier, sticky) == 2);

    assert!(size_of::<IndexedReference>() == 16);
    assert!(align_of::<IndexedReference>() == 8);
    assert!(offset_of!(IndexedReference, span) == 0);
    assert!(offset_of!(IndexedReference, index) == 8);

    assert!(size_of::<NamedReference>() == 24);
    assert!(align_of::<NamedReference>() == 8);
    assert!(offset_of!(NamedReference, span) == 0);
    assert!(offset_of!(NamedReference, name) == 8);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(size_of::<Pattern>() == 32);
    assert!(align_of::<Pattern>() == 4);
    assert!(offset_of!(Pattern, span) == 0);
    assert!(offset_of!(Pattern, body) == 8);

    assert!(size_of::<Disjunction>() == 24);
    assert!(align_of::<Disjunction>() == 4);
    assert!(offset_of!(Disjunction, span) == 0);
    assert!(offset_of!(Disjunction, body) == 8);

    assert!(size_of::<Alternative>() == 24);
    assert!(align_of::<Alternative>() == 4);
    assert!(offset_of!(Alternative, span) == 0);
    assert!(offset_of!(Alternative, body) == 8);

    assert!(size_of::<Term>() == 12);
    assert!(align_of::<Term>() == 4);

    assert!(size_of::<BoundaryAssertion>() == 12);
    assert!(align_of::<BoundaryAssertion>() == 4);
    assert!(offset_of!(BoundaryAssertion, span) == 0);
    assert!(offset_of!(BoundaryAssertion, kind) == 8);

    assert!(size_of::<BoundaryAssertionKind>() == 1);
    assert!(align_of::<BoundaryAssertionKind>() == 1);

    assert!(size_of::<LookAroundAssertion>() == 36);
    assert!(align_of::<LookAroundAssertion>() == 4);
    assert!(offset_of!(LookAroundAssertion, span) == 0);
    assert!(offset_of!(LookAroundAssertion, kind) == 8);
    assert!(offset_of!(LookAroundAssertion, body) == 12);

    assert!(size_of::<LookAroundAssertionKind>() == 1);
    assert!(align_of::<LookAroundAssertionKind>() == 1);

    assert!(size_of::<Quantifier>() == 48);
    assert!(align_of::<Quantifier>() == 8);
    assert!(offset_of!(Quantifier, span) == 0);
    assert!(offset_of!(Quantifier, min) == 8);
    assert!(offset_of!(Quantifier, max) == 16);
    assert!(offset_of!(Quantifier, greedy) == 32);
    assert!(offset_of!(Quantifier, body) == 36);

    assert!(size_of::<Character>() == 16);
    assert!(align_of::<Character>() == 4);
    assert!(offset_of!(Character, span) == 0);
    assert!(offset_of!(Character, kind) == 8);
    assert!(offset_of!(Character, value) == 12);

    assert!(size_of::<CharacterKind>() == 1);
    assert!(align_of::<CharacterKind>() == 1);

    assert!(size_of::<CharacterClassEscape>() == 12);
    assert!(align_of::<CharacterClassEscape>() == 4);
    assert!(offset_of!(CharacterClassEscape, span) == 0);
    assert!(offset_of!(CharacterClassEscape, kind) == 8);

    assert!(size_of::<CharacterClassEscapeKind>() == 1);
    assert!(align_of::<CharacterClassEscapeKind>() == 1);

    assert!(size_of::<UnicodePropertyEscape>() == 28);
    assert!(align_of::<UnicodePropertyEscape>() == 4);
    assert!(offset_of!(UnicodePropertyEscape, span) == 0);
    assert!(offset_of!(UnicodePropertyEscape, negative) == 8);
    assert!(offset_of!(UnicodePropertyEscape, strings) == 9);
    assert!(offset_of!(UnicodePropertyEscape, name) == 12);
    assert!(offset_of!(UnicodePropertyEscape, value) == 20);

    assert!(size_of::<Dot>() == 8);
    assert!(align_of::<Dot>() == 4);
    assert!(offset_of!(Dot, span) == 0);

    assert!(size_of::<CharacterClass>() == 28);
    assert!(align_of::<CharacterClass>() == 4);
    assert!(offset_of!(CharacterClass, span) == 0);
    assert!(offset_of!(CharacterClass, negative) == 8);
    assert!(offset_of!(CharacterClass, strings) == 9);
    assert!(offset_of!(CharacterClass, kind) == 10);
    assert!(offset_of!(CharacterClass, body) == 12);

    assert!(size_of::<CharacterClassContentsKind>() == 1);
    assert!(align_of::<CharacterClassContentsKind>() == 1);

    assert!(size_of::<CharacterClassContents>() == 8);
    assert!(align_of::<CharacterClassContents>() == 4);

    assert!(size_of::<CharacterClassRange>() == 40);
    assert!(align_of::<CharacterClassRange>() == 4);
    assert!(offset_of!(CharacterClassRange, span) == 0);
    assert!(offset_of!(CharacterClassRange, min) == 8);
    assert!(offset_of!(CharacterClassRange, max) == 24);

    assert!(size_of::<ClassStringDisjunction>() == 28);
    assert!(align_of::<ClassStringDisjunction>() == 4);
    assert!(offset_of!(ClassStringDisjunction, span) == 0);
    assert!(offset_of!(ClassStringDisjunction, strings) == 8);
    assert!(offset_of!(ClassStringDisjunction, body) == 12);

    assert!(size_of::<ClassString>() == 28);
    assert!(align_of::<ClassString>() == 4);
    assert!(offset_of!(ClassString, span) == 0);
    assert!(offset_of!(ClassString, strings) == 8);
    assert!(offset_of!(ClassString, body) == 12);

    assert!(size_of::<CapturingGroup>() == 40);
    assert!(align_of::<CapturingGroup>() == 4);
    assert!(offset_of!(CapturingGroup, span) == 0);
    assert!(offset_of!(CapturingGroup, name) == 8);
    assert!(offset_of!(CapturingGroup, body) == 16);

    assert!(size_of::<IgnoreGroup>() == 48);
    assert!(align_of::<IgnoreGroup>() == 4);
    assert!(offset_of!(IgnoreGroup, span) == 0);
    assert!(offset_of!(IgnoreGroup, modifiers) == 8);
    assert!(offset_of!(IgnoreGroup, body) == 24);

    assert!(size_of::<Modifiers>() == 16);
    assert!(align_of::<Modifiers>() == 4);
    assert!(offset_of!(Modifiers, span) == 0);
    assert!(offset_of!(Modifiers, enabling) == 8);
    assert!(offset_of!(Modifiers, disabling) == 11);

    assert!(size_of::<Modifier>() == 3);
    assert!(align_of::<Modifier>() == 1);
    assert!(offset_of!(Modifier, ignore_case) == 0);
    assert!(offset_of!(Modifier, multiline) == 1);
    assert!(offset_of!(Modifier, sticky) == 2);

    assert!(size_of::<IndexedReference>() == 12);
    assert!(align_of::<IndexedReference>() == 4);
    assert!(offset_of!(IndexedReference, span) == 0);
    assert!(offset_of!(IndexedReference, index) == 8);

    assert!(size_of::<NamedReference>() == 16);
    assert!(align_of::<NamedReference>() == 4);
    assert!(offset_of!(NamedReference, span) == 0);
    assert!(offset_of!(NamedReference, name) == 8);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
