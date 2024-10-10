use syn::{parse::ParseStream, Result, Token};

/// Checks if `cond` is `true`, returning [`Err(syn::Error)`] with `msg` if it's not.
/// ## Example
/// ```ignore
/// use syn::{parse::Parse, LitStr};
///
/// struct Foo(LitStr);
///
/// impl Parse for Foo {
///     fn parse(input: ParseStream) -> Result<Self> {
///         let s = input.parse::<LitStr>()?;
///         parse_assert!(s.value() == "foo", s, "Expected 'foo'");
///         Ok(Foo(s))
///     }
/// }
/// ```
macro_rules! parse_assert {
    ($cond:expr, $toks:expr, $msg:expr) => {
        if !($cond) {
            return Err(syn::Error::new_spanned($toks, $msg));
        }
    };
}
pub(crate) use parse_assert;

/// Consume a comma token if it's present, noop otherwise
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn eat_comma(input: &ParseStream) -> Result<()> {
    if input.peek(Token!(,)) {
        input.parse::<Token!(,)>()?;
    }
    Ok(())
}
