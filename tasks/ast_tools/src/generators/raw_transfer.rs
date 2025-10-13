//! Generator for raw transfer deserializer.

use std::{borrow::Cow, fmt::Debug, str};

use cow_utils::CowUtils;
use lazy_regex::{Captures, Lazy, Regex, lazy_regex, regex::Replacer};
use proc_macro2::TokenStream;
use quote::quote;
use rustc_hash::FxHashSet;

use oxc_allocator::Allocator;
use oxc_ast::{
    AstBuilder,
    ast::{
        Expression, LogicalOperator, ObjectExpression, ObjectPropertyKind, Program, PropertyKind,
    },
};
use oxc_ast_visit::VisitMut;
use oxc_span::SPAN;

use crate::{
    ALLOCATOR_CRATE_PATH, Generator, NAPI_PARSER_PACKAGE_PATH, OXLINT_APP_PATH,
    codegen::{Codegen, DeriveId},
    derives::estree::{
        get_fieldless_variant_value, get_struct_field_name, should_flatten_field,
        should_skip_enum_variant, should_skip_field,
    },
    output::{Output, javascript::VariantGenerator},
    schema::{
        BoxDef, CellDef, Def, EnumDef, FieldDef, MetaType, OptionDef, PointerDef, PrimitiveDef,
        Schema, StructDef, TypeDef, TypeId, VecDef,
        extensions::layout::{GetLayout, GetOffset},
    },
    utils::{FxIndexMap, format_cow, number_lit, upper_case_first, write_it},
};

use super::define_generator;

/// Offset of length field in `&str`
const STR_LEN_OFFSET: u32 = 8;

/// Bytes reserved for `malloc`'s metadata
const MALLOC_RESERVED_SIZE: u32 = 16;

/// Minimum alignment requirement for end of `Allocator`'s chunk
const ALLOCATOR_CHUNK_END_ALIGN: u32 = 16;

/// Size of block of memory used for raw transfer.
/// This size includes metadata stored after the `Allocator` chunk which contains AST data.
///
/// Must be a multiple of [`ALLOCATOR_CHUNK_END_ALIGN`].
/// 16 bytes less than 2 GiB, to allow 16 bytes for `malloc` metadata (like Bumpalo does).
const BLOCK_SIZE: u32 = (1 << 31) - MALLOC_RESERVED_SIZE; // 2 GiB - 16 bytes
const _: () = assert!(BLOCK_SIZE.is_multiple_of(ALLOCATOR_CHUNK_END_ALIGN));

/// Alignment of block of memory used for raw transfer.
const BLOCK_ALIGN: u64 = 1 << 32; // 4 GiB

// Offsets of `Vec`'s fields.
// `Vec` is `#[repr(transparent)]` and `RawVec` is `#[repr(C)]`, so these offsets are fixed.
pub(super) const VEC_PTR_FIELD_OFFSET: usize = 0;
pub(super) const VEC_LEN_FIELD_OFFSET: usize = 8;

/// Generator for raw transfer deserializer.
pub struct RawTransferGenerator;

define_generator!(RawTransferGenerator);

impl Generator for RawTransferGenerator {
    fn generate_many(&self, schema: &Schema, codegen: &Codegen) -> Vec<Output> {
        let consts = get_constants(schema);

        let deserializers = generate_deserializers(consts, schema, codegen);

        let (constants_js, constants_rust) = generate_constants(consts);

        let mut outputs = deserializers
            .into_iter()
            .map(|(path, code)| Output::Javascript { path, code })
            .collect::<Vec<_>>();

        outputs.extend([
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/constants.js"),
                code: constants_js.clone(),
            },
            Output::Javascript {
                // This file is also valid as TS
                path: format!("{OXLINT_APP_PATH}/src-js/generated/constants.ts"),
                code: constants_js,
            },
            Output::Rust {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/src/generated/raw_transfer_constants.rs"),
                tokens: constants_rust.clone(),
            },
            Output::Rust {
                path: format!("{OXLINT_APP_PATH}/src/generated/raw_transfer_constants.rs"),
                tokens: constants_rust.clone(),
            },
            Output::Rust {
                path: format!("{ALLOCATOR_CRATE_PATH}/src/generated/fixed_size_constants.rs"),
                tokens: constants_rust,
            },
        ]);

        outputs
    }
}

