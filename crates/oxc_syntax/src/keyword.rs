use phf::{phf_set, Set};

#[inline]
pub fn is_reserved_keyword_or_global_object(s: &str) -> bool {
    is_reserved_keyword(s) || is_global_object(s)
}

#[inline]
pub fn is_reserved_keyword(s: &str) -> bool {
    RESERVED_KEYWORDS.contains(s)
}

/// Checks `Infinity`, `NaN`, `globalThis` and `undefined`
#[inline]
pub fn is_global_object(s: &str) -> bool {
    GLOBAL_OBJECTS.contains(s)
}

/// Value properties of the global object
///
/// Reference: <https://tc39.es/ecma262/multipage/global-object.html#sec-value-properties-of-the-global-object>
pub const GLOBAL_OBJECTS: Set<&'static str> = phf_set! {
    "Infinity",
    "NaN",
    "globalThis",
    "undefined",
};

/// All reserved keywords, including keywords that are contextually disallowed as identifiers.
///
/// Reference: <https://tc39.es/ecma262/#prod-ReservedWord>
pub const RESERVED_KEYWORDS: Set<&'static str> = phf_set! {
    // contextually disallowed as identifiers
    "let",
    "static",
    // future reserved keywords
    "implements",
    "interface",
    "package",
    "private",
    "protected",
    "public",
    // reserved word
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
};
