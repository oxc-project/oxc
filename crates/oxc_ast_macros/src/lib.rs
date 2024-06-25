use proc_macro::TokenStream;
use std::str::FromStr;

/// Attach to AST node type (struct or enum), to signal to codegen to create visitor for this type.
///
/// Macro does not generate any code - it's purely a means to communicate information to the codegen.
///
/// Only thing macro does is add `#[derive(VisitedNode)]` to the item.
/// Deriving `VisitedNode` does nothing, but supports the `#[scope]` attr on struct fields.
/// This is a workaround for Rust not supporting helper attributes for `proc_macro_attribute` macros,
/// so we need to use a derive macro to get that support.
///
/// Use native Rust `TokenStream`, to avoid dependency on slow-compiling crates like `syn` and `quote`.
#[proc_macro_attribute]
#[allow(clippy::missing_panics_doc)]
pub fn visited_node(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut stream = TokenStream::from_str("#[derive(::oxc_ast_macros::VisitedNode)]").unwrap();
    stream.extend(input);
    stream
}

/// Dummy derive macro for a non-existent trait `VisitedNode`.
///
/// Does not generate any code, only purpose is to allow using `#[scope]` attr in the type def.
#[proc_macro_derive(VisitedNode, attributes(scope))]
pub fn visited_node_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
