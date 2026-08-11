//! Generator for random AST construction implementations.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Path, parse_str};

use crate::{
    AST_GENERATOR_CRATE_PATH, Codegen, Generator, Result,
    output::Output,
    parse::attr::{AttrLocation, AttrPart, AttrPositions, attr_positions},
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, TypeId, VariantDef, Visibility},
};

use super::define_generator;

/// Generates `Generate` implementations for AST schema types.
pub struct AstGeneratorGenerator;

define_generator!(AstGeneratorGenerator);

impl Generator for AstGeneratorGenerator {
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("ast_gen", attr_positions!(Struct | Enum | StructField | EnumVariant))]
    }

    fn parse_attr(&self, _name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        match part {
            AttrPart::String("with", value) => match location {
                AttrLocation::Struct(def) => def.ast_gen.with = Some(value),
                AttrLocation::Enum(def) => def.ast_gen.with = Some(value),
                AttrLocation::StructField(def, index) => {
                    def.fields[index].ast_gen.with = Some(value);
                }
                AttrLocation::EnumVariant(def, index) => {
                    def.variants[index].ast_gen.with = Some(value);
                }
                _ => return Err(()),
            },
            AttrPart::String("weight", value) => {
                let Ok(weight) = value.parse::<u32>() else { return Err(()) };
                if weight == 0 {
                    return Err(());
                }
                let AttrLocation::EnumVariant(def, index) = location else { return Err(()) };
                def.variants[index].ast_gen.weight = Some(weight);
            }
            _ => return Err(()),
        }
        Ok(())
    }

    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let impls = schema
            .structs_and_enums()
            .filter_map(|def| match def {
                crate::schema::StructOrEnum::Struct(def) => generate_struct(def, schema),
                crate::schema::StructOrEnum::Enum(def) => generate_enum(def, schema),
            })
            .collect::<TokenStream>();

        let output = quote! {
            //! Generated random AST implementations.

            ///@@line_break
            use rand::Rng;

            ///@@line_break
            use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
            use oxc_ast::ast::*;
            use oxc_str::{Ident, Str};

            ///@@line_break
            use crate::AstGenerator;

            /// Generate an arena-backed AST value.
            pub trait Generate<'a>: Sized {
                /// Generate a value.
                fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self;
            }

            #impls
        };

        Output::Rust {
            path: format!("{AST_GENERATOR_CRATE_PATH}/src/generated.rs"),
            tokens: output,
        }
    }
}

fn generate_struct(def: &StructDef, schema: &Schema) -> Option<TokenStream> {
    if def.file(schema).krate() != "oxc_ast"
        || def.is_foreign
        || def.visibility != Visibility::Public
        || !is_supported_crate(def.file(schema).krate())
    {
        return None;
    }
    let ty = qualified_struct_type(def, schema)?;
    if matches!(def.name(), "Span" | "SourceType") {
        return None;
    }
    let body = if is_unsupported(def.file(schema)) {
        unsupported_body(def.name())
    } else if let Some(path) = &def.ast_gen.with {
        let path: Path = parse_str(path).unwrap();
        quote!( #path(generator) )
    } else if def.file(schema).krate() == "oxc_ast" && def.visit.has_visitor() {
        generate_struct_with_builder(def, schema)
    } else {
        generate_struct_literal(def, schema)
    };
    let body = if def_is_typescript_only(def.name(), def.file(schema)) {
        typescript_only_body(def.name(), &body)
    } else {
        body
    };

    Some(quote! {
        ///@@line_break
        impl<'a> Generate<'a> for #ty {
            fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
                let _ = &generator;
                #body
            }
        }
    })
}

