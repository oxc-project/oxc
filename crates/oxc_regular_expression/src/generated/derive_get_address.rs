// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/get_address.rs`

#![allow(clippy::match_same_arms)]

use oxc_allocator::{Address, GetAddress};

use crate::ast::*;

impl GetAddress for CharacterClassContents<'_> {
    // `#[inline]` because compiler should boil this down to a single assembly instruction
    #[inline]
    fn address(&self) -> Address {
        match self {
            Self::CharacterClassRange(it) => GetAddress::address(it),
            Self::CharacterClassEscape(it) => GetAddress::address(it),
            Self::UnicodePropertyEscape(it) => GetAddress::address(it),
            Self::Character(it) => GetAddress::address(it),
            Self::NestedCharacterClass(it) => GetAddress::address(it),
            Self::ClassStringDisjunction(it) => GetAddress::address(it),
        }
    }
}
