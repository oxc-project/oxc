use proc_macro::{TokenStream, TokenTree};
use std::str::FromStr;

enum ItemKind {
    Enum,
    Struct,
    Unknown,
}

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
    let mut input = input.into_iter();
    let mut stream = TokenStream::new();
    let mut output = TokenStream::from_str("#[derive(::oxc_ast_macros::Ast)]").unwrap();

    let mut item_kind = ItemKind::Unknown;

    while let Some(next) = input.next() {
        if let TokenTree::Ident(ident) = &next {
            match ident.to_string().as_str() {
                "enum" => {
                    assert!(matches!(item_kind, ItemKind::Unknown));
                    item_kind = ItemKind::Enum;
                    stream.extend(Some(next));
                    break;
                }
                "struct" => {
                    assert!(matches!(item_kind, ItemKind::Unknown));
                    item_kind = ItemKind::Struct;
                    stream.extend(Some(next));
                    break;
                }
                _ => {}
            }
        }

        stream.extend(Some(next));
    }

    // append the remained of the input tokens to the stream
    stream.extend(input);

    let repr = match item_kind {
        ItemKind::Enum => TokenStream::from_str("#[repr(C, u8)]").unwrap(),
        // ItemKind::Struct => TokenStream::from_str("#[repr(C)]").unwrap(),
        ItemKind::Struct => TokenStream::default(),
        ItemKind::Unknown => unreachable!(),
    };

    output.extend(repr);
    output.extend(stream);
    output
}

/// Dummy derive macro for a non-existent trait `Ast`.
///
/// Does not generate any code.
/// Only purpose is to allow using `#[scope]`, `#[visit]`, and other attrs in the AST node type defs.
#[proc_macro_derive(Ast, attributes(span, scope, visit, visit_as, visit_args, serde, tsify))]
pub fn ast_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
