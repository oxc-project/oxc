//! Shared property-key syntax metadata.

/// The syntactic class of a property key before a transform converted it into a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyKeyOrigin {
    /// The string came from an identifier-like key such as `obj.foo`.
    Unquoted,
    /// The string came from a quoted key such as `obj["foo"]`.
    Quoted,
}