fn generate_struct_with_builder(def: &StructDef, schema: &Schema) -> TokenStream {
    let ty = qualified_struct_path(def, schema).unwrap();
    let args = def
        .fields
        .iter()
        .filter(|field| !is_builder_default(field, schema))
        .map(|field| generate_field(def, field, schema));
    quote!( #ty::new(#(#args),*, generator.ast()) )
}

fn generate_struct_literal(def: &StructDef, schema: &Schema) -> TokenStream {
    let ty = qualified_struct_path(def, schema).unwrap();
    let fields = def.fields.iter().map(|field| {
        let ident = field.ident();
        let value = generate_field(def, field, schema);
        quote!( #ident: #value )
    });
    quote!( #ty { #(#fields),* } )
}

fn generate_field(parent: &StructDef, field: &FieldDef, schema: &Schema) -> TokenStream {
    if let Some(path) = &field.ast_gen.with {
        let path: Path = parse_str(path).unwrap();
        return quote!( #path(generator) );
    }
    if parent.name() == "Program" {
        let ty = field.type_def(schema).ty(schema);
        return match field.name() {
            "span" => quote!(oxc_span::SPAN),
            "source_type" => quote!(generator.source_type()),
            "source_text" => quote!(""),
            "comments" | "directives" => {
                quote!(oxc_allocator::Vec::new_in(generator.ast()))
            }
            _ => quote!(generator.generate::<#ty>()),
        };
    }
    if field.name() == "span" {
        return quote!(oxc_span::SPAN);
    }
    if field.type_def(schema).innermost_type(schema).name() == "Span" {
        return quote!(oxc_span::SPAN);
    }
    if field.name() == "raw" && matches!(field.type_def(schema), TypeDef::Option(_)) {
        return quote!(None);
    }
    if !def_is_typescript_only(parent.name(), parent.file(schema))
        && (field.estree.is_ts || type_is_typescript_only(field.type_def(schema), schema))
    {
        let ty = field.type_def(schema).ty(schema);
        let fallback = generate_unsupported_field(field, schema);
        return quote! {
            if generator.is_typescript() {
                generator.generate::<#ty>()
            } else {
                #fallback
            }
        };
    }
    if type_is_unsupported(field.type_def(schema), schema) {
        return generate_unsupported_field(field, schema);
    }
    if field.type_def(schema).innermost_type(schema).name() == "RegExpFlags" {
        return quote!(RegExpFlags::empty());
    }
    if field.type_def(schema).innermost_type(schema).name() == "CommentNewlines" {
        return quote!(CommentNewlines::empty());
    }
    let ty = field.type_def(schema).ty(schema);
    quote!(generator.generate::<#ty>())
}

fn generate_enum(def: &EnumDef, schema: &Schema) -> Option<TokenStream> {
    if !matches!(def.file(schema).krate(), "oxc_ast" | "oxc_syntax")
        || def.is_foreign
        || def.visibility != Visibility::Public
        || !is_supported_crate(def.file(schema).krate())
    {
        return None;
    }
    if def.file(schema).krate() == "oxc_syntax"
        && !matches!(def.file(schema).import_path(), "::class" | "::number" | "::operator")
    {
        return None;
    }
    let ty = qualified_enum_type(def, schema)?;
    if matches!(def.name(), "SourceType" | "Language" | "LanguageVariant" | "ModuleKind") {
        return None;
    }
    let body = if is_unsupported(def.file(schema)) {
        unsupported_body(def.name())
    } else if let Some(path) = &def.ast_gen.with {
        let path: Path = parse_str(path).unwrap();
        quote!( #path(generator) )
    } else {
        generate_enum_body(def, schema)
    };
    let body = if def_is_typescript_only(def.name(), def.file(schema)) {
        typescript_only_body(def.name(), &body)
    } else {
        body
    };

    Some(quote! {
        ///@@line_break
        impl<'a> Generate<'a> for #ty {
            fn generate<R: Rng + ?Sized>(generator: &mut AstGenerator<'a, '_, R>) -> Self {
                let _ = &generator;
                #body
            }
        }
    })
}

fn generate_enum_body(def: &EnumDef, schema: &Schema) -> TokenStream {
    let typescript_choices = enum_choices(def, schema, true);
    assert!(!typescript_choices.is_empty(), "{} has no supported variants", def.name());

    if def_is_typescript_only(def.name(), def.file(schema)) {
        return generate_choices_body(&typescript_choices, schema);
    }

    let javascript_choices = enum_choices(def, schema, false);
    assert!(!javascript_choices.is_empty(), "{} has no JavaScript variants", def.name());
    if choices_equal(&javascript_choices, &typescript_choices) {
        return generate_choices_body(&javascript_choices, schema);
    }

    let javascript_body = generate_choices_body(&javascript_choices, schema);
    let typescript_body = generate_choices_body(&typescript_choices, schema);
    quote! {
        if generator.is_typescript() {
            #typescript_body
        } else {
            #javascript_body
        }
    }
}

fn generate_choices_body(choices: &[EnumChoice<'_>], schema: &Schema) -> TokenStream {
    let minimum = choices.iter().map(|choice| choice_min_cost(*choice, schema)).min().unwrap();
    let terminal_choices = choices
        .iter()
        .copied()
        .filter(|choice| choice_min_cost(*choice, schema) == minimum)
        .collect::<Vec<_>>();

    if terminal_choices.len() == choices.len() {
        let weights = choices.iter().map(|choice| choice_weight(*choice));
        let arms = choices.iter().enumerate().map(|(index, choice)| {
            let index = u32::try_from(index).unwrap();
            let value = generate_choice(*choice, schema);
            quote!( #index => #value )
        });
        return quote! {
            match generator.random_weighted(&[#(#weights),*]) {
                #(#arms,)*
                _ => unreachable!(),
            }
        };
    }

    let all_weights = choices.iter().map(|choice| choice_weight(*choice));
    let all_arms = choices.iter().enumerate().map(|(index, choice)| {
        let index = u32::try_from(index).unwrap();
        let value = generate_choice(*choice, schema);
        quote!( #index => #value )
    });
    let terminal_weights = terminal_choices.iter().map(|choice| choice_weight(*choice));
    let terminal_arms = terminal_choices.iter().enumerate().map(|(index, choice)| {
        let index = u32::try_from(index).unwrap();
        let value = generate_choice(*choice, schema);
        quote!( #index => #value )
    });

    quote! {
        if generator.at_limit() {
            match generator.random_weighted(&[#(#terminal_weights),*]) {
                #(#terminal_arms,)*
                _ => unreachable!(),
            }
        } else {
            match generator.random_weighted(&[#(#all_weights),*]) {
                #(#all_arms,)*
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Clone, Copy)]
enum EnumChoice<'s> {
    Variant(&'s VariantDef),
    Inherited(&'s EnumDef),
}

fn enum_choices<'s>(
    def: &'s EnumDef,
    schema: &'s Schema,
    include_typescript: bool,
) -> Vec<EnumChoice<'s>> {
    let mut choices = Vec::new();
    collect_enum_choices(def, schema, include_typescript, &mut choices);
    choices
}

fn collect_enum_choices<'s>(
    def: &'s EnumDef,
    schema: &'s Schema,
    include_typescript: bool,
    choices: &mut Vec<EnumChoice<'s>>,
) {
    choices.extend(
        def.variants
            .iter()
            .filter(|variant| {
                variant.field_type(schema).is_none_or(|ty| {
                    !type_is_unsupported(ty, schema)
                        && (include_typescript || !type_is_typescript_only(ty, schema))
                })
            })
            .map(EnumChoice::Variant),
    );

    for inherited in def.inherits_enums(schema) {
        if is_unsupported(inherited.file(schema))
            || (!include_typescript
                && def_is_typescript_only(inherited.name(), inherited.file(schema)))
        {
            continue;
        }
        if inherited.ast_gen.with.is_some() {
            choices.push(EnumChoice::Inherited(inherited));
        } else {
            collect_enum_choices(inherited, schema, include_typescript, choices);
        }
    }
}

fn choices_equal(left: &[EnumChoice<'_>], right: &[EnumChoice<'_>]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| match (*left, *right) {
            (EnumChoice::Variant(left), EnumChoice::Variant(right)) => std::ptr::eq(left, right),
            (EnumChoice::Inherited(left), EnumChoice::Inherited(right)) => {
                std::ptr::eq(left, right)
            }
            _ => false,
        })
}

fn choice_weight(choice: EnumChoice<'_>) -> u32 {
    match choice {
        EnumChoice::Variant(variant) => variant.ast_gen.weight.unwrap_or(1),
        EnumChoice::Inherited(_) => 1,
    }
}

fn choice_min_cost(choice: EnumChoice<'_>, schema: &Schema) -> u32 {
    match choice {
        EnumChoice::Variant(variant) => variant_min_cost(variant, schema),
        EnumChoice::Inherited(def) => min_cost(def.id(), schema, &mut vec![]),
    }
}

fn generate_choice(choice: EnumChoice<'_>, schema: &Schema) -> TokenStream {
    match choice {
        EnumChoice::Variant(variant) => generate_variant(variant),
        EnumChoice::Inherited(def) => {
            let ty = def.ty(schema);
            quote!( generator.generate::<#ty>().into() )
        }
    }
}

fn generate_variant(variant: &VariantDef) -> TokenStream {
    if let Some(path) = &variant.ast_gen.with {
        let path: Path = parse_str(path).unwrap();
        return quote!( #path(generator) );
    }
    let ident = variant.ident();
    if variant.is_fieldless() {
        quote!( Self::#ident )
    } else {
        quote!( Self::#ident(generator.generate()) )
    }
}

fn variant_min_cost(variant: &VariantDef, schema: &Schema) -> u32 {
    variant.field_type_id.map_or(0, |id| min_cost(id, schema, &mut vec![]))
}

fn min_cost(type_id: TypeId, schema: &Schema, visiting: &mut Vec<TypeId>) -> u32 {
    if visiting.contains(&type_id) {
        return u32::MAX;
    }
    visiting.push(type_id);
    let cost = match &schema.types[type_id] {
        TypeDef::Struct(def) => {
            if is_unsupported(def.file(schema)) {
                u32::MAX
            } else {
                def.fields
                    .iter()
                    .filter(|field| !is_builder_default(field, schema))
                    .try_fold(u32::from(def.kind.has_kind), |total, field| {
                        if type_is_unsupported(field.type_def(schema), schema) {
                            Some(total)
                        } else {
                            total.checked_add(min_cost(field.type_id, schema, visiting))
                        }
                    })
                    .unwrap_or(u32::MAX)
            }
        }
        TypeDef::Enum(def) => enum_choices(def, schema, true)
            .into_iter()
            .map(|choice| match choice {
                EnumChoice::Variant(variant) => {
                    variant.field_type_id.map_or(0, |id| min_cost(id, schema, visiting))
                }
                EnumChoice::Inherited(inherited) => min_cost(inherited.id(), schema, visiting),
            })
            .min()
            .unwrap_or(0),
        TypeDef::Option(_) | TypeDef::Vec(_) | TypeDef::Cell(_) | TypeDef::Primitive(_) => 0,
        TypeDef::Box(def) => min_cost(def.inner_type_id, schema, visiting),
        TypeDef::Pointer(_) => u32::MAX,
    };
    visiting.pop();
    cost
}

fn is_builder_default(field: &FieldDef, schema: &Schema) -> bool {
    if field.name() == "node_id" || field.builder.is_default {
        return true;
    }
    match field.type_def(schema).innermost_type(schema) {
        TypeDef::Struct(def) => def.builder.is_default,
        TypeDef::Enum(def) => def.builder.is_default,
        _ => false,
    }
}

fn type_is_unsupported(ty: &TypeDef, schema: &Schema) -> bool {
    match ty.innermost_type(schema) {
        TypeDef::Struct(def) => {
            def.file(schema).krate() != "oxc_ast" || is_unsupported(def.file(schema))
        }
        TypeDef::Enum(def) => is_unsupported(def.file(schema)),
        _ => false,
    }
}

fn type_is_typescript_only(ty: &TypeDef, schema: &Schema) -> bool {
    match ty.innermost_type(schema) {
        TypeDef::Struct(def) => def_is_typescript_only(def.name(), def.file(schema)),
        TypeDef::Enum(def) => def_is_typescript_only(def.name(), def.file(schema)),
        _ => false,
    }
}

fn def_is_typescript_only(name: &str, file: &crate::schema::File) -> bool {
    file.krate() == "oxc_ast"
        && file.import_path() == "::ast::ts"
        && !matches!(name, "Decorator" | "ImportOrExportKind")
}

fn generate_unsupported_field(field: &FieldDef, schema: &Schema) -> TokenStream {
    let ty = field.type_def(schema);
    match ty {
        TypeDef::Option(_) => quote!(None),
        TypeDef::Vec(def) => {
            let inner = def.inner_type(schema).ty(schema);
            quote!(ArenaVec::<#inner>::new_in(generator.ast()))
        }
        TypeDef::Primitive(def) if def.name() == "bool" => quote!(false),
        TypeDef::Enum(def) if def.name() == "ImportOrExportKind" => {
            quote!(ImportOrExportKind::Value)
        }
        _ => {
            let ty = ty.ty(schema);
            quote!(<#ty as Default>::default())
        }
    }
}

fn is_unsupported(file: &crate::schema::File) -> bool {
    file.krate() == "oxc_ast" && file.import_path() == "::ast::jsx"
}

fn is_supported_crate(krate: &str) -> bool {
    matches!(krate, "oxc_ast" | "oxc_syntax")
}

fn unsupported_body(name: &str) -> TokenStream {
    let message = format!("{name} generation is not implemented for JSX or TSX");
    quote!(panic!(#message))
}

fn typescript_only_body(name: &str, body: &TokenStream) -> TokenStream {
    let message = format!("{name} generation requires a TypeScript source type");
    quote! {
        if generator.is_typescript() {
            #body
        } else {
            panic!(#message)
        }
    }
}

fn qualified_struct_type(def: &StructDef, schema: &Schema) -> Option<TokenStream> {
    qualified_type(def.name(), def.file(schema), def.has_lifetime(schema))
}

fn qualified_struct_path(def: &StructDef, schema: &Schema) -> Option<TokenStream> {
    qualified_type(def.name(), def.file(schema), false)
}

fn qualified_enum_type(def: &EnumDef, schema: &Schema) -> Option<TokenStream> {
    qualified_type(def.name(), def.file(schema), def.has_lifetime(schema))
}

fn qualified_type(
    name: &str,
    file: &crate::schema::File,
    has_lifetime: bool,
) -> Option<TokenStream> {
    let ident = format_ident!("{name}");
    let lifetime = if has_lifetime { quote!(<'a>) } else { quote!() };
    let crate_ident = format_ident!("{}", file.krate());

    if file.krate() == "oxc_ast" && file.import_path().starts_with("::ast") {
        return Some(quote!(oxc_ast::ast::#ident #lifetime));
    }
    if file.import_path().is_empty() {
        Some(quote!(#crate_ident::#ident #lifetime))
    } else {
        let import_path: Path = parse_str(file.import_path()).ok()?;
        Some(quote!(#crate_ident #import_path :: #ident #lifetime))
    }
}