/// Generate deserializer functions for all types.
///
/// Generates a single file which is the base of both the JS and TS deserializers.
/// Code which is specific to JS or TS deserializer is gated by the `IS_TS` const,
/// code which adds `range` fields is gated by `RANGE` const.
/// e.g.:
/// * `if (IS_TS) node.typeAnnotation = null;`
/// * `return { type: 'Function', id, params, ...(IS_TS && { typeAnnotation: null }) };`
/// * `return { type: 'ThisExpression', start, end, ...(RANGE && { range: [start, end] }) };`
///
/// When printing the JS and TS deserializers, the value of `IS_TS` is set to `true` or `false`,
/// and minifier then shakes out the dead code for each.
#[expect(clippy::items_after_statements)]
fn generate_deserializers(
    consts: Constants,
    schema: &Schema,
    codegen: &Codegen,
) -> Vec<(/* path */ String, /* code */ String)> {
    let estree_derive_id = codegen.get_derive_id_by_name("ESTree");
    let span_type_id = schema.type_names["Span"];

    // Prelude to generated deserializer.
    // Defines the main `deserialize` function.
    let data_pointer_pos_32 = consts.data_pointer_pos / 4;

    #[rustfmt::skip]
    let mut code = format!("
        let uint8, uint32, float64, sourceText, sourceIsAscii, sourceByteLen;

        let parent = null;
        let getLoc;

        const textDecoder = new TextDecoder('utf-8', {{ ignoreBOM: true }}),
            decodeStr = textDecoder.decode.bind(textDecoder),
            {{ fromCodePoint }} = String;

        const NodeProto = Object.create(null, {{
            loc: {{
                get() {{
                    return getLoc(this);
                }},
                enumerable: true,
            }}
        }});

        export function deserialize(buffer, sourceText, sourceByteLen) {{
            return deserializeWith(buffer, sourceText, sourceByteLen, null, deserializeRawTransferData);
        }}

        export function deserializeProgramOnly(buffer, sourceText, sourceByteLen, getLoc) {{
            return deserializeWith(buffer, sourceText, sourceByteLen, getLoc, deserializeProgram);
        }}

        function deserializeWith(buffer, sourceTextInput, sourceByteLenInput, getLocInput, deserialize) {{
            uint8 = buffer;
            uint32 = buffer.uint32;
            float64 = buffer.float64;

            sourceText = sourceTextInput;
            sourceByteLen = sourceByteLenInput;
            sourceIsAscii = sourceText.length === sourceByteLen;

            if (LOC) getLoc = getLocInput;

            const data = deserialize(uint32[{data_pointer_pos_32}]);

            uint8 = uint32 = float64 = sourceText = undefined;

            return data;
        }}
    ");

    for type_def in &schema.types {
        match type_def {
            TypeDef::Struct(struct_def) => {
                generate_struct(struct_def, &mut code, span_type_id, estree_derive_id, schema);
            }
            TypeDef::Enum(enum_def) => {
                generate_enum(enum_def, &mut code, estree_derive_id, schema);
            }
            TypeDef::Primitive(primitive_def) => {
                generate_primitive(primitive_def, &mut code, schema);
            }
            TypeDef::Option(option_def) => {
                generate_option(option_def, &mut code, estree_derive_id, schema);
            }
            TypeDef::Box(box_def) => {
                generate_box(box_def, &mut code, estree_derive_id, schema);
            }
            TypeDef::Vec(vec_def) => {
                generate_vec(vec_def, &mut code, estree_derive_id, schema);
            }
            TypeDef::Cell(_cell_def) => {
                // No deserializers for `Cell`s - use inner type's deserializer
            }
            TypeDef::Pointer(_pointer_def) => {
                // No deserializers for pointers - use `Box`'s deserializer.
                // TODO: Need to make sure deserializer for `Box<T>` is generated.
            }
        }
    }

    // Create deserializers with various settings, by setting `IS_TS`, `RANGE`, `LOC`, `PARENT`
    // and `PRESERVE_PARENS` consts, and running through minifier to shake out irrelevant code
    struct VariantGen {
        variant_paths: Vec<String>,
    }

    impl VariantGenerator<5> for VariantGen {
        const FLAG_NAMES: [&str; 5] = ["IS_TS", "RANGE", "LOC", "PARENT", "PRESERVE_PARENS"];

        fn variants(&mut self) -> Vec<[bool; 5]> {
            let mut variants = Vec::with_capacity(9);

            for is_ts in [false, true] {
                for range in [false, true] {
                    for parent in [false, true] {
                        self.variant_paths.push(format!(
                            "{NAPI_PARSER_PACKAGE_PATH}/generated/deserialize/{}{}{}.js",
                            if is_ts { "ts" } else { "js" },
                            if range { "_range" } else { "" },
                            if parent { "_parent" } else { "" },
                        ));

                        variants.push([
                            is_ts, range, /* loc */ false, parent,
                            /* preserve_parens */ true,
                        ]);
                    }
                }
            }

            self.variant_paths.push(format!("{OXLINT_APP_PATH}/src-js/generated/deserialize.js"));
            variants.push([
                /* is_ts */ true, /* range */ true, /* loc */ true,
                /* parent */ true, /* preserve_parens */ false,
            ]);

            variants
        }

        fn pre_process_variant<'a>(
            &mut self,
            program: &mut Program<'a>,
            flags: [bool; 5],
            allocator: &'a Allocator,
        ) {
            if flags[2] {
                // `loc` enabled
                LocFieldAdder::new(allocator).visit_program(program);
            }
        }
    }

    let mut generator = VariantGen { variant_paths: vec![] };
    let codes = generator.generate(&code);

    generator.variant_paths.into_iter().zip(codes).collect()
}

/// Type of deserializer in which some code appears.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DeserializerType {
    Both,
    JsOnly,
    TsOnly,
}

