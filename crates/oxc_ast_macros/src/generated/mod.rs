// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/main.rs`

use proc_macro2::TokenStream;
use quote::quote;

pub fn get_trait_crate_and_generics(trait_name: &str) -> (TokenStream, TokenStream) {
    match trait_name {
        "CloneIn" => (quote!(::oxc_allocator::CloneIn), quote!(< 'static >)),
        "GetAddress" => (quote!(::oxc_allocator::GetAddress), TokenStream::new()),
        "GetSpan" => (quote!(::oxc_span::GetSpan), TokenStream::new()),
        "GetSpanMut" => (quote!(::oxc_span::GetSpanMut), TokenStream::new()),
        "ContentEq" => (quote!(::oxc_span::ContentEq), TokenStream::new()),
        "ESTree" => (quote!(::oxc_estree::ESTree), TokenStream::new()),
        _ => panic!("Invalid derive trait(generate_derive): {trait_name}"),
    }
}
