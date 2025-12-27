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
                parse_span, parse_span_or_empty,
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
    _struct_def: &StructDef,
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

    // Skip fields marked with #[estree(skip)]
    if should_skip_field(field, schema) {
        return generate_default_field(field, schema);
    }

    // Get the ESTree field name (might be renamed)
    let estree_name = get_struct_field_name(field);
    let estree_name_str = estree_name.as_ref();

    // Get field type
    let field_type = field.type_def(schema);

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
                "ImportOrExportKind" => {
                    (quote!(crate::ast::ts::ImportOrExportKind::Value), false)
                }
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
/// - Declaration → EmptyStatement placeholder (valid declaration)
fn enum_uses_placeholder(enum_name: &str) -> Option<&'static str> {
    match enum_name {
        "Expression" => Some("NullLiteral"),
        "Statement" => Some("EmptyStatement"),
        // Note: Declaration does NOT use placeholders because it doesn't have
        // a simple "no-op" variant like EmptyStatement. Unknown declarations
        // will cause an error, and lint_with_external_ast handles this by
        // returning UnknownNode result and skipping Rust rules.
        // This is acceptable because:
        // - Most custom syntax appears at statement level, not declaration level
        // - JS plugin rules still run and handle custom syntax
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
        if let Some((type_name, arm)) = generate_enum_variant_arm_with_type(variant, schema) {
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
        // while Rust rules continue running on the surrounding valid JS/TS code
        match placeholder_type {
            "NullLiteral" => quote! {
                // Unknown node type (likely custom syntax) - use NullLiteral placeholder
                _other => Ok(Self::NullLiteral(ABox::new_in(
                    crate::ast::literal::NullLiteral { span: parse_span_or_empty(json) },
                    allocator
                ))),
            },
            "EmptyStatement" => quote! {
                // Unknown node type (likely custom syntax) - use EmptyStatement placeholder
                _other => Ok(Self::EmptyStatement(ABox::new_in(
                    crate::ast::js::EmptyStatement { span: parse_span_or_empty(json) },
                    allocator
                ))),
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
        "Class" | "Function" => {
            // Class/Function can be expression or declaration - use first one
            // (The correct one depends on context, which we don't have here)
            let (_, arm) = &variants[0];
            arm.clone()
        }
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

/// Generate a match arm for an enum variant, returning (type_name, arm_body).
fn generate_enum_variant_arm_with_type(
    variant: &VariantDef,
    schema: &Schema,
) -> Option<(String, TokenStream)> {
    let variant_ident = variant.ident();

    // Get the inner type of the variant
    let inner_type = variant.field_type(schema)?;

    // Get the ESTree type name for this variant
    let estree_type_name = get_estree_type_name_for_type(inner_type, schema);

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

    Some((estree_type_name.to_string(), arm_body))
}

/// Get the ESTree `type` field value for a type.
fn get_estree_type_name_for_type<'a>(type_def: &'a TypeDef, schema: &'a Schema) -> &'a str {
    // Unwrap Box/Vec/Option to get to the actual type
    let inner_type = type_def.innermost_type(schema);

    match inner_type {
        TypeDef::Struct(struct_def) => {
            // Use the renamed name if specified, otherwise use the struct name
            struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name())
        }
        TypeDef::Enum(enum_def) => {
            // Enums don't have a single type name - this shouldn't happen for enum variants
            enum_def.name()
        }
        _ => {
            // Primitives don't have type names
            "unknown"
        }
    }
}