/// Generate deserialize function for a struct.
fn generate_struct(
    struct_def: &StructDef,
    code: &mut String,
    span_type_id: TypeId,
    estree_derive_id: DeriveId,
    schema: &Schema,
) {
    if !struct_def.generates_derive(estree_derive_id) || struct_def.estree.skip {
        return;
    }

    let fn_name = struct_def.deser_name(schema);
    let mut generator = StructDeserializerGenerator::new(span_type_id, schema);

    let body = struct_def.estree.via.as_deref().and_then(|converter_name| {
        let converter = schema.meta_by_name(converter_name);
        generator.apply_converter(converter, struct_def, 0).map(|value| {
            if generator.preamble.is_empty() {
                format!("return {value};")
            } else {
                let preamble = generator.preamble.join("");
                format!(
                    "
                        {preamble}
                        return {value};
                    "
                )
            }
        })
    });

    let body = body.unwrap_or_else(|| {
        let mut inline_preamble_str = String::new();
        let mut fields_str = String::new();
        let mut assignments_preamble_str = String::new();
        let mut assignments_str = String::new();

        generator.generate_struct_fields(struct_def, 0, DeserializerType::Both);

        let has_type_field = generator.fields.contains_key("type");

        let mut all_fields_inline = true;
        for (field_name, StructFieldValue { value, deser_type, inline }) in generator.fields {
            if let Some(value) = value.strip_prefix("...") {
                assert!(inline, "Spread fields must be inlined");
                match deser_type {
                    DeserializerType::Both => write_it!(fields_str, "...{value},"),
                    DeserializerType::JsOnly => write_it!(fields_str, "...(!IS_TS && {value}),"),
                    DeserializerType::TsOnly => write_it!(fields_str, "...(IS_TS && {value}),"),
                }
            } else if inline || !has_type_field {
                let value = if generator.dependent_field_names.contains(&field_name) {
                    write_it!(inline_preamble_str, "const {field_name} = {value};\n");
                    &field_name
                } else {
                    &value
                };

                if deser_type == DeserializerType::Both {
                    write_it!(fields_str, "{field_name}: {value},");
                } else {
                    let condition = match deser_type {
                        DeserializerType::JsOnly => "!IS_TS",
                        DeserializerType::TsOnly => "IS_TS",
                        DeserializerType::Both => unreachable!(),
                    };
                    write_it!(fields_str, "...({condition} && {{ {field_name}: {value} }}),");
                }
            } else {
                all_fields_inline = false;

                let value = if generator.dependent_field_names.contains(&field_name) {
                    write_it!(assignments_preamble_str, "const {field_name} = {value};\n");
                    &field_name
                } else {
                    &value
                };

                if deser_type == DeserializerType::Both {
                    write_it!(fields_str, "{field_name}: null,");
                    write_it!(assignments_str, "node.{field_name} = {value};");
                } else {
                    let condition = match deser_type {
                        DeserializerType::JsOnly => "!IS_TS",
                        DeserializerType::TsOnly => "IS_TS",
                        DeserializerType::Both => unreachable!(),
                    };
                    write_it!(fields_str, "...({condition} && {{ {field_name}: null }}),");
                    write_it!(assignments_str, "if ({condition}) node.{field_name} = {value};");
                }
            }
        }

        for preamble_part in generator.preamble {
            assignments_preamble_str.push_str(preamble_part.trim());
        }

        let mut parent_assignment_str = "";
        if has_type_field {
            fields_str.push_str("...(PARENT && { parent }),\n");

            if !all_fields_inline {
                inline_preamble_str.push_str("const previousParent = parent;\n");
                parent_assignment_str = "parent = ";
                assignments_str.push_str("if (PARENT) parent = previousParent;\n");
            }
        }

        format!(
            "
            {inline_preamble_str}
            const node = {parent_assignment_str} {{
                {fields_str}
            }};
            {assignments_preamble_str}
            {assignments_str}
            return node;
        "
        )
    });

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            {body}
        }}
    ");
}

struct StructDeserializerGenerator<'s> {
    /// Dependencies
    dependent_field_names: FxHashSet<String>,
    /// Preamble
    preamble: Vec<String>,
    /// Fields, keyed by fields name (field name in ESTree AST)
    fields: FxIndexMap<String, StructFieldValue>,
    /// `TypeId` for `Span`
    span_type_id: TypeId,
    /// Schema
    schema: &'s Schema,
}

struct StructFieldValue {
    /// Value of the field
    value: String,
    /// Which deserializer(s) should include this field
    deser_type: DeserializerType,
    /// `true` if value can be inlined in object definition
    inline: bool,
}

impl<'s> StructDeserializerGenerator<'s> {
    fn new(span_type_id: TypeId, schema: &'s Schema) -> Self {
        Self {
            dependent_field_names: FxHashSet::default(),
            preamble: vec![],
            fields: FxIndexMap::default(),
            span_type_id,
            schema,
        }
    }

    fn generate_struct_fields(
        &mut self,
        struct_def: &StructDef,
        struct_offset: u32,
        deser_type: DeserializerType,
    ) {
        for &field_index in &struct_def.estree.field_indices {
            let field_index = field_index as usize;
            if let Some(field) = struct_def.fields.get(field_index) {
                self.generate_struct_field_owned(field, struct_def, struct_offset, deser_type);
            } else {
                let (field_name, converter_name) =
                    &struct_def.estree.add_fields[field_index - struct_def.fields.len()];
                self.generate_struct_field_added(
                    struct_def,
                    field_name,
                    converter_name,
                    struct_offset,
                    deser_type,
                );
            }
        }

        // Add `type` field if there isn't one already, and struct isn't marked `#[estree(no_type)]`
        if !struct_def.estree.no_type && !self.fields.contains_key("type") {
            let struct_name =
                struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
            self.fields.insert_before(
                0,
                "type".to_string(),
                StructFieldValue {
                    value: format!("'{struct_name}'"),
                    deser_type: DeserializerType::Both,
                    inline: true,
                },
            );
        }
    }

