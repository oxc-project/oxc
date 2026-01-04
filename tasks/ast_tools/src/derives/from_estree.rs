//! Derive for `FromESTree` impls, which deserialize ESTree JSON into oxc AST.

use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, VariantDef};

use super::estree::{
    get_fieldless_variant_value, get_struct_field_name, should_skip_enum_variant, should_skip_field,
};
use super::{Derive, StructOrEnum, define_derive};

/// Derive for `FromESTree` impls, which deserialize ESTree JSON into oxc AST.
pub struct DeriveFromESTree;

define_derive!(DeriveFromESTree);

impl Derive for DeriveFromESTree {
    fn trait_name(&self) -> &'static str {
        "FromESTree"
    }

    fn trait_has_lifetime(&self) -> bool {
        true
    }

    fn crate_name(&self) -> &'static str {
        "oxc_ast"
    }

    fn snake_name(&self) -> String {
        "from_estree".to_string()
    }

    // We don't declare any attrs here because we reuse the ESTree schema extension
    // data which is already populated by DeriveESTree's attribute parsing.
    // The `#[estree]` attrs are owned by DeriveESTree.

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(
                unused_imports,
                clippy::match_same_arms,
                clippy::semicolon_if_nothing_returned,
                clippy::too_many_lines
            )]

            ///@@line_break
            use oxc_allocator::{Allocator, Box as ABox, Vec as AVec};
            use crate::deserialize::{
                DeserError, DeserResult, ESTreeField, ESTreeType, FromESTree,
                FromESTreeConverter, parse_span, parse_span_or_empty, record_unknown_span,
            };
        }
    }

    /// Generate implementation of `FromESTree` for a struct or enum.
    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => {
                if struct_def.estree.skip {
                    return quote!();
                }
                generate_impl_for_struct(struct_def, schema)
            }
            StructOrEnum::Enum(enum_def) => {
                if enum_def.estree.skip {
                    return quote!();
                }
                generate_impl_for_enum(enum_def, schema)
            }
        }
    }
}

/// Generate `FromESTree` implementation for a struct.
fn generate_impl_for_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    // Use concrete lifetime 'a (not anonymous '_) since we return data with that lifetime
    let ty = struct_def.ty_with_lifetime(schema, false);

    // Check if this is a tuple struct (field name is numeric)
    let is_tuple_struct =
        struct_def.fields.first().is_some_and(|f| f.name().chars().all(|c| c.is_ascii_digit()));

    if is_tuple_struct {
        // For tuple structs, generate using a sensible default
        // This handles cases like `struct RegExpFlagsAlias(#[estree(skip)] u8)`
        let struct_name = struct_def.name();
        let default_value = match struct_name {
            // RegExpFlags is a bitflags struct - use empty()
            "RegExpFlags" => quote!(Self::empty()),
            // Other tuple structs use Default
            _ => quote!(Default::default()),
        };
        return quote! {
            impl<'a> FromESTree<'a> for #ty {
                fn from_estree(_json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
                    Ok(#default_value)
                }
            }
        };
    }

    let (body, uses_allocator) = generate_body_for_struct(struct_def, schema);

    // Use underscore prefix for allocator if it's not used
    let allocator_param = if uses_allocator { quote!(allocator) } else { quote!(_allocator) };

    quote! {
        impl<'a> FromESTree<'a> for #ty {
            fn from_estree(json: &serde_json::Value, #allocator_param: &'a Allocator) -> DeserResult<Self> {
                #body
            }
        }
    }
}

/// Generate body of `from_estree` method for a struct.
/// Returns (body TokenStream, uses_allocator bool).
fn generate_body_for_struct(struct_def: &StructDef, schema: &Schema) -> (TokenStream, bool) {
    // Generate field deserializations and track if allocator is used
    let field_results: Vec<(TokenStream, bool)> = struct_def
        .fields
        .iter()
        .map(|field| generate_field_deserialization(field, struct_def, schema))
        .collect();

    let field_deserializations: Vec<_> = field_results.iter().map(|(ts, _)| ts.clone()).collect();
    let uses_allocator = field_results.iter().any(|(_, uses)| *uses);

    // Generate struct construction
    let field_assignments = struct_def
        .fields
        .iter()
        .map(|field| {
            let field_ident = field.ident();
            quote! { #field_ident }
        })
        .collect::<Vec<_>>();

    let struct_ident = struct_def.ident();

    let body = quote! {
        #(#field_deserializations)*
        Ok(#struct_ident {
            #(#field_assignments,)*
        })
    };

    (body, uses_allocator)
}

