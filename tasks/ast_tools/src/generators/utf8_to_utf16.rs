//! Generator for visitor to convert spans from UTF-8 offsets to UTF-16 offsets.

use proc_macro2::TokenStream;
use quote::quote;

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
/// 1. Types where a shorthand syntax means 2 nodes have same span e.g. `const {x} = y;`, `export {x}`.
/// 2. `WithClause`, where `IdentifierName` for `with` keyword has span outside of the `WithClause`.
/// 3. `TemplateLiteral`s, where `quasis` and `expressions` are interleaved.
///
/// Define custom visitors for these types, which ensure `convert_offset` is always called with offsets
/// in ascending order.
fn generate(schema: &Schema, codegen: &Codegen) -> TokenStream {
    let estree_derive_id = codegen.get_derive_id_by_name("ESTree");
    let span_type_id = schema.type_names["Span"];
    let skip_type_ids = [
        "ObjectProperty",
        "BindingProperty",
        "ImportSpecifier",
        "ExportSpecifier",
        "WithClause",
        "TemplateLiteral",
    ]
    .map(|type_name| schema.type_names[type_name]);

    let methods = schema.types.iter().filter_map(|type_def| {
        let struct_def = type_def.as_struct()?;

        if !struct_def.generates_derive(estree_derive_id) {
            return None;
        }

        if skip_type_ids.contains(&struct_def.id) {
            return None;
        }

        // Skip `oxc_regular_expression` types. They don't appear in ESTree AST.
        if struct_def.file(schema).krate() == "oxc_regular_expression" {
            return None;
        }

        generate_visitor(struct_def, span_type_id, schema)
    });

    quote! {
        use oxc_span::GetSpan;
        use oxc_syntax::scope::ScopeFlags;
        use oxc_ast::ast::*;

        ///@@line_break
        use crate::{
            utf8_to_utf16::Utf8ToUtf16Converter,
            VisitMut, walk_mut,
        };

        ///@@line_break
        impl<'a> VisitMut<'a> for Utf8ToUtf16Converter<'_> {
            #(#methods)*

            ///@@line_break
            fn visit_object_property(&mut self, it: &mut ObjectProperty<'a>) {
                self.convert_offset(&mut it.span.start);

                // If shorthand, span of `key` and `value` are the same
                match (it.shorthand, &mut it.key, &mut it.value) {
                    (true, PropertyKey::StaticIdentifier(key), Expression::Identifier(value)) => {
                        self.visit_identifier_name(key);
                        value.span = key.span;
                    }
                    (_, key, value) => {
                        self.visit_property_key(key);
                        self.visit_expression(value);
                    }
                }

                self.convert_offset(&mut it.span.end);
            }

            ///@@line_break
            fn visit_binding_property(&mut self, it: &mut BindingProperty<'a>) {
                self.convert_offset(&mut it.span.start);

                // If shorthand, span of `key` and `value` are the same
                match (it.shorthand, &mut it.key, &mut it.value) {
                    (
                        true,
                        PropertyKey::StaticIdentifier(key),
                        BindingPattern { kind: BindingPatternKind::BindingIdentifier(value), .. },
                    ) => {
                        self.visit_identifier_name(key);
                        value.span = key.span;
                    }
                    (
                        true,
                        PropertyKey::StaticIdentifier(key),
                        BindingPattern { kind: BindingPatternKind::AssignmentPattern(pattern), .. },
                    ) => {
                        self.visit_assignment_pattern(pattern);
                        key.span = pattern.left.span();
                    }
                    (_, key, value) => {
                        self.visit_property_key(key);
                        self.visit_binding_pattern(value);
                    }
                }

                self.convert_offset(&mut it.span.end);
            }

            ///@@line_break
            fn visit_export_specifier(&mut self, it: &mut ExportSpecifier<'a>) {
                self.convert_offset(&mut it.span.start);

                // `local` and `exported` have same span if e.g.:
                // * `export {x}`
                // * `export {x} from 'foo.js;`
                // * `export {"a-b"} from 'foo.js';`
                match (&mut it.local, &mut it.exported) {
                    (
                        ModuleExportName::IdentifierReference(local),
                        ModuleExportName::IdentifierName(exported),
                    ) if local.span == exported.span => {
                        self.visit_identifier_reference(local);
                        exported.span = local.span;
                    }
                    (
                        ModuleExportName::IdentifierName(local),
                        ModuleExportName::IdentifierName(exported),
                    ) if local.span == exported.span => {
                        self.visit_identifier_name(local);
                        exported.span = local.span;
                    }
                    (
                        ModuleExportName::StringLiteral(local),
                        ModuleExportName::StringLiteral(exported),
                    ) if local.span == exported.span => {
                        self.visit_string_literal(local);
                        exported.span = local.span;
                    }
                    (local, exported) => {
                        self.visit_module_export_name(local);
                        self.visit_module_export_name(exported);
                    }
                }

                self.convert_offset(&mut it.span.end);
            }

            ///@@line_break
            fn visit_import_specifier(&mut self, it: &mut ImportSpecifier<'a>) {
                self.convert_offset(&mut it.span.start);

                // `imported` and `local` have same span if e.g. `import {x} from 'foo';`
                match &mut it.imported {
                    ModuleExportName::IdentifierName(imported) if imported.span == it.local.span => {
                        self.visit_identifier_name(imported);
                        it.local.span = imported.span;
                    }
                    imported => {
                        self.visit_module_export_name(imported);
                        self.visit_binding_identifier(&mut it.local);
                    }
                }

                self.convert_offset(&mut it.span.end);
            }

            ///@@line_break
            fn visit_with_clause(&mut self, it: &mut WithClause<'a>) {
                // `WithClause::attributes_keyword` has a span before start of the `WithClause`.
                // ESTree does not include that node, nor the span of the `WithClause` itself,
                // so skip processing those spans.
                self.visit_import_attributes(&mut it.with_entries);
            }

            ///@@line_break
            fn visit_template_literal(&mut self, it: &mut TemplateLiteral<'a>) {
                self.convert_offset(&mut it.span.start);

                // Visit `quasis` and `expressions` in source order. The two `Vec`s are interleaved.
                for (quasi, expression) in it.quasis.iter_mut().zip(&mut it.expressions) {
                    self.visit_template_element(quasi);
                    self.visit_expression(expression);
                }
                self.visit_template_element(it.quasis.last_mut().unwrap());

                self.convert_offset(&mut it.span.end);
            }
        }
    }
}

/// Generate visitor method.
fn generate_visitor(
    struct_def: &StructDef,
    span_type_id: TypeId,
    schema: &Schema,
) -> Option<TokenStream> {
    // Find `Span` field.
    // Panic if `Span` appears in any other field which is included in ESTree AST.
    // We could handle that case, but it's a bit complicated, so not implementing that until we need it.
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

    if !has_span_field {
        return None;
    }

    // Generate visitor method
    let ty = struct_def.ty(schema);

    let visitor_names = struct_def.visit.visitor_names.as_ref().unwrap();
    let visit_method_ident = visitor_names.visitor_ident();
    let walk_fn_ident = visitor_names.walk_ident();

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

    let visitor = quote! {
        ///@@line_break
        fn #visit_method_ident(&mut self, it: &mut #ty #extra_params) {
            self.convert_offset(&mut it.span.start);
            walk_mut::#walk_fn_ident(self, it #extra_args);
            self.convert_offset(&mut it.span.end);
        }
    };

    Some(visitor)
}