    fn generate_struct_field_owned(
        &mut self,
        field: &FieldDef,
        struct_def: &StructDef,
        struct_offset: u32,
        mut deser_type: DeserializerType,
    ) {
        if should_skip_field(field, self.schema) {
            return;
        }

        if field.estree.is_js {
            deser_type = DeserializerType::JsOnly;
        } else if field.estree.is_ts {
            deser_type = DeserializerType::TsOnly;
        }

        let field_name = get_struct_field_name(field).to_string();
        let field_type_id = field.type_id;
        let field_type = &self.schema.types[field_type_id];
        let field_offset = struct_offset + field.offset_64();

        if should_flatten_field(field, self.schema) {
            let TypeDef::Struct(field_struct_def) = field_type else {
                panic!(
                    "Cannot flatten a field which is not a struct: `{}::{}`",
                    struct_def.name(),
                    field.name(),
                );
            };

            self.generate_struct_fields(field_struct_def, field_offset, deser_type);

            if field_type_id == self.span_type_id {
                self.fields.insert(
                    "range".to_string(),
                    StructFieldValue {
                        value: "...(RANGE && { range: [start, end] })".to_string(),
                        deser_type,
                        inline: true,
                    },
                );

                self.dependent_field_names.extend(["start".to_string(), "end".to_string()]);
            }

            return;
        }

        // Get fields to concatenate
        // (if fields marked `#[estree(prepend_to)]` or `#[estree(append_to)]` targeting this field)
        let mut concat_fields = [field; 3];
        let mut concat_field_count = 1;
        if let Some(prepend_field_index) = field.estree.prepend_field_index {
            concat_fields[0] = &struct_def.fields[prepend_field_index];
            concat_field_count = 2;
        }
        if let Some(append_field_index) = field.estree.append_field_index {
            concat_fields[concat_field_count] = &struct_def.fields[append_field_index];
            concat_field_count += 1;
        }

        let mut inline = false;
        let value = if concat_field_count > 1 {
            // Concatenate fields
            for (index, &field) in concat_fields[..concat_field_count].iter().enumerate() {
                let field_pos = pos_offset(struct_offset + field.offset_64());
                match field.type_def(self.schema) {
                    TypeDef::Vec(vec_def) => {
                        let field_fn = vec_def.deser_name(self.schema);
                        if index == 0 {
                            self.preamble
                                .push(format!("const {field_name} = {field_fn}({field_pos});"));
                        } else {
                            self.preamble
                                .push(format!("{field_name}.push(...{field_fn}({field_pos}));"));
                        }
                    }
                    TypeDef::Option(option_def) => {
                        let option_field_name = get_struct_field_name(field).to_string();
                        let field_fn = option_def.deser_name(self.schema);
                        self.preamble
                            .push(format!("const {option_field_name} = {field_fn}({field_pos});"));
                        if index == 0 {
                            self.preamble.push(format!(
                                "const {field_name} = {option_field_name} === null ? [] : [{option_field_name}];"
                            ));
                        } else {
                            self.preamble.push(format!(
                                "if ({option_field_name} !== null) {field_name}.push({option_field_name});"
                            ));
                        }
                    }
                    _ => panic!("Cannot append: `{}::{}`", struct_def.name(), field.name()),
                }
            }

            field_name.clone()
        } else if let Some(converter_name) = &field.estree.via {
            let converter = self.schema.meta_by_name(converter_name);
            self.apply_converter(converter, struct_def, struct_offset).unwrap()
        } else {
            // Primitives and fieldless enums can be inlined into object literal,
            // because they don't have a `parent` field
            inline = match field_type.innermost_type(self.schema) {
                TypeDef::Primitive(_) => true,
                TypeDef::Enum(enum_def) => enum_def.is_fieldless(),
                _ => false,
            };

            let value_fn = field_type.deser_name(self.schema);
            let pos = pos_offset(field_offset);
            format!("{value_fn}({pos})")
        };

        self.fields.insert(field_name, StructFieldValue { value, deser_type, inline });
    }

    fn generate_struct_field_added(
        &mut self,
        struct_def: &StructDef,
        field_name: &str,
        converter_name: &str,
        struct_offset: u32,
        mut deser_type: DeserializerType,
    ) {
        let converter = self.schema.meta_by_name(converter_name);

        if converter.estree.is_js {
            deser_type = DeserializerType::JsOnly;
        } else if converter.estree.is_ts {
            deser_type = DeserializerType::TsOnly;
        }

        let value = self.apply_converter(converter, struct_def, struct_offset).unwrap();
        self.fields
            .insert(field_name.to_string(), StructFieldValue { value, deser_type, inline: false });
    }

    fn apply_converter(
        &mut self,
        converter: &MetaType,
        struct_def: &StructDef,
        struct_offset: u32,
    ) -> Option<String> {
        let raw_deser = converter.estree.raw_deser.as_deref()?;

        let value = THIS_REGEX.replace_all(raw_deser, ThisReplacer::new(self));
        let value = DESER_REGEX.replace_all(&value, DeserReplacer::new(self.schema));
        let value = POS_OFFSET_REGEX
            .replace_all(&value, PosOffsetReplacer::new(self, struct_def, struct_offset));
        let value = POS_REGEX.replace_all(&value, PosReplacer::new(struct_offset));
        let value = value.cow_replace("SOURCE_TEXT", "sourceText");

        let value = if let Some((preamble, value)) = value.trim().rsplit_once('\n') {
            self.preamble.push(preamble.to_string());
            value.trim().to_string()
        } else {
            value.trim().to_string()
        };
        Some(value)
    }
}