/// Generate deserialization for a single struct field.
/// Returns (TokenStream, uses_allocator bool).
fn generate_field_deserialization(
    field: &FieldDef,
    struct_def: &StructDef,
    schema: &Schema,
) -> (TokenStream, bool) {
    let field_ident = field.ident();
    let field_name = field.name();

    // Special handling for span field
    if field_name == "span" {
        return (
            quote! {
                let span = parse_span_or_empty(json);
            },
            false, // span parsing doesn't use allocator
        );
    }

    // Check if this field is the source of an `append_to` (i.e., it's appended to another field).
    // If so, its deserialization is handled by that other field, so we generate nothing here.
    if is_appended_to_another_field(field, struct_def) {
        return (quote! {}, false);
    }

    // Skip fields marked with #[estree(skip)]
    if should_skip_field(field, schema) {
        return generate_default_field(field, schema);
    }

    // Get the ESTree field name (might be renamed)
    let estree_name = get_struct_field_name(field);
    let estree_name_str = estree_name.as_ref();

    // Get field type
    let field_type = field.type_def(schema);

    // Check if field has a `via` converter
    if let Some(converter_name) = &field.estree.via {
        return generate_via_field_deserialization(
            field,
            converter_name,
            &estree_name,
            struct_def,
            schema,
        );
    }

    // Handle flattened fields - pass the entire parent object to deserialize the field
    // because its properties are merged into the parent in ESTree representation
    if field.estree.flatten {
        return (
            quote! {
                let #field_ident = FromESTree::from_estree(json, allocator)?;
            },
            true, // allocator is passed to FromESTree
        );
    }

    // Check if this field has appended or prepended fields (e.g., `rest` appended to `properties`,
    // or `directives` prepended to `body`).
    // In ESTree, these are combined into a single array, but in oxc they're separate fields.
    if field.estree.append_field_index.is_some() || field.estree.prepend_field_index.is_some() {
        return generate_field_with_concat_deserialization(field, &estree_name, struct_def, schema);
    }

    // Determine if field is optional
    let is_optional = matches!(field_type, TypeDef::Option(_));

    // TypeScript-specific fields (#[ts]) should be treated as optional with defaults
    // since JS parsers won't provide them
    let is_ts_only = field.estree.is_ts;

    if is_optional {
        // For Option<T>, use estree_field_opt
        (
            quote! {
                let #field_ident = match json.estree_field_opt(#estree_name_str) {
                    Some(field_json) if !field_json.is_null() => {
                        Some(FromESTree::from_estree(field_json, allocator)?)
                    }
                    _ => None,
                };
            },
            true, // allocator is passed to FromESTree
        )
    } else if is_ts_only {
        // For TypeScript-only fields, try to get the field but default if not present
        // This allows JS parsers (which don't have TS fields) to work
        let (default_value, _) = generate_default_for_type(field_type, field, schema);
        (
            quote! {
                let #field_ident = match json.estree_field_opt(#estree_name_str) {
                    Some(field_json) if !field_json.is_null() => {
                        FromESTree::from_estree(field_json, allocator)?
                    }
                    _ => #default_value,
                };
            },
            true, // allocator is passed to FromESTree
        )
    } else {
        // For required fields
        (
            quote! {
                let #field_ident = FromESTree::from_estree(
                    json.estree_field(#estree_name_str)?,
                    allocator
                )?;
            },
            true, // allocator is passed to FromESTree
        )
    }
}

