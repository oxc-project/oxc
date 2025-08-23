//! Generator for visitor to convert spans from UTF-8 offsets to UTF-16 offsets.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_VISIT_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, Schema, StructDef, TypeId},
    utils::create_ident,
};

use super::define_generator;

/// Generator for visitor to convert spans from UTF-8 offsets to UTF-16 offsets.
pub struct Utf8ToUtf16ConverterGenerator;

define_generator!(Utf8ToUtf16ConverterGenerator);

impl Generator for Utf8ToUtf16ConverterGenerator {
    fn generate(&self, schema: &Schema, codegen: &Codegen) -> Output {
        let output = generate(schema, codegen);
        Output::Rust {
            path: output_path(AST_VISIT_CRATE_PATH, "utf8_to_utf16_converter.rs"),
            tokens: output,
        }
    }
}

/// Generate `VisitMut` impl for `Utf8ToUtf16Converter`.
///
/// For each AST node, update `span.start` first, then visit child nodes, then update `span.end`.
/// This ensures offsets are updated in ascending order
/// (assuming AST has not been modified since it was parsed, so nodes are in original order).
///
/// The only exceptions are:
///
/// * Types where a shorthand syntax means 2 nodes have same span e.g. `const {x} = y;`, `export {x}`.
/// * `TemplateLiteral`s and `TSTemplateLiteralType`s, where `quasis` and `expressions` are interleaved.
/// * Decorators before `export` in `@dec export class C {}` / `@dec export default class {}`
///   have span before the start of `ExportNamedDeclaration` / `ExportDefaultDeclaration` span.
/// * `BindingPattern` where `type_annotation` has span within `BindingPatternKind`.
///   Except for `BindingRestElement`, where `type_annotation`'s span is after `BindingPatternKind`.
/// * `FormalParameters` where span can include a `TSThisParameter` which is visited before it.
///
/// Delegate to the custom visitors for these types in `oxc_ast_visit/src/utf8_to_utf16/visit.rs`,
/// which ensure `convert_offset` is always called with offsets in ascending order.
fn generate(schema: &Schema, codegen: &Codegen) -> TokenStream {
    let estree_derive_id = codegen.get_derive_id_by_name("ESTree");
    let span_type_id = schema.type_names["Span"];
    let comment_type_id = schema.type_names["Comment"];

    // Types with custom visitors (see comment above)
    let custom_visitor_type_ids = [
        "FormalParameters",
        "ObjectProperty",
        "BindingPattern",
        "BindingRestElement",
        "BindingProperty",
        "ExportNamedDeclaration",
        "ExportDefaultDeclaration",
        "ExportSpecifier",
        "ImportSpecifier",
        "TemplateLiteral",
        "TSTemplateLiteralType",
    ]
    .map(|type_name| schema.type_names[type_name]);

    let methods = schema.types.iter().filter_map(|type_def| {
        let struct_def = type_def.as_struct()?;

        // Skip `Comment` because we handle adjusting comment spans separately
        if struct_def.id == comment_type_id {
            return None;
        }

        if !struct_def.generates_derive(estree_derive_id) {
            return None;
        }

        // Skip types in `oxc_syntax` and `napi/parser` crates. They don't appear in ESTree AST.
        if matches!(struct_def.file(schema).krate(), "oxc_syntax" | "napi/parser") {
            return None;
        }

        let has_custom_visitor = custom_visitor_type_ids.contains(&struct_def.id);

        generate_visitor(struct_def, has_custom_visitor, span_type_id, schema)
    });

    quote! {
        use oxc_ast::ast::*;
        use oxc_syntax::scope::ScopeFlags;

        ///@@line_break
        use crate::{
            utf8_to_utf16::Utf8ToUtf16Converter,
            VisitMut, walk_mut,
        };

        ///@@line_break
        impl<'a> VisitMut<'a> for Utf8ToUtf16Converter<'_> {
            #(#methods)*
        }
    }
}

/// Generate visitor method.
fn generate_visitor(
    struct_def: &StructDef,
    has_custom_visitor: bool,
    span_type_id: TypeId,
    schema: &Schema,
) -> Option<TokenStream> {
    // Skip types which don't have a `span: Span` field (unless they have a custom visitor)
    if !has_custom_visitor && !has_span_field(struct_def, span_type_id, schema) {
        return None;
    }

    // Generate visit method
    let ty = struct_def.ty(schema);

    let visitor_names = struct_def.visit.visitor_names.as_ref().unwrap();
    let visit_method_ident = visitor_names.visitor_ident();

    let (extra_params, extra_args): (TokenStream, TokenStream) = struct_def
        .visit
        .visit_args
        .iter()
        .map(|(arg_name, arg_type_name)| {
            let param_ident = create_ident(arg_name);
            let arg_type_ident = create_ident(arg_type_name);
            (quote!( , #param_ident: #arg_type_ident ), quote!( , #param_ident ))
        })
        .unzip();

    let body = if has_custom_visitor {
        let convert_method_ident = format_ident!("convert_{}", struct_def.snake_name());
        quote! {
            ///@ Custom implementation
            self.#convert_method_ident(it #extra_args);
        }
    } else {
        let walk_fn_ident = visitor_names.walk_ident();
        quote! {
            self.convert_offset(&mut it.span.start);
            walk_mut::#walk_fn_ident(self, it #extra_args);
            self.convert_offset(&mut it.span.end);
        }
    };

    let visitor = quote! {
        ///@@line_break
        fn #visit_method_ident(&mut self, it: &mut #ty #extra_params) {
            #body
        }
    };

    Some(visitor)
}

/// Check if struct has a `span: Span` field.
///
/// Panic if `Span` appears in any other field which is included in ESTree AST.
/// We could handle that case, but it's a bit complicated, so not implementing that until we need it.
fn has_span_field(struct_def: &StructDef, span_type_id: TypeId, schema: &Schema) -> bool {
    let mut has_span_field = false;
    for field in &struct_def.fields {
        if field.type_id == span_type_id && field.name() == "span" {
            has_span_field = true;
        } else {
            assert!(
                field.estree.skip
                    || field.type_def(schema).innermost_type(schema).id() != span_type_id,
                "Cannot handle `Span` field: `{}::{}` in `Utf8ToUtf16Converter` generator",
                struct_def.name(),
                field.name(),
            );
        }
    }
    has_span_field
}