/// Generate deserialize function for an enum.
fn generate_enum(
    enum_def: &EnumDef,
    code: &mut String,
    estree_derive_id: DeriveId,
    schema: &Schema,
) {
    if !enum_def.generates_derive(estree_derive_id) || enum_def.estree.skip {
        return;
    }

    let type_name = enum_def.name();
    let fn_name = enum_def.deser_name(schema);
    let payload_offset = enum_def.layout_64().align;

    let body = if let Some(converter_name) = &enum_def.estree.via {
        apply_converter_for_enum(converter_name, 0, schema)
    } else {
        None
    };

    let body = body.unwrap_or_else(|| {
        let mut variants = enum_def
            .all_variants(schema)
            .filter(|variant| !should_skip_enum_variant(variant))
            .collect::<Vec<_>>();
        variants.sort_by_key(|variant| variant.discriminant);

        let mut switch_cases = String::new();
        for variant in variants {
            write_it!(switch_cases, "case {}: ", variant.discriminant);

            if let Some(converter_name) = &variant.estree.via {
                let ret = apply_converter_for_enum(converter_name, payload_offset, schema).unwrap();
                switch_cases.push_str(&ret);
            } else if let Some(variant_type) = variant.field_type(schema) {
                let variant_fn_name = variant_type.deser_name(schema);
                let payload_pos = pos_offset(payload_offset);
                write_it!(switch_cases, "return {variant_fn_name}({payload_pos});");
            } else {
                write_it!(
                    switch_cases,
                    "return '{}';",
                    get_fieldless_variant_value(enum_def, variant)
                );
            }
        }

        format!(
            "
            switch(uint8[pos]) {{
                {switch_cases}
                default: throw new Error(`Unexpected discriminant ${{uint8[pos]}} for {type_name}`);
            }}
            "
        )
    });

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            {body}
        }}
    ");
}

/// Generate deserialize function for a primitive.
fn generate_primitive(primitive_def: &PrimitiveDef, code: &mut String, schema: &Schema) {
    #[expect(clippy::match_same_arms)]
    let ret = match primitive_def.name() {
        // Reuse deserializer for `&str`
        "Atom" => return,
        // Dummy type
        "PointerAlign" => return,
        "bool" => "return uint8[pos] === 1;",
        "u8" => "return uint8[pos];",
        // "u16" => "return uint16[pos >> 1];",
        "u32" => "return uint32[pos >> 2];",
        #[rustfmt::skip]
        "u64" => "
            const pos32 = pos >> 2;
            return uint32[pos32] + uint32[pos32 + 1] * 4294967296;
        ",
        "f64" => "return float64[pos >> 3];",
        "&str" => STR_DESERIALIZER_BODY,
        // Reuse deserializers for zeroed and atomic types
        type_name if type_name.starts_with("NonZero") => return,
        type_name if type_name.starts_with("Atomic") => return,
        type_name => panic!("Cannot generate deserializer for primitive `{type_name}`"),
    };

    let fn_name = primitive_def.deser_name(schema);

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            {ret}
        }}
    ");
}

static STR_DESERIALIZER_BODY: &str = "
    const pos32 = pos >> 2,
        len = uint32[pos32 + 2];
    if (len === 0) return '';

    pos = uint32[pos32];
    if (sourceIsAscii && pos < sourceByteLen) return sourceText.substr(pos, len);

    // Longer strings use `TextDecoder`
    // TODO: Find best switch-over point
    const end = pos + len;
    if (len > 50) return decodeStr(uint8.subarray(pos, end));

    // Shorter strings decode by hand to avoid native call
    let out = '',
        c;
    do {
        c = uint8[pos++];
        if (c < 0x80) {
            out += fromCodePoint(c);
        } else {
            out += decodeStr(uint8.subarray(pos - 1, end));
            break;
        }
    } while (pos < end);

    return out;
";

/// Generate deserialize function for an `Option`.
fn generate_option(
    option_def: &OptionDef,
    code: &mut String,
    estree_derive_id: DeriveId,
    schema: &Schema,
) {
    let inner_type = option_def.inner_type(schema);
    if should_skip_innermost_type(inner_type, estree_derive_id, schema) {
        return;
    }

    let fn_name = option_def.deser_name(schema);
    let inner_fn_name = inner_type.deser_name(schema);
    let inner_layout = inner_type.layout_64();

    let (none_condition, payload_offset) = if option_def.layout_64().size == inner_layout.size {
        let niche = inner_layout.niche.clone().unwrap();
        let none_condition = match niche.size {
            1 => format!("uint8[{}] === {}", pos_offset(niche.offset), niche.value()),
            // 2 => format!("uint16[{}] === {}", pos_offset_shift(niche.offset, 1), niche.value()),
            4 => format!("uint32[{}] === {}", pos_offset_shift(niche.offset, 2), niche.value()),
            8 => {
                // TODO: Use `float64[pos >> 3] === 0` instead of
                // `uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0`?
                let value = niche.value();
                format!(
                    "uint32[{}] === {} && uint32[{}] === {}",
                    pos_offset_shift(niche.offset, 2),
                    value & u128::from(u32::MAX),
                    pos_offset_shift(niche.offset + 4, 2),
                    value >> 32,
                )
            }
            size => panic!("Invalid niche size: {size}"),
        };
        (none_condition, Cow::Borrowed("pos"))
    } else {
        ("uint8[pos] === 0".to_string(), pos_offset(inner_layout.align))
    };

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            if ({none_condition}) return null;
            return {inner_fn_name}({payload_offset});
        }}
    ");
}

/// Generate deserialize function for a `Box`.
fn generate_box(box_def: &BoxDef, code: &mut String, estree_derive_id: DeriveId, schema: &Schema) {
    let inner_type = box_def.inner_type(schema);
    if should_skip_innermost_type(inner_type, estree_derive_id, schema) {
        return;
    }

    let fn_name = box_def.deser_name(schema);
    let inner_fn_name = inner_type.deser_name(schema);

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            return {inner_fn_name}(uint32[pos >> 2]);
        }}
    ");
}