/// Generate deserialization for a Vec field that has elements prepended or appended to it in ESTree.
/// For example:
/// - `ObjectPattern.properties` has `rest` appended, so the ESTree array contains both `Property`
///   and `RestElement` nodes, but oxc separates them.
/// - `Program.body` has `directives` prepended, so ESTree's `body` array starts with directives.
/// Returns (TokenStream, uses_allocator bool).
fn generate_field_with_concat_deserialization(
    field: &FieldDef,
    estree_name: &str,
    struct_def: &StructDef,
    schema: &Schema,
) -> (TokenStream, bool) {
    let field_ident = field.ident();
    let estree_name_str = estree_name;

    // Collect info about prepended and appended fields
    let prepend_info = field.estree.prepend_field_index.map(|idx| {
        let f = &struct_def.fields[idx];
        let ty = f.type_def(schema);
        let inner_ty = ty.innermost_type(schema);
        let type_name = match inner_ty {
            TypeDef::Struct(s) => s.estree.rename.as_deref().unwrap_or(s.name()).to_string(),
            _ => inner_ty.name().to_string(),
        };
        (f.ident(), type_name, matches!(ty, TypeDef::Option(_)))
    });

    let append_info = field.estree.append_field_index.map(|idx| {
        let f = &struct_def.fields[idx];
        let ty = f.type_def(schema);
        let inner_ty = ty.innermost_type(schema);
        let type_name = match inner_ty {
            TypeDef::Struct(s) => s.estree.rename.as_deref().unwrap_or(s.name()).to_string(),
            _ => inner_ty.name().to_string(),
        };
        (f.ident(), type_name, matches!(ty, TypeDef::Option(_)))
    });

    // Build the output bindings
    let mut output_bindings = vec![field_ident.clone()];
    if let Some((ident, _, _)) = &prepend_info {
        output_bindings.insert(0, ident.clone());
    }
    if let Some((ident, _, _)) = &append_info {
        output_bindings.push(ident.clone());
    }

    // Generate the filtering logic
    // For prepended elements, we need special handling for types like Directive
    // which share the same ESTree type as regular statements. We use the presence
    // of the "directive" field to distinguish them.
    let prepend_check = prepend_info.as_ref().map(|(_, type_name, is_optional)| {
        // Special case: Directive has ESTree type "ExpressionStatement" but has a "directive" field
        let is_directive = type_name == "ExpressionStatement";
        let check_condition = if is_directive {
            // Check for presence of "directive" field to identify directives
            quote! { elem.get("directive").is_some() }
        } else {
            quote! { elem_type == #type_name }
        };

        if *is_optional {
            quote! {
                if #check_condition {
                    prepended_element = Some(ABox::new_in(
                        FromESTree::from_estree(elem, allocator)?,
                        allocator
                    ));
                    continue;
                }
            }
        } else {
            quote! {
                if #check_condition {
                    prepended_elements.push(FromESTree::from_estree(elem, allocator)?);
                    continue;
                }
            }
        }
    });

    let append_check = append_info.as_ref().map(|(_, type_name, is_optional)| {
        if *is_optional {
            quote! {
                if elem_type == #type_name {
                    appended_element = Some(ABox::new_in(
                        FromESTree::from_estree(elem, allocator)?,
                        allocator
                    ));
                    continue;
                }
            }
        } else {
            quote! {
                if elem_type == #type_name {
                    appended_elements.push(FromESTree::from_estree(elem, allocator)?);
                    continue;
                }
            }
        }
    });

    // Generate initializers for prepend/append collections
    let prepend_init = prepend_info.as_ref().map(|(_, _, is_optional)| {
        if *is_optional {
            quote! { let mut prepended_element = None; }
        } else {
            quote! { let mut prepended_elements = AVec::new_in(allocator); }
        }
    });

    let append_init = append_info.as_ref().map(|(_, _, is_optional)| {
        if *is_optional {
            quote! { let mut appended_element = None; }
        } else {
            quote! { let mut appended_elements = AVec::new_in(allocator); }
        }
    });

    // Generate the final tuple based on what fields we have
    let result_tuple = match (&prepend_info, &append_info) {
        (Some((_, _, is_prep_opt)), Some((_, _, is_app_opt))) => {
            let prep_val =
                if *is_prep_opt { quote!(prepended_element) } else { quote!(prepended_elements) };
            let app_val =
                if *is_app_opt { quote!(appended_element) } else { quote!(appended_elements) };
            quote! { (#prep_val, main_elements, #app_val) }
        }
        (Some((_, _, is_prep_opt)), None) => {
            let prep_val =
                if *is_prep_opt { quote!(prepended_element) } else { quote!(prepended_elements) };
            quote! { (#prep_val, main_elements) }
        }
        (None, Some((_, _, is_app_opt))) => {
            let app_val =
                if *is_app_opt { quote!(appended_element) } else { quote!(appended_elements) };
            quote! { (main_elements, #app_val) }
        }
        (None, None) => {
            // Shouldn't happen, but handle it
            quote! { main_elements }
        }
    };

    (
        quote! {
            let (#(#output_bindings),*) = {
                let arr = json.estree_field(#estree_name_str)?
                    .as_array()
                    .ok_or(DeserError::ExpectedArray)?;

                #prepend_init
                let mut main_elements = AVec::with_capacity_in(arr.len(), allocator);
                #append_init

                for elem in arr {
                    let elem_type = elem.estree_type()?;
                    #prepend_check
                    #append_check
                    // Regular element
                    main_elements.push(FromESTree::from_estree(elem, allocator)?);
                }

                #result_tuple
            };
        },
        true, // allocator is used
    )
}

/// Check if a field is the source of an `append_to` attribute.
/// This means this field's value is appended to another field in ESTree representation,
/// and its deserialization is handled by that target field.
fn is_appended_to_another_field(field: &FieldDef, struct_def: &StructDef) -> bool {
    // A field is appended to another if any other field has this field's index in its
    // append_field_index or prepend_field_index
    let field_index = struct_def.fields.iter().position(|f| f.name() == field.name());
    if let Some(field_index) = field_index {
        struct_def.fields.iter().any(|other_field| {
            other_field.estree.append_field_index == Some(field_index)
                || other_field.estree.prepend_field_index == Some(field_index)
        })
    } else {
        false
    }
}

/// Generate deserialization for a field with `#[estree(via = ...)]` converter.
/// Returns (TokenStream, uses_allocator bool).
fn generate_via_field_deserialization(
    field: &FieldDef,
    converter_name: &str,
    estree_name: &str,
    struct_def: &StructDef,
    schema: &Schema,
) -> (TokenStream, bool) {
    let field_ident = field.ident();
    let estree_name_str = estree_name;

    // Get the converter path
    let krate = struct_def.file(schema).krate();
    let converter_path = get_converter_path(converter_name, krate, schema);

    // Fields with `via` are typically optional in ESTree (might be empty array, null, etc.)
    // Use the converter's from_estree_converter method
    (
        quote! {
            let #field_ident = match json.estree_field_opt(#estree_name_str) {
                Some(field_json) => {
                    #converter_path::from_estree_converter(field_json, allocator)?
                }
                None => {
                    #converter_path::from_estree_converter(&serde_json::Value::Null, allocator)?
                }
            };
        },
        true, // allocator is passed to the converter
    )
}

/// Get the path to a converter type.
/// Uses the same approach as the ESTree derive - look up the converter in the schema's meta types.
fn get_converter_path(converter_name: &str, from_krate: &str, schema: &Schema) -> TokenStream {
    let converter = schema.meta_by_name(converter_name);
    converter.import_path_from_crate(from_krate, schema)
}

/// Generate a default value for a type.
/// Returns (default value TokenStream, uses_allocator bool).
fn generate_default_for_type(
    type_def: &TypeDef,
    _field: &FieldDef,
    schema: &Schema,
) -> (TokenStream, bool) {
    match type_def {
        TypeDef::Primitive(prim) => {
            let default = match prim.name() {
                "bool" => quote!(false),
                "u8" | "u16" | "u32" | "u64" | "usize" => quote!(0),
                "i8" | "i16" | "i32" | "i64" | "isize" => quote!(0),
                "f32" | "f64" => quote!(0.0),
                _ => quote!(Default::default()),
            };
            (default, false)
        }
        TypeDef::Option(_) => (quote!(None), false),
        TypeDef::Vec(_) => {
            // Vec needs allocator - use AVec::new_in
            (quote!(AVec::new_in(allocator)), true)
        }
        TypeDef::Box(box_def) => {
            // Box<T> needs allocator and inner default - can't easily default
            let inner_type = box_def.inner_type(schema);
            let inner_name = inner_type.name();
            (
                quote!(panic!("Cannot default Box<{}> - field should not be skipped", #inner_name)),
                false,
            )
        }
        _ => {
            // Check the innermost type name for known types without Default
            let inner_type = type_def.innermost_type(schema);
            let type_name = inner_type.name();
            match type_name {
                "VariableDeclarationKind" => {
                    (quote!(crate::ast::js::VariableDeclarationKind::Var), false)
                }
                "WithClauseKeyword" => (quote!(crate::ast::js::WithClauseKeyword::With), false),
                "NumberBase" => (quote!(oxc_syntax::number::NumberBase::Decimal), false),
                "BigintBase" => (quote!(oxc_syntax::number::BigintBase::Decimal), false),
                "RegExpFlags" => (quote!(crate::ast::literal::RegExpFlags::empty()), false),
                "ImportOrExportKind" => (quote!(crate::ast::ts::ImportOrExportKind::Value), false),
                _ => (quote!(Default::default()), false),
            }
        }
    }
}

/// Generate default value for a skipped field.
/// Returns (TokenStream, uses_allocator bool).
fn generate_default_field(field: &FieldDef, schema: &Schema) -> (TokenStream, bool) {
    let field_ident = field.ident();
    let field_type = field.type_def(schema);

    // Try to generate a sensible default based on type
    let (default_value, uses_allocator) = match field_type {
        TypeDef::Option(_) => (quote!(None), false),
        TypeDef::Vec(_) => (quote!(AVec::new_in(allocator)), true),
        TypeDef::Cell(_) => (quote!(std::cell::Cell::default()), false),
        TypeDef::Primitive(p) => {
            let default = match p.name() {
                "bool" => quote!(false),
                "u8" | "u16" | "u32" | "u64" | "usize" => quote!(0),
                "i8" | "i16" | "i32" | "i64" | "isize" => quote!(0),
                "f32" | "f64" => quote!(0.0),
                _ => quote!(Default::default()),
            };
            (default, false)
        }
        TypeDef::Box(box_def) => {
            // Box<T> needs allocator and inner default - can't easily default
            // Skip this field entirely with a panic placeholder
            let inner_type = box_def.inner_type(schema);
            let inner_name = inner_type.name();
            (
                quote!(panic!("Cannot default Box<{}> - field should not be skipped", #inner_name)),
                false,
            )
        }
        _ => {
            // Check the innermost type name for known types without Default
            let inner_type = field_type.innermost_type(schema);
            let type_name = inner_type.name();
            match type_name {
                // Specific defaults for types that don't implement Default
                "VariableDeclarationKind" => {
                    (quote!(crate::ast::js::VariableDeclarationKind::Var), false)
                }
                "WithClauseKeyword" => (quote!(crate::ast::js::WithClauseKeyword::With), false),
                "NumberBase" => (quote!(oxc_syntax::number::NumberBase::Decimal), false),
                "BigintBase" => (quote!(oxc_syntax::number::BigintBase::Decimal), false),
                "RegExpFlags" => (quote!(crate::ast::literal::RegExpFlags::empty()), false),
                _ => (quote!(Default::default()), false),
            }
        }
    };

    (
        quote! {
            let #field_ident = #default_value;
        },
        uses_allocator,
    )
}

/// Generate `FromESTree` implementation for an enum.
fn generate_impl_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    // Use concrete lifetime 'a (not anonymous '_) since we return data with that lifetime
    let ty = enum_def.ty_with_lifetime(schema, false);

    // Check if this is a fieldless enum (like PropertyKind)
    if enum_def.is_fieldless() {
        return generate_impl_for_fieldless_enum(enum_def, schema);
    }

    // For enums with fields, we need to dispatch based on the `type` field
    let body = generate_body_for_enum_with_fields(enum_def, schema);

    quote! {
        impl<'a> FromESTree<'a> for #ty {
            fn from_estree(json: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
                #body
            }
        }
    }
}

/// Generate `FromESTree` implementation for a fieldless enum (string value).
fn generate_impl_for_fieldless_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    // Use concrete lifetime 'a (not anonymous '_) since we return data with that lifetime
    let ty = enum_def.ty_with_lifetime(schema, false);

    let match_arms = enum_def
        .all_variants(schema)
        .filter(|variant| !should_skip_enum_variant(variant))
        .map(|variant| {
            let variant_ident = variant.ident();
            let estree_value = get_fieldless_variant_value(enum_def, variant);
            let estree_value_str = estree_value.as_ref();
            quote! {
                #estree_value_str => Ok(Self::#variant_ident),
            }
        })
        .collect::<Vec<_>>();

    let enum_name = enum_def.name();

    quote! {
        impl<'a> FromESTree<'a> for #ty {
            fn from_estree(json: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
                let s = json.as_str().ok_or(DeserError::ExpectedString)?;
                match s {
                    #(#match_arms)*
                    other => Err(DeserError::InvalidFieldValue(
                        #enum_name,
                        other.to_string()
                    )),
                }
            }
        }
    }
}

/// Returns true if this enum type should use placeholders for unknown node types
/// instead of returning an error.
///
/// According to the plan:
/// - Expression → NullLiteral placeholder
/// - Statement → EmptyStatement placeholder
/// - ClassElement → StaticBlock placeholder (for custom syntax like GlimmerTemplate in class bodies)
fn enum_uses_placeholder(enum_name: &str) -> Option<&'static str> {
    match enum_name {
        "Expression" => Some("NullLiteral"),
        "Statement" => Some("EmptyStatement"),
        // ClassElement needs placeholders because custom syntax like GlimmerTemplate
        // can appear in class bodies (e.g., Ember/Glimmer components)
        "ClassElement" => Some("StaticBlock"),
        _ => None,
    }
}

/// Generate body for enum with fields - dispatch based on `type` field.
fn generate_body_for_enum_with_fields(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    // Collect match arms, tracking duplicates
    let mut arms_by_type: std::collections::HashMap<String, Vec<(&VariantDef, TokenStream)>> =
        std::collections::HashMap::new();

    for variant in enum_def.all_variants(schema) {
        if should_skip_enum_variant(variant) {
            continue;
        }
        // Get all type names this variant can match (some types like Class have multiple)
        for (type_name, arm) in generate_enum_variant_arms_with_types(variant, schema) {
            arms_by_type.entry(type_name).or_default().push((variant, arm));
        }
    }

    // Generate match arms, handling duplicates with disambiguation
    let match_arms: Vec<TokenStream> = arms_by_type
        .into_iter()
        .map(|(type_name, variants)| {
            if variants.len() == 1 {
                // Single variant for this type - use directly
                let (_, arm) = &variants[0];
                let type_name_str = type_name.as_str();
                quote! { #type_name_str => { #arm } }
            } else {
                // Multiple variants map to same type - need disambiguation
                let type_name_str = type_name.as_str();
                let disambiguation = generate_disambiguation(&type_name, &variants, schema);
                quote! { #type_name_str => { #disambiguation } }
            }
        })
        .collect();

    // Determine fallback behavior for unknown types
    let enum_name = enum_def.name();
    let fallback_arm = if let Some(placeholder_type) = enum_uses_placeholder(enum_name) {
        // Use a placeholder for unknown nodes in this enum type
        // This allows custom syntax (GlimmerTemplate, SvelteComponent, etc.) to be skipped
        // while Rust rules continue running on the surrounding valid JS/TS code.
        // We also record the span so diagnostics in these regions can be filtered out.
        match placeholder_type {
            "NullLiteral" => quote! {
                // Unknown node type (likely custom syntax) - use NullLiteral placeholder
                _other => {
                    let span = parse_span_or_empty(json);
                    record_unknown_span(span);
                    Ok(Self::NullLiteral(ABox::new_in(
                        crate::ast::literal::NullLiteral { span },
                        allocator
                    )))
                },
            },
            "EmptyStatement" => quote! {
                // Unknown node type (likely custom syntax) - use EmptyStatement placeholder
                _other => {
                    let span = parse_span_or_empty(json);
                    record_unknown_span(span);
                    Ok(Self::EmptyStatement(ABox::new_in(
                        crate::ast::js::EmptyStatement { span },
                        allocator
                    )))
                },
            },
            "StaticBlock" => quote! {
                // Unknown node type (likely custom syntax like GlimmerTemplate) - use StaticBlock placeholder
                _other => {
                    let span = parse_span_or_empty(json);
                    record_unknown_span(span);
                    Ok(Self::StaticBlock(ABox::new_in(
                        crate::ast::js::StaticBlock { span, body: AVec::new_in(allocator), scope_id: std::cell::Cell::default() },
                        allocator
                    )))
                },
            },
            _ => quote! {
                other => Err(DeserError::UnknownNodeType(other.to_string())),
            },
        }
    } else {
        // No placeholder - return error for unknown types
        quote! {
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    };

    quote! {
        let type_name = json.estree_type()?;
        match type_name {
            #(#match_arms)*
            #fallback_arm
        }
    }
}

/// Generate disambiguation logic for variants that share an ESTree type name.
fn generate_disambiguation(
    type_name: &str,
    variants: &[(&VariantDef, TokenStream)],
    schema: &Schema,
) -> TokenStream {
    match type_name {
        "Literal" => generate_literal_disambiguation(variants, schema),
        "MemberExpression" => generate_member_expression_disambiguation(variants, schema),
        "BinaryExpression" => generate_binary_expression_disambiguation(variants, schema),
        "TSModuleDeclaration" => generate_ts_module_declaration_disambiguation(variants, schema),
        "JSXIdentifier" => generate_jsx_identifier_disambiguation(variants, schema),
        "Property" => generate_property_disambiguation(variants, schema),
        _ => {
            // Unknown disambiguation case - use first variant
            let (_, arm) = &variants[0];
            arm.clone()
        }
    }
}

/// Generate disambiguation for Literal variants based on value type.
fn generate_literal_disambiguation(
    variants: &[(&VariantDef, TokenStream)],
    schema: &Schema,
) -> TokenStream {
    // Find specific literal variants
    let mut null_arm: Option<TokenStream> = None;
    let mut bool_arm: Option<TokenStream> = None;
    let mut number_arm: Option<TokenStream> = None;
    let mut bigint_arm: Option<TokenStream> = None;
    let mut regex_arm: Option<TokenStream> = None;
    let mut string_arm: Option<TokenStream> = None;

    for (variant, arm) in variants {
        let inner_type = variant.field_type(schema);
        let inner_name = inner_type.map(|t| t.innermost_type(schema).name()).unwrap_or("");

        match inner_name {
            "NullLiteral" => null_arm = Some(arm.clone()),
            "BooleanLiteral" => bool_arm = Some(arm.clone()),
            "NumericLiteral" => number_arm = Some(arm.clone()),
            "BigIntLiteral" => bigint_arm = Some(arm.clone()),
            "RegExpLiteral" => regex_arm = Some(arm.clone()),
            "StringLiteral" => string_arm = Some(arm.clone()),
            _ => {}
        }
    }

    // Default fallback - use first variant
    let fallback = variants[0].1.clone();

    // Use fallback for any missing arms
    let null_arm = null_arm.unwrap_or_else(|| fallback.clone());
    let bool_arm = bool_arm.unwrap_or_else(|| fallback.clone());
    let number_arm = number_arm.unwrap_or_else(|| fallback.clone());
    let bigint_arm = bigint_arm.unwrap_or_else(|| fallback.clone());
    let regex_arm = regex_arm.unwrap_or_else(|| fallback.clone());
    let string_arm = string_arm.unwrap_or_else(|| fallback.clone());

    quote! {
        // Disambiguate Literal based on value type
        if json.get("value").is_some_and(|v| v.is_null()) {
            #null_arm
        } else if json.get("value").is_some_and(|v| v.is_boolean()) {
            #bool_arm
        } else if json.get("value").is_some_and(|v| v.is_number()) {
            #number_arm
        } else if json.get("bigint").is_some() {
            #bigint_arm
        } else if json.get("regex").is_some() {
            #regex_arm
        } else if json.get("value").is_some_and(|v| v.is_string()) {
            #string_arm
        } else {
            #fallback
        }
    }
}

/// Generate disambiguation for MemberExpression variants based on computed/property.
fn generate_member_expression_disambiguation(
    variants: &[(&VariantDef, TokenStream)],
    schema: &Schema,
) -> TokenStream {
    let mut computed_arm: Option<TokenStream> = None;
    let mut static_arm: Option<TokenStream> = None;
    let mut private_arm: Option<TokenStream> = None;

    for (variant, arm) in variants {
        let inner_type = variant.field_type(schema);
        let inner_name = inner_type.map(|t| t.innermost_type(schema).name()).unwrap_or("");

        match inner_name {
            "ComputedMemberExpression" => computed_arm = Some(arm.clone()),
            "StaticMemberExpression" => static_arm = Some(arm.clone()),
            "PrivateFieldExpression" => private_arm = Some(arm.clone()),
            _ => {}
        }
    }

    let fallback = variants[0].1.clone();
    let computed_arm = computed_arm.unwrap_or_else(|| fallback.clone());
    let static_arm = static_arm.unwrap_or_else(|| fallback.clone());
    let private_arm = private_arm.unwrap_or_else(|| fallback.clone());

    quote! {
        // Disambiguate MemberExpression based on computed flag and property type
        let is_computed = json.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
        let property_type = json.get("property").and_then(|v| v.get("type")).and_then(|v| v.as_str());

        if property_type == Some("PrivateIdentifier") {
            #private_arm
        } else if is_computed {
            #computed_arm
        } else {
            #static_arm
        }
    }
}

/// Generate disambiguation for BinaryExpression (vs PrivateInExpression).
fn generate_binary_expression_disambiguation(
    variants: &[(&VariantDef, TokenStream)],
    schema: &Schema,
) -> TokenStream {
    let mut binary_arm: Option<TokenStream> = None;
    let mut private_in_arm: Option<TokenStream> = None;

    for (variant, arm) in variants {
        let inner_type = variant.field_type(schema);
        let inner_name = inner_type.map(|t| t.innermost_type(schema).name()).unwrap_or("");

        match inner_name {
            "BinaryExpression" => binary_arm = Some(arm.clone()),
            "PrivateInExpression" => private_in_arm = Some(arm.clone()),
            _ => {}
        }
    }

    let fallback = variants[0].1.clone();
    let binary_arm = binary_arm.unwrap_or_else(|| fallback.clone());
    let private_in_arm = private_in_arm.unwrap_or_else(|| fallback.clone());

    quote! {
        // Disambiguate BinaryExpression vs PrivateInExpression
        let operator = json.get("operator").and_then(|v| v.as_str());
        let left_type = json.get("left").and_then(|v| v.get("type")).and_then(|v| v.as_str());

        if operator == Some("in") && left_type == Some("PrivateIdentifier") {
            #private_in_arm
        } else {
            #binary_arm
        }
    }
}

/// Generate disambiguation for TSModuleDeclaration (vs TSGlobalDeclaration).
/// Both types serialize to ESTree type "TSModuleDeclaration", but TSGlobalDeclaration
/// has `kind: "global"` while TSModuleDeclaration has `kind: "module"` or `kind: "namespace"`.
fn generate_ts_module_declaration_disambiguation(
    variants: &[(&VariantDef, TokenStream)],
    schema: &Schema,
) -> TokenStream {
    let mut module_arm: Option<TokenStream> = None;
    let mut global_arm: Option<TokenStream> = None;

    for (variant, arm) in variants {
        let inner_type = variant.field_type(schema);
        let inner_name = inner_type.map(|t| t.innermost_type(schema).name()).unwrap_or("");

        match inner_name {
            "TSModuleDeclaration" => module_arm = Some(arm.clone()),
            "TSGlobalDeclaration" => global_arm = Some(arm.clone()),
            _ => {}
        }
    }

    let fallback = variants[0].1.clone();
    let module_arm = module_arm.unwrap_or_else(|| fallback.clone());
    let global_arm = global_arm.unwrap_or_else(|| fallback.clone());

    quote! {
        // Disambiguate TSModuleDeclaration vs TSGlobalDeclaration based on kind field
        let kind = json.get("kind").and_then(|v| v.as_str());

        if kind == Some("global") {
            #global_arm
        } else {
            #module_arm
        }
    }
}

/// Generate disambiguation for JSXIdentifier (IdentifierReference vs ThisExpression).
/// Both serialize to ESTree type "JSXIdentifier", but ThisExpression has `name: "this"`.
fn generate_jsx_identifier_disambiguation(
    variants: &[(&VariantDef, TokenStream)],
    schema: &Schema,
) -> TokenStream {
    let mut identifier_arm: Option<TokenStream> = None;
    let mut this_arm: Option<TokenStream> = None;
    let mut jsx_identifier_arm: Option<TokenStream> = None;

    for (variant, arm) in variants {
        let inner_type = variant.field_type(schema);
        let inner_name = inner_type.map(|t| t.innermost_type(schema).name()).unwrap_or("");

        match inner_name {
            "IdentifierReference" => identifier_arm = Some(arm.clone()),
            "ThisExpression" => this_arm = Some(arm.clone()),
            "JSXIdentifier" => jsx_identifier_arm = Some(arm.clone()),
            _ => {}
        }
    }

    let fallback = variants[0].1.clone();
    let identifier_arm = identifier_arm.unwrap_or_else(|| fallback.clone());
    let this_arm = this_arm.unwrap_or_else(|| identifier_arm.clone());
    let jsx_identifier_arm = jsx_identifier_arm.unwrap_or_else(|| identifier_arm.clone());

    quote! {
        // Disambiguate JSXIdentifier variants based on name field
        let name = json.get("name").and_then(|v| v.as_str());

        if name == Some("this") {
            #this_arm
        } else {
            // Check if we have a dedicated JSXIdentifier variant, otherwise use IdentifierReference
            #jsx_identifier_arm
        }
    }
}

/// Generate disambiguation for Property (AssignmentTargetPropertyIdentifier vs AssignmentTargetPropertyProperty,
/// BindingProperty shorthand vs non-shorthand, ObjectProperty, etc.).
/// These types all serialize to ESTree type "Property" but differ by `shorthand` field.
fn generate_property_disambiguation(
    variants: &[(&VariantDef, TokenStream)],
    schema: &Schema,
) -> TokenStream {
    let mut shorthand_arm: Option<TokenStream> = None;
    let mut non_shorthand_arm: Option<TokenStream> = None;
    let mut object_property_arm: Option<TokenStream> = None;
    let mut binding_property_arm: Option<TokenStream> = None;

    for (variant, arm) in variants {
        let inner_type = variant.field_type(schema);
        let inner_name = inner_type.map(|t| t.innermost_type(schema).name()).unwrap_or("");

        match inner_name {
            // AssignmentTargetProperty variants
            "AssignmentTargetPropertyIdentifier" => shorthand_arm = Some(arm.clone()),
            "AssignmentTargetPropertyProperty" => non_shorthand_arm = Some(arm.clone()),
            // Object/Binding property types
            "ObjectProperty" => object_property_arm = Some(arm.clone()),
            "BindingProperty" => binding_property_arm = Some(arm.clone()),
            _ => {}
        }
    }

    let fallback = variants[0].1.clone();

    // Use shorthand for disambiguation between AssignmentTarget variants
    if shorthand_arm.is_some() || non_shorthand_arm.is_some() {
        let shorthand_arm = shorthand_arm.unwrap_or_else(|| fallback.clone());
        let non_shorthand_arm = non_shorthand_arm.unwrap_or_else(|| fallback.clone());

        return quote! {
            // Disambiguate Property variants based on shorthand field
            let is_shorthand = json.get("shorthand").and_then(|v| v.as_bool()).unwrap_or(false);

            if is_shorthand {
                #shorthand_arm
            } else {
                #non_shorthand_arm
            }
        };
    }

    // For ObjectProperty vs BindingProperty, we need context-based dispatch
    // which should be handled by the parent enum. Fall back to first variant.
    object_property_arm.or(binding_property_arm).unwrap_or(fallback)
}

/// Generate a match arm for an enum variant, returning (type_name, arm_body).
/// Generate match arms for an enum variant.
/// Returns a list of (type_name, arm_body) pairs because some types like Class
/// can match multiple ESTree type names (ClassDeclaration, ClassExpression).
fn generate_enum_variant_arms_with_types(
    variant: &VariantDef,
    schema: &Schema,
) -> Vec<(String, TokenStream)> {
    let variant_ident = variant.ident();
    let variant_name = variant.name();

    // Get the inner type of the variant
    let inner_type = match variant.field_type(schema) {
        Some(t) => t,
        None => return vec![],
    };

    // Check if this variant has a `via` converter that changes the ESTree type name
    if let Some(via_name) = &variant.estree.via {
        // Look up the converter's ts_type to get the ESTree type name
        let meta = schema.meta_by_name(via_name);
        if let Some(ts_type) = &meta.estree.ts_type {
            // The converter produces a different type - use the converter for deserialization
            let converter_path = get_converter_path(via_name, "oxc_ast", schema);
            let arm_body = quote! {
                Ok(Self::#variant_ident(#converter_path::from_estree_converter(json, allocator)?))
            };
            return vec![(ts_type.clone(), arm_body)];
        }
    }

    // Check if this is a Box type (most enum variants are Box<T>)
    let is_boxed = matches!(inner_type, TypeDef::Box(_));

    let arm_body = if is_boxed {
        quote! {
            Ok(Self::#variant_ident(ABox::new_in(FromESTree::from_estree(json, allocator)?, allocator)))
        }
    } else {
        quote! {
            Ok(Self::#variant_ident(FromESTree::from_estree(json, allocator)?))
        }
    };

    // Get all ESTree type names this variant can match
    // Use variant name to determine type names (for Class/Function variants)
    let type_names = get_estree_type_names_for_variant(variant_name, inner_type, schema);

    type_names.into_iter().map(|name| (name, arm_body.clone())).collect()
}

/// Get ESTree `type` field values for an enum variant.
/// Uses the variant name to determine the appropriate type name(s).
/// This is important for types like Class/Function where the same inner type
/// is used in different contexts (ClassDeclaration vs ClassExpression).
fn get_estree_type_names_for_variant(
    variant_name: &str,
    type_def: &TypeDef,
    schema: &Schema,
) -> Vec<String> {
    // Unwrap Box/Vec/Option to get to the actual type
    let inner_type = type_def.innermost_type(schema);

    // First, check if the variant name itself is one of the known ESTree type names.
    // For example, Statement::ClassDeclaration should match "ClassDeclaration",
    // and Expression::ClassExpression should match "ClassExpression".
    match variant_name {
        // Class variants - use the variant name directly
        "ClassDeclaration" => return vec!["ClassDeclaration".to_string()],
        "ClassExpression" => return vec!["ClassExpression".to_string()],
        // Function variants - use the variant name directly
        // FunctionDeclaration also matches TSDeclareFunction since they share the same oxc struct
        "FunctionDeclaration" => {
            return vec!["FunctionDeclaration".to_string(), "TSDeclareFunction".to_string()];
        }
        "FunctionExpression" => {
            return vec![
                "FunctionExpression".to_string(),
                "TSEmptyBodyFunctionExpression".to_string(),
            ];
        }
        // MethodDefinition variants in ClassElement enum
        "MethodDefinition" => {
            return vec!["MethodDefinition".to_string(), "TSAbstractMethodDefinition".to_string()];
        }
        // PropertyDefinition variants in ClassElement enum
        "PropertyDefinition" => {
            return vec![
                "PropertyDefinition".to_string(),
                "TSAbstractPropertyDefinition".to_string(),
            ];
        }
        // AccessorProperty variants in ClassElement enum
        "AccessorProperty" => {
            return vec!["AccessorProperty".to_string(), "TSAbstractAccessorProperty".to_string()];
        }
        _ => {}
    }

    // Fall back to getting type names from the inner type
    match inner_type {
        TypeDef::Struct(struct_def) => {
            let name = struct_def.name();
            // Use the renamed name if specified, otherwise use the struct name
            let type_name = struct_def.estree.rename.as_deref().unwrap_or(name);
            vec![type_name.to_string()]
        }
        TypeDef::Enum(enum_def) => {
            // Enums don't have a single type name - this shouldn't happen for enum variants
            vec![enum_def.name().to_string()]
        }
        _ => {
            // Primitives don't have type names
            vec!["unknown".to_string()]
        }
    }
}
