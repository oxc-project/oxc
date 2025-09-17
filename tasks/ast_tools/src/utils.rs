use std::borrow::Cow;

use indexmap::{IndexMap, IndexSet};
use phf::{Set as PhfSet, phf_set};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use rustc_hash::FxBuildHasher;
use syn::{Ident, LitInt};

pub type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
pub type FxIndexSet<K> = IndexSet<K, FxBuildHasher>;

/// Reserved word in Rust.
/// From <https://doc.rust-lang.org/reference/keywords.html>.
static RESERVED_NAMES: PhfSet<&'static str> = phf_set! {
    // Strict keywords
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
    "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "dyn",
    // Reserved keywords
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized",
    "virtual", "yield", "try",
    // Weak keywords
    "macro_rules", "union", // "dyn" also listed as a weak keyword, but is already on strict list
};

/// Returns `true` if `name` is a reserved word in Rust.
pub fn is_reserved_name(name: &str) -> bool {
    RESERVED_NAMES.contains(name)
}

/// Create an [`Ident`] from a string.
///
/// If the name is a reserved word, it's prepended with `r#`.
/// e.g. `type` -> `r#type`.
///
/// [`Ident`]: struct@Ident
pub fn create_ident(name: &str) -> Ident {
    if is_reserved_name(name) { format_ident!("r#{name}") } else { create_safe_ident(name) }
}

/// Create an [`Ident`] from a string, without checking it's not a reserved word.
///
/// The provided `name` for the ident must not be a reserved word.
///
/// [`Ident`]: struct@Ident
pub fn create_safe_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

/// Create an identifier from a string.
///
/// If the name is a reserved word, it's prepended with `r#`.
/// e.g. `type` -> `r#type`.
pub fn create_ident_tokens(name: &str) -> TokenStream {
    if name.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        let lit = LitInt::new(name, Span::call_site());
        quote!(#lit)
    } else {
        let ident = create_ident(name);
        quote!(#ident)
    }
}

/// Convert [`Ident`] to `String`, removing `r#` from start.
///
/// [`Ident`]: struct@Ident
pub fn ident_name(ident: &Ident) -> String {
    let name = ident.to_string();
    if let Some(unprefixed) = name.strip_prefix("r#") { unprefixed.to_string() } else { name }
}

/// Convert integer to [`LitInt`].
///
/// This prints without a `usize` postfix. i.e. `123` not `123usize`.
///
/// [`LitInt`]: struct@LitInt
pub fn number_lit<N: Into<u64>>(n: N) -> LitInt {
    LitInt::new(&n.into().to_string(), Span::call_site())
}

/// Pluralize name.
pub fn pluralize(name: &str) -> String {
    if name.ends_with("child") || name.ends_with("Child") {
        format!("{name}ren")
    } else {
        match name.as_bytes().last() {
            Some(b's') => format!("{name}es"),
            Some(b'y') => format!("{}ies", &name[..name.len() - 1]),
            _ => format!("{name}s"),
        }
    }
}

/// Upper case first character of a string.
pub fn upper_case_first(s: &str) -> Cow<'_, str> {
    let mut chars = s.chars();
    let first_char = chars.next().unwrap();
    if first_char.is_uppercase() {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(first_char.to_uppercase().chain(chars).collect::<String>())
    }
}

/// Macro to `format!` arguments, and wrap the formatted string in a `Cow::Owned`.
macro_rules! format_cow {
    ($($tokens:tt)+) => {
        std::borrow::Cow::Owned(format!($($tokens)+))
    }
}
pub(crate) use format_cow;

/// Macro to `write!` and unwrap result.
macro_rules! write_it {
    ($($tokens:tt)+) => {{
        use std::fmt::Write;
        write!($($tokens)+).unwrap();
    }}
}
pub(crate) use write_it;

/// Macro to create a `String`.
///
/// e.g. `string!("hello")` -> "hello".to_string().
macro_rules! string {
    ($s:literal) => {
        $s.to_string()
    };
}
pub(crate) use string;