/// Generate deserialize function for a `Vec`.
fn generate_vec(vec_def: &VecDef, code: &mut String, estree_derive_id: DeriveId, schema: &Schema) {
    let inner_type = vec_def.inner_type(schema);
    if should_skip_innermost_type(inner_type, estree_derive_id, schema) {
        return;
    }

    let fn_name = vec_def.deser_name(schema);
    let inner_fn_name = inner_type.deser_name(schema);
    let inner_type_size = inner_type.layout_64().size;

    let ptr_pos32 = pos32_offset(VEC_PTR_FIELD_OFFSET);
    let len_pos32 = pos32_offset(VEC_LEN_FIELD_OFFSET);

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            const arr = [],
                pos32 = pos >> 2;
            pos = uint32[{ptr_pos32}];
            const endPos = pos + uint32[{len_pos32}] * {inner_type_size};
            while (pos !== endPos) {{
                arr.push({inner_fn_name}(pos));
                pos += {inner_type_size};
            }}
            return arr;
        }}
    ");
}

/// Check if innermost type does not require a deserializer.
pub(super) fn should_skip_innermost_type(
    type_def: &TypeDef,
    estree_derive_id: DeriveId,
    schema: &Schema,
) -> bool {
    match type_def.innermost_type(schema) {
        TypeDef::Struct(struct_def) => {
            !struct_def.generates_derive(estree_derive_id) || struct_def.estree.skip
        }
        TypeDef::Enum(enum_def) => {
            !enum_def.generates_derive(estree_derive_id) || enum_def.estree.skip
        }
        _ => false,
    }
}

/// Generate pos offset string.
///
/// * If `offset == 0` -> `pos`.
/// * Otherwise -> `pos + <offset>` (e.g. `pos + 8`).
pub(super) fn pos_offset<O>(offset: O) -> Cow<'static, str>
where
    O: TryInto<u64>,
    <O as TryInto<u64>>::Error: Debug,
{
    let offset = offset.try_into().unwrap();
    if offset == 0 { Cow::Borrowed("pos") } else { format_cow!("pos + {offset}") }
}

/// Generate pos offset and shift string.
///
/// * If `offset == 0` and `shift == 0` -> `pos`.
/// * If `offset == 0` -> `pos >> <shift>` (e.g. `pos >> 2`).
/// * If `shift == 0` -> `pos + <offset>` (e.g. `pos + 8`).
/// * Otherwise -> `(pos + <offset>) >> <shift>` (e.g. `(pos + 8) >> 2`).
pub(super) fn pos_offset_shift<O, S>(offset: O, shift: S) -> Cow<'static, str>
where
    O: TryInto<u64>,
    <O as TryInto<u64>>::Error: Debug,
    S: TryInto<u64>,
    <S as TryInto<u64>>::Error: Debug,
{
    let offset = offset.try_into().unwrap();
    let shift = shift.try_into().unwrap();
    match (offset, shift) {
        (0, 0) => Cow::Borrowed("pos"),
        (0, _) => format_cow!("pos >> {shift}"),
        (_, 0) => format_cow!("pos + {offset}"),
        (_, _) => format_cow!("(pos + {offset}) >> {shift}"),
    }
}

/// Generate pos32 offset string.
///
/// * If `offset == 0` -> `pos32`.
/// * Otherwise -> `pos32 + <offset>` (e.g. `pos32 + 4`).
pub(super) fn pos32_offset<O>(offset: O) -> Cow<'static, str>
where
    O: TryInto<u64>,
    <O as TryInto<u64>>::Error: Debug,
{
    let offset = offset.try_into().unwrap();
    let offset32 = offset >> 2;
    if offset32 == 0 { Cow::Borrowed("pos32") } else { format_cow!("pos32 + {offset32}") }
}

// `raw_deser` replacements

/// Get `raw_deser` for converter and replace `POS`, `DESER` and `SOURCE_TEXT` within it.
///
/// Returns `None` if converter is not annotated `#[estree(raw_deser = "...")]`.
fn apply_converter_for_enum(converter_name: &str, offset: u32, schema: &Schema) -> Option<String> {
    let converter = schema.meta_by_name(converter_name);
    let raw_deser = converter.estree.raw_deser.as_deref()?;

    let value = POS_REGEX.replace_all(raw_deser, PosReplacer::new(offset));
    let value = DESER_REGEX.replace_all(&value, DeserReplacer::new(schema));
    let value = value.cow_replace("SOURCE_TEXT", "sourceText");
    let value = if let Some((preamble, value)) = value.trim().rsplit_once('\n') {
        format!("{preamble} return {value};")
    } else {
        format!("return {value};")
    };

    Some(value)
}

static THIS_REGEX: Lazy<Regex> = lazy_regex!(r"THIS\.([a-zA-Z_]+)");

struct ThisReplacer<'d> {
    dependent_field_names: &'d mut FxHashSet<String>,
}

impl<'d> ThisReplacer<'d> {
    fn new(generator: &'d mut StructDeserializerGenerator<'_>) -> Self {
        Self { dependent_field_names: &mut generator.dependent_field_names }
    }
}

impl Replacer for ThisReplacer<'_> {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        assert_eq!(caps.len(), 2);
        let field_name = caps.get(1).unwrap().as_str();
        dst.push_str(field_name);
        self.dependent_field_names.insert(field_name.to_string());
    }
}

static DESER_REGEX: Lazy<Regex> = lazy_regex!(r"DESER\[([A-Za-z0-9<>_]+)\]");

struct DeserReplacer<'s> {
    schema: &'s Schema,
}

impl<'s> DeserReplacer<'s> {
    fn new(schema: &'s Schema) -> Self {
        Self { schema }
    }
}

impl Replacer for DeserReplacer<'_> {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        assert_eq!(caps.len(), 2);
        let type_name = caps.get(1).unwrap().as_str();
        let type_def = self.schema.type_by_name(type_name);
        let fn_name = type_def.deser_name(self.schema);
        dst.push_str(&fn_name);
    }
}

