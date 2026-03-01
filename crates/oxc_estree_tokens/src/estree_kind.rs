//! [`ESTreeKind`] — compact token kind for raw transfer to JS.
//!
//! Maps the full [`Kind`] enum (169 variants) to a small set of 12 ESTree token types.
//!
//! ## How it works
//!
//! `ESTreeKind` is `#[repr(u8)]` with discriminants chosen to match the first 12 `Kind`
//! discriminants (0–11). [`ESTreeKind::to_kind`] maps each variant back to the `Kind` with
//! the same discriminant, so `estree_kind.to_kind()` is a no-op.
//!
//! After conversion, the kind byte in `Token` is the `ESTreeKind` discriminant.
//! `token.kind()` is nonsense if interpreted as a `Kind`, but JS side reads it as an `ESTreeKind`
//! and looks up the token type string from a 12-entry table corresponding to `ESTreeKind`'s variants.

use oxc_parser::Kind;

/// Compact ESTree token type, used for raw transfer to JS.
///
/// Each variant corresponds to one ESTree token type string.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ESTreeKind {
    Identifier = 0,
    Keyword = 1,
    PrivateIdentifier = 2,
    Punctuator = 3,
    Numeric = 4,
    String = 5,
    Boolean = 6,
    Null = 7,
    RegularExpression = 8,
    Template = 9,
    JSXText = 10,
    JSXIdentifier = 11,
}

const KINDS_LEN: usize = Kind::VARIANTS.len();

// Verify number of `Kind` variants, so we catch if new variants are added
const _: () = assert!(KINDS_LEN == 169);

// Verify that the `Kind` discriminants we rely on for `to_kind` haven't shifted.
// If any of these assertions fail, the `to_kind` mapping needs updating.
const _: () = {
    assert!(Kind::Eof as u8 == 0);
    assert!(Kind::Undetermined as u8 == 1);
    assert!(Kind::Skip as u8 == 2);
    assert!(Kind::HashbangComment as u8 == 3);
    assert!(Kind::Ident as u8 == 4);
    assert!(Kind::Await as u8 == 5);
    assert!(Kind::Break as u8 == 6);
    assert!(Kind::Case as u8 == 7);
    assert!(Kind::Catch as u8 == 8);
    assert!(Kind::Class as u8 == 9);
    assert!(Kind::Const as u8 == 10);
    assert!(Kind::Continue as u8 == 11);
};

impl ESTreeKind {
    /// Convert to the [`Kind`] with the same discriminant value.
    //
    // `#[inline(always)]` because this boils down to a no-op.
    // `ESTreeKind` discriminants and the discriminants of corresponding `Kind` values are the same.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn to_kind(self) -> Kind {
        match self {
            Self::Identifier => Kind::Eof,
            Self::Keyword => Kind::Undetermined,
            Self::PrivateIdentifier => Kind::Skip,
            Self::Punctuator => Kind::HashbangComment,
            Self::Numeric => Kind::Ident,
            Self::String => Kind::Await,
            Self::Boolean => Kind::Break,
            Self::Null => Kind::Case,
            Self::RegularExpression => Kind::Catch,
            Self::Template => Kind::Class,
            Self::JSXText => Kind::Const,
            Self::JSXIdentifier => Kind::Continue,
        }
    }

    /// Convert a [`Kind`] to the corresponding `ESTreeKind` via a lookup table.
    #[inline]
    pub(crate) fn from_kind(kind: Kind) -> Self {
        CONVERSION_TABLE[kind as usize]
    }
}

/// Lookup table mapping each [`Kind`] discriminant to its [`ESTreeKind`].
///
/// Indexed by `kind as u8 as usize`. Has `KINDS_LEN` entries (one per [`Kind`] variant).
static CONVERSION_TABLE: [ESTreeKind; KINDS_LEN] = {
    // Classify each `Kind` into an `ESTreeKind`
    let mut table = [ESTreeKind::Identifier; KINDS_LEN];

    let mut i = 0;
    while i < KINDS_LEN {
        let kind = Kind::VARIANTS[i];
        table[i] = match kind {
            Kind::Ident | Kind::Await => ESTreeKind::Identifier,
            Kind::True | Kind::False => ESTreeKind::Boolean,
            Kind::Null => ESTreeKind::Null,
            Kind::Str => ESTreeKind::String,
            Kind::RegExp => ESTreeKind::RegularExpression,
            Kind::NoSubstitutionTemplate
            | Kind::TemplateHead
            | Kind::TemplateMiddle
            | Kind::TemplateTail => ESTreeKind::Template,
            Kind::PrivateIdentifier => ESTreeKind::PrivateIdentifier,
            Kind::JSXText => ESTreeKind::JSXText,
            _ if kind.is_number() => ESTreeKind::Numeric,
            _ if kind.is_contextual_keyword() => ESTreeKind::Identifier,
            _ if kind.is_any_keyword() => ESTreeKind::Keyword,
            _ => ESTreeKind::Punctuator,
        };
        i += 1;
    }

    table
};
