use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
};

use oxc_allocator::GetAllocator;
use oxc_str::{CompactStr, Ident, JSStr};

/// The JavaScript string value of a statically known property key.
///
/// Static means statically determinable, not a `static` class member.
/// Names already stored in the AST borrow their storage. Numeric keys own their Rust `f64`
/// `Display` text, preserving the existing naming behavior. Both forms compare and hash by text,
/// so `{ 1: a, "1": b }` declares the same name twice.
#[derive(Clone)]
pub enum StaticPropertyName<'a> {
    /// A name already stored in the AST.
    Borrowed(JSStr<'a>),
    /// The string conversion of a non-string literal.
    Owned(CompactStr),
}

impl StaticPropertyName<'_> {
    /// Borrow the complete JavaScript property name.
    pub fn as_js_str(&self) -> JSStr<'_> {
        match self {
            Self::Borrowed(name) => *name,
            Self::Owned(name) => JSStr::from(name.as_str()),
        }
    }

    /// Borrow a UTF-8 name, if it contains no lone surrogate.
    pub fn as_str(&self) -> Option<&str> {
        self.as_js_str().as_str()
    }
}

impl<'a> StaticPropertyName<'a> {
    /// Return the name with arena storage, borrowing AST strings without copying them.
    pub fn into_js_str_in(self, allocator: &impl GetAllocator<'a>) -> JSStr<'a> {
        match self {
            Self::Borrowed(name) => name,
            Self::Owned(name) => JSStr::from_str_in(name.as_str(), allocator),
        }
    }

    /// Convert a name into UTF-8 when a consumer requires a Rust string.
    ///
    /// A failure means the name contains a lone surrogate, not that it is dynamic.
    pub fn into_cow_str(self) -> Option<Cow<'a, str>> {
        match self {
            Self::Borrowed(name) => name.as_str().map(Cow::Borrowed),
            Self::Owned(name) => Some(Cow::Owned(name.into_string())),
        }
    }
}

impl<'a> From<&'a str> for StaticPropertyName<'a> {
    fn from(name: &'a str) -> Self {
        Self::Borrowed(JSStr::from(name))
    }
}

impl<'a> From<JSStr<'a>> for StaticPropertyName<'a> {
    fn from(name: JSStr<'a>) -> Self {
        Self::Borrowed(name)
    }
}

impl<'a> From<Ident<'a>> for StaticPropertyName<'a> {
    fn from(name: Ident<'a>) -> Self {
        Self::Borrowed(name.as_js_str())
    }
}

impl PartialEq for StaticPropertyName<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_js_str() == other.as_js_str()
    }
}

impl Eq for StaticPropertyName<'_> {}

impl Hash for StaticPropertyName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_js_str().hash(state);
    }
}

impl PartialEq<str> for StaticPropertyName<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_js_str() == other
    }
}

impl PartialEq<&str> for StaticPropertyName<'_> {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<Ident<'_>> for StaticPropertyName<'_> {
    fn eq(&self, other: &Ident<'_>) -> bool {
        self.as_js_str() == *other
    }
}

impl fmt::Display for StaticPropertyName<'_> {
    /// Display a name for diagnostics, escaping lone surrogates in the same lowercase spelling as
    /// the Debug form and printed output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_js_str().display(), f)
    }
}

impl PartialEq<JSStr<'_>> for StaticPropertyName<'_> {
    fn eq(&self, other: &JSStr<'_>) -> bool {
        self.as_js_str() == *other
    }
}

impl fmt::Debug for StaticPropertyName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_js_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::StaticPropertyName;
    use oxc_allocator::Allocator;
    use oxc_str::{JSStr, JSStrBuilder, format_compact_str};
    use rustc_hash::FxHashSet;

    #[test]
    fn names_preserve_identity_across_storage_and_surrogates() {
        let allocator = Allocator::new();
        let mut names = FxHashSet::default();
        assert!(names.insert(StaticPropertyName::Owned("42".into())));
        assert!(!names.insert(StaticPropertyName::from("42")));
        for units in
            [&[0xD800][..], &[0xD801], &[0xDC00], &[0xD800, 0xDC00], &[0xDC00, 0xD800], &[0xFFFD]]
        {
            let mut builder = JSStrBuilder::new_in(&&allocator);
            builder.push_utf16(units);
            let name = StaticPropertyName::Borrowed(builder.into_js_str());
            assert!(names.insert(name.clone()));
            assert!(!names.insert(name));
        }
        assert!(names.insert(StaticPropertyName::from(r"\uD800")));
        assert!(!names.insert(StaticPropertyName::Borrowed(JSStr::from("𐀀"))));
        assert_eq!(names.len(), 8);
    }

    #[test]
    fn regex_names_use_canonical_flags_without_raw_text() {
        use crate::ast::{PropertyKey, RegExp, RegExpFlags, RegExpPattern};
        use crate::builder::AstBuilder;
        use oxc_span::SPAN;

        let allocator = Allocator::new();
        let ast = AstBuilder::new(&allocator);
        for raw in [None, Some("/a/ig".into()), Some("/a/gi".into())] {
            let regex = RegExp {
                pattern: RegExpPattern { text: "a".into(), pattern: None },
                flags: RegExpFlags::I | RegExpFlags::G,
            };
            let key = PropertyKey::new_reg_exp_literal(SPAN, regex, raw, &ast);
            assert_eq!(key.static_name().unwrap(), StaticPropertyName::from("/a/gi"));
        }
    }

    #[test]
    fn numbers_are_named_by_f64_display() {
        for value in [1e21, 1e-7, -f64::MAX, -5e-324, -f64::MIN_POSITIVE, f64::NAN, -0.0] {
            let text = value.to_string();
            let name = StaticPropertyName::Owned(format_compact_str!("{value}"));
            assert_eq!(name.to_string(), text);
            assert_eq!(name, StaticPropertyName::from(text.as_str()));
        }
    }
}