#[expect(clippy::trivial_regex)]
static POS_REGEX: Lazy<Regex> = lazy_regex!("POS");

struct PosReplacer {
    offset: u32,
}

impl PosReplacer {
    fn new(offset: u32) -> Self {
        Self { offset }
    }
}

impl Replacer for PosReplacer {
    fn replace_append(&mut self, _caps: &Captures, dst: &mut String) {
        dst.push_str(&pos_offset(self.offset));
    }
}

static POS_OFFSET_REGEX: Lazy<Regex> =
    lazy_regex!(r"POS_OFFSET(?:<([A-Za-z]+)>)?\.([a-zA-Z_]+(?:\.[a-zA-Z_]+)*)(?:\s*\+\s*(\d+))?");

struct PosOffsetReplacer<'s, 'd> {
    schema: &'s Schema,
    struct_def: &'d StructDef,
    struct_offset: u32,
}

impl<'s, 'd> PosOffsetReplacer<'s, 'd> {
    fn new(
        generator: &StructDeserializerGenerator<'s>,
        struct_def: &'d StructDef,
        struct_offset: u32,
    ) -> Self {
        Self { schema: generator.schema, struct_def, struct_offset }
    }
}

impl Replacer for PosOffsetReplacer<'_, '_> {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        assert_eq!(caps.len(), 4);

        let struct_def = if let Some(struct_name) = caps.get(1) {
            self.schema.type_by_name(struct_name.as_str()).as_struct().unwrap()
        } else {
            self.struct_def
        };

        let mut field_names = caps.get(2).unwrap().as_str().split('.');
        let field_name = field_names.next().unwrap();
        let field = struct_def.field_by_name(field_name);
        let mut offset = self.struct_offset + field.offset_64();
        let mut type_def = field.type_def(self.schema);
        for field_name in field_names {
            let struct_def = type_def.as_struct().unwrap();
            let field = struct_def.field_by_name(field_name);
            offset += field.offset_64();
            type_def = field.type_def(self.schema);
        }

        if let Some(add) = caps.get(3) {
            offset += str::parse::<u32>(add.as_str()).unwrap();
        }

        if offset == 0 {
            write_it!(dst, "pos");
        } else {
            write_it!(dst, "pos + {offset}");
        }
    }
}

/// Trait to get deserializer function name for a type.
pub(super) trait DeserializeFunctionName {
    fn deser_name(&self, schema: &Schema) -> String {
        format!("deserialize{}", self.plain_name(schema))
    }

    fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str>;
}

impl DeserializeFunctionName for TypeDef {
    fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str> {
        match self {
            TypeDef::Struct(def) => def.plain_name(schema),
            TypeDef::Enum(def) => def.plain_name(schema),
            TypeDef::Primitive(def) => def.plain_name(schema),
            TypeDef::Option(def) => def.plain_name(schema),
            TypeDef::Box(def) => def.plain_name(schema),
            TypeDef::Vec(def) => def.plain_name(schema),
            TypeDef::Cell(def) => def.plain_name(schema),
            TypeDef::Pointer(def) => def.plain_name(schema),
        }
    }
}

macro_rules! impl_deser_name_simple {
    ($ty:ident) => {
        impl DeserializeFunctionName for $ty {
            fn plain_name<'s>(&'s self, _schema: &'s Schema) -> Cow<'s, str> {
                Cow::Borrowed(self.name())
            }
        }
    };
}

impl_deser_name_simple!(StructDef);
impl_deser_name_simple!(EnumDef);

macro_rules! impl_deser_name_concat {
    ($ty:ident, $prefix:expr) => {
        impl DeserializeFunctionName for $ty {
            fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str> {
                format_cow!("{}{}", $prefix, self.inner_type(schema).plain_name(schema))
            }
        }
    };
}

impl_deser_name_concat!(OptionDef, "Option");
impl_deser_name_concat!(BoxDef, "Box");
impl_deser_name_concat!(VecDef, "Vec");

impl DeserializeFunctionName for PrimitiveDef {
    fn plain_name<'s>(&'s self, _schema: &'s Schema) -> Cow<'s, str> {
        let type_name = self.name();
        if matches!(type_name, "&str" | "Atom") {
            // Use 1 deserializer for both `&str` and `Atom`
            Cow::Borrowed("Str")
        } else if let Some(type_name) = type_name.strip_prefix("NonZero") {
            // Use zeroed type's deserializer for `NonZero*` types
            Cow::Borrowed(type_name)
        } else if let Some(type_name) = type_name.strip_prefix("Atomic") {
            // Use standard type's deserializer for `Atomic*` types
            Cow::Borrowed(type_name)
        } else {
            upper_case_first(type_name)
        }
    }
}

impl DeserializeFunctionName for CellDef {
    fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str> {
        // `Cell`s use same deserializer as inner type, as layout is identical
        self.inner_type(schema).plain_name(schema)
    }
}

impl DeserializeFunctionName for PointerDef {
    fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str> {
        // Pointers use same deserializer as `Box`, as layout is identical
        format_cow!("Box{}", self.inner_type(schema).plain_name(schema))
    }
}

/// Constants for position of fields in buffer which deserialization starts from.
#[derive(Clone, Copy)]
struct Constants {
    /// Size of buffer in bytes
    buffer_size: u32,
    /// Offset within buffer of `u32` containing position of `RawTransferData`
    data_pointer_pos: u32,
    /// Offset within buffer of `bool` indicating if AST is TS or JS
    is_ts_pos: u32,
    /// Offset of `Program` in buffer, relative to position of `RawTransferData`
    program_offset: u32,
    /// Offset of `u32` source text length, relative to position of `Program`
    source_len_offset: u32,
    /// Size of `RawTransferData` in bytes
    raw_metadata_size: u32,
}

