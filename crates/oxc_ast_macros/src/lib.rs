use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Item, parse_macro_input};

mod ast;
mod generated {
    pub mod derived_traits;
    pub mod structs;
}

/// This attribute serves two purposes:
///
/// Firstly, it is a marker for `oxc_ast_tools`, to understand that a type is part of the AST.
///
/// Secondly, it adds the following code around the type:
///
/// ### `#[repr(C)]`
///
/// `#[repr]` attribute is added to the type, to make the memory layout of the type predictable:
///
/// * Structs: `#[repr(C)]`. e.g. `#[repr(C)] struct S { x: X, y: Y }`
/// * Fieldless enums: `#[repr(u8)]` e.g. `#[repr(u8)] enum E { X, Y, Z, }`.
/// * Fieldful enums: `#[repr(C, u8)]` e.g. `#[repr(C, u8)] enum E { X(X), Y(Y) }`
///
/// ### Derive `Ast` trait
///
/// `#[derive(oxc_ast_macros::Ast)]` is added to the type.
///
/// `Ast` derive macro is a no-op (see below) but allows custom attributes on AST types.
///
/// See `derives` and `generators` directories in `tasks/ast_tools` for details of the various
/// custom attributes, and how they're used.
///
/// ### Trait assertions
///
/// `oxc_ast_tools` generates code for trait impls where those traits are specified with
/// `#[generate_derive(SomeTrait)]`. This is similar to `#[derive(SomeTrait)]`, but the code is
/// generated ahead-of-time, rather than in a proc macro.
///
/// Add assertions that traits used in `#[generate_derive(...)]` are in scope.
#[proc_macro_attribute]
pub fn ast(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as Item);
    let expanded = ast::ast(&mut input, TokenStream2::from(args));
    TokenStream::from(expanded)
}

/// Dummy attribute macro `#[ast_meta]`.
///
/// This macro passes the input through unchanged, except that it applies
/// `#[derive(::oxc_ast_macros::Ast)]` to the type, to allow the use of helper attributes.
///
/// This attribute should be used on types which are not part of the AST, but are used by `oxc_ast_tools`
/// in processing the AST in some way. The purpose of this attribute is to pass data to `oxc_ast_tools`.
#[proc_macro_attribute]
pub fn ast_meta(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = TokenStream::from(quote!( #[derive(::oxc_ast_macros::Ast)] ));
    output.extend(input);
    output
}

/// Dummy derive macro for a non-existent trait `Ast`.
///
/// Does not generate any code.
/// Its only purpose is to allow the occurrence of helper attributes used in `tasks/ast_tools`.
///
/// See [`macro@ast`] for further details.
#[proc_macro_derive(
    Ast,
    attributes(
        builder,
        clone_in,
        content_eq,
        estree,
        generate_derive,
        js_only,
        plural,
        scope,
        span,
        ts,
        visit
    )
)]
pub fn ast_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
