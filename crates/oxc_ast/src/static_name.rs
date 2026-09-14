use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
};

use oxc_str::JSStr;

/// The JavaScript string value of a statically known property key.
///
/// Identifiers and string literals borrow their storage. Numeric and regular
/// expression keys own the UTF-8 text produced by their string conversion.
#[derive(Clone)]
pub enum StaticName<'a> {
    /// A name already stored in the AST.
    Borrowed(JSStr<'a>),
    /// The string conversion of a non-string literal.
    Owned(String),
}

impl StaticName<'_> {
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

impl<'a> StaticName<'a> {
    /// Convert a name into UTF-8 when a consumer requires a Rust string.
    ///
    /// A failure means the name contains a lone surrogate, not that it is dynamic.
    pub fn into_utf8(self) -> Option<Cow<'a, str>> {
        match self {
            Self::Borrowed(name) => name.as_str().map(Cow::Borrowed),
            Self::Owned(name) => Some(Cow::Owned(name)),
        }
    }
}

impl<'a> From<&'a str> for StaticName<'a> {
    fn from(name: &'a str) -> Self {
        Self::Borrowed(JSStr::from(name))
    }
}

impl<'a> From<JSStr<'a>> for StaticName<'a> {
    fn from(name: JSStr<'a>) -> Self {
        Self::Borrowed(name)
    }
}

impl<'a> From<oxc_str::Ident<'a>> for StaticName<'a> {
    fn from(name: oxc_str::Ident<'a>) -> Self {
        Self::Borrowed(JSStr::from(name.as_str()))
    }
}

impl PartialEq for StaticName<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_js_str() == other.as_js_str()
    }
}

impl Eq for StaticName<'_> {}

impl Hash for StaticName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_js_str().hash(state);
    }
}

impl PartialEq<str> for StaticName<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_js_str() == other
    }
}

impl PartialEq<&str> for StaticName<'_> {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<oxc_str::Ident<'_>> for StaticName<'_> {
    fn eq(&self, other: &oxc_str::Ident<'_>) -> bool {
        self == other.as_str()
    }
}

impl fmt::Display for StaticName<'_> {
    /// Display a name for diagnostics, escaping lone surrogates.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.as_str() {
            return f.write_str(name);
        }
        for ch in self.as_js_str().chars() {
            if let Some(ch) = ch.to_char() {
                write!(f, "{ch}")?;
            } else {
                write!(f, "\\u{:04X}", ch.to_u32())?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for StaticName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_js_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::StaticName;
    use oxc_allocator::Allocator;
    use oxc_str::{JSStr, JSStrBuilder};
    use rustc_hash::FxHashSet;

    #[test]
    fn names_preserve_identity_across_storage_and_surrogates() {
        let allocator = Allocator::new();
        let mut names = FxHashSet::default();
        assert!(names.insert(StaticName::Owned("42".into())));
        assert!(!names.insert(StaticName::from("42")));
        for units in [&[0xD800][..], &[0xD801], &[0xDC00], &[0xD800, 0xDC00], &[0xFFFD]] {
            let mut builder = JSStrBuilder::new_in(&allocator);
            builder.push_utf16(units);
            let name = StaticName::Borrowed(builder.into_js_str());
            assert!(names.insert(name.clone()));
            assert!(!names.insert(name));
        }
        assert!(names.insert(StaticName::from(r"\uD800")));
        assert!(!names.insert(StaticName::Borrowed(JSStr::from("𐀀"))));
        assert_eq!(names.len(), 7);
    }
}