/// Generate constants file.
fn generate_constants(consts: Constants) -> (String, TokenStream) {
    let Constants {
        buffer_size,
        data_pointer_pos,
        is_ts_pos,
        program_offset,
        source_len_offset,
        raw_metadata_size,
    } = consts;

    let data_pointer_pos_32 = data_pointer_pos / 4;

    #[rustfmt::skip]
    let js_output = format!("
        export const BUFFER_SIZE = {buffer_size};
        export const BUFFER_ALIGN = {BLOCK_ALIGN};
        export const DATA_POINTER_POS_32 = {data_pointer_pos_32};
        export const IS_TS_FLAG_POS = {is_ts_pos};
        export const PROGRAM_OFFSET = {program_offset};
        export const SOURCE_LEN_OFFSET = {source_len_offset};
    ");

    let block_size = number_lit(BLOCK_SIZE);
    let block_align = number_lit(BLOCK_ALIGN);
    let buffer_size = number_lit(buffer_size);
    let raw_metadata_size = number_lit(raw_metadata_size);
    let rust_output = quote! {
        #![expect(clippy::unreadable_literal)]
        #![allow(dead_code)]

        ///@@line_break
        pub const BLOCK_SIZE: usize = #block_size;
        pub const BLOCK_ALIGN: usize = #block_align;
        pub const BUFFER_SIZE: usize = #buffer_size;
        pub const RAW_METADATA_SIZE: usize = #raw_metadata_size;
    };

    (js_output, rust_output)
}

/// Calculate constants.
fn get_constants(schema: &Schema) -> Constants {
    let raw_metadata_struct = schema.type_by_name("RawTransferMetadata").as_struct().unwrap();
    let raw_metadata2_struct = schema.type_by_name("RawTransferMetadata2").as_struct().unwrap();

    // Check layout and fields of `RawTransferMetadata` and `RawTransferMetadata2` are identical
    assert_eq!(raw_metadata_struct.layout, raw_metadata2_struct.layout);
    assert_eq!(raw_metadata_struct.fields.len(), raw_metadata2_struct.fields.len());

    let mut data_offset_field = None;
    let mut is_ts_field = None;
    for (field1, field2) in raw_metadata_struct.fields.iter().zip(&raw_metadata2_struct.fields) {
        assert_eq!(field1.name(), field2.name());
        assert_eq!(field1.type_id, field2.type_id);
        assert_eq!(field1.offset_64(), field2.offset_64());
        match field1.name() {
            "data_offset" => data_offset_field = Some(field1),
            "is_ts" => is_ts_field = Some(field1),
            _ => {}
        }
    }
    let data_offset_field = data_offset_field.unwrap();
    let is_ts_field = is_ts_field.unwrap();

    let raw_metadata_size = raw_metadata_struct.layout_64().size;

    // Round up to multiple of `ALLOCATOR_CHUNK_END_ALIGN`
    let fixed_metadata_struct =
        schema.type_by_name("FixedSizeAllocatorMetadata").as_struct().unwrap();
    let fixed_metadata_size =
        fixed_metadata_struct.layout_64().size.next_multiple_of(ALLOCATOR_CHUNK_END_ALIGN);

    let buffer_size = BLOCK_SIZE - fixed_metadata_size;

    // Get offsets of data within buffer
    let raw_metadata_pos = buffer_size - raw_metadata_size;
    let data_pointer_pos = raw_metadata_pos + data_offset_field.offset_64();
    let is_ts_pos = raw_metadata_pos + is_ts_field.offset_64();

    let program_offset = schema
        .type_by_name("RawTransferData")
        .as_struct()
        .unwrap()
        .field_by_name("program")
        .offset_64();

    let source_len_offset = schema
        .type_by_name("Program")
        .as_struct()
        .unwrap()
        .field_by_name("source_text")
        .offset_64()
        + STR_LEN_OFFSET;

    Constants {
        buffer_size,
        data_pointer_pos,
        is_ts_pos,
        program_offset,
        source_len_offset,
        raw_metadata_size,
    }
}

/// Visitor to add `__proto__` field to all objects with `range` field in all deserialize functions,
/// to give access to `loc`.
///
/// Works on AST pre-minification.
struct LocFieldAdder<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> LocFieldAdder<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self { ast: AstBuilder::new(allocator) }
    }
}

impl<'a> VisitMut<'a> for LocFieldAdder<'a> {
    fn visit_object_expression(&mut self, obj_expr: &mut ObjectExpression<'a>) {
        // Check if object has `range` field (`...(RANGE && { range: [start, end] })`)
        let has_range_field = obj_expr.properties.iter().any(|prop| {
            if let ObjectPropertyKind::SpreadProperty(spread) = prop
                && let Expression::ParenthesizedExpression(paren_expr) = &spread.argument
                && let Expression::LogicalExpression(logical_expr) = &paren_expr.expression
                && logical_expr.operator == LogicalOperator::And
                && let Expression::Identifier(ident) = &logical_expr.left
                && ident.name == "RANGE"
            {
                true
            } else {
                false
            }
        });
        if !has_range_field {
            return;
        }

        // Insert `__proto__: NodeProto` as first field
        let prop = self.ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            self.ast.property_key_static_identifier(SPAN, "__proto__"),
            self.ast.expression_identifier(SPAN, "NodeProto"),
            false,
            false,
            false,
        );
        obj_expr.properties.insert(0, prop);
    }
}
