use proc_macro::TokenStream;
use std::str::FromStr;

/// Attach to AST node type (struct or enum), to signal to codegen to create visitor for this type.
///
/// Macro's role is not to generate code - it's purely a means to communicate information to the codegen.
///
/// Only thing macro does is add `#[derive(Ast)]` to the item.
/// Deriving `Ast` does nothing, but supports `#[scope]`, `#[visit]`, and other attrs on struct fields.
/// These "helper" attributes are also signals to the codegen, and do nothing in themselves.
///
/// This is a workaround for Rust not supporting helper attributes for `proc_macro_attribute` macros,
/// so we need to use a derive macro to get that support.
///
/// Use native Rust `TokenStream`, to avoid dependency on slow-compiling crates like `syn` and `quote`.
#[proc_macro_attribute]
#[allow(clippy::missing_panics_doc)]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut stream = TokenStream::from_str("#[derive(::oxc_ast_macros::Ast)]").unwrap();
    stream.extend(input);
    stream
}

/// Dummy derive macro for a non-existent trait `Ast`.
///
/// Does not generate any code.
/// Only purpose is to allow using `#[scope]`, `#[visit]`, and other attrs in the AST node type defs.
#[proc_macro_derive(Ast, attributes(scope, visit, span, serde, tsify))]
pub fn ast_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
