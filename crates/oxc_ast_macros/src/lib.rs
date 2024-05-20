use proc_macro::TokenStream;

/// Attach to AST node type (struct or enum), to signal to codegen to create visitor for this type.
/// Macro itself does nothing - just passes through the token stream unchanged.
#[proc_macro_attribute]
pub fn visited_node(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
