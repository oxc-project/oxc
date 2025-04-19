//! Generator for raw transfer deserializer.

use std::{borrow::Cow, fmt::Debug, str};

use cow_utils::CowUtils;
use lazy_regex::{Captures, Lazy, Regex, lazy_regex, regex::Replacer};
use rustc_hash::FxHashSet;

use crate::{
    Generator, NAPI_PARSER_PACKAGE_PATH,
    codegen::{Codegen, DeriveId},
    derives::estree::{
        get_fieldless_variant_value, get_struct_field_name, should_flatten_field,
        should_skip_enum_variant, should_skip_field,
    },
    output::Output,
    schema::{
        BoxDef, CellDef, Def, EnumDef, FieldDef, MetaType, OptionDef, PrimitiveDef, Schema,
        StructDef, TypeDef, VecDef,
        extensions::layout::{GetLayout, GetOffset},
    },
    utils::{FxIndexMap, format_cow, upper_case_first, write_it},
};

use super::define_generator;

// Offsets of `Vec`'s fields.
// These are based on observation, and are not stable.
// They will be fully reliable once we implement our own `Vec` type and make it `#[repr(C)]`.
const VEC_PTR_FIELD_OFFSET: usize = 0;
const VEC_LEN_FIELD_OFFSET: usize = 24;

/// Generator for raw transfer deserializer.
pub struct RawTransferGenerator;

define_generator!(RawTransferGenerator);

impl Generator for RawTransferGenerator {
    fn generate_many(&self, schema: &Schema, codegen: &Codegen) -> Vec<Output> {
        let Codes { js, ts, .. } = generate_deserializers(schema, codegen);
        vec![
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/deserialize/js.js"),
                code: js,
            },
            Output::Javascript {
                path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/deserialize/ts.js"),
                code: ts,
            },
        ]
    }
}

/// Prelude to generated deserializer.
/// Defines the main `deserialize` function.
static PRELUDE: &str = "
    'use strict';

    module.exports = deserialize;

    let uint8, uint32, float64, sourceText, sourceIsAscii, sourceLen;

    const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true }),
        decodeStr = textDecoder.decode.bind(textDecoder),
        { fromCodePoint } = String;

    function deserialize(buffer, sourceTextInput, sourceLenInput) {
        uint8 = buffer;
        uint32 = new Uint32Array(buffer.buffer, buffer.byteOffset);
        float64 = new Float64Array(buffer.buffer, buffer.byteOffset);

        sourceText = sourceTextInput;
        sourceLen = sourceLenInput;
        sourceIsAscii = sourceText.length === sourceLen;

        // (2 * 1024 * 1024 * 1024 - 16) >> 2
        const metadataPos32 = 536870908;

        const data = deserializeRawTransferData(uint32[metadataPos32]);

        uint8 = uint32 = float64 = sourceText = undefined;

        return data;
    }
";

/// Container for generated code.
struct Codes {
    /// Code which is part of JS deserializer only
    js: String,
    /// Code which is part of TS deserializer only
    ts: String,
    /// Code which is part of both deserializers
    both: String,
}

/// Generate deserializer functions for all types.
fn generate_deserializers(schema: &Schema, codegen: &Codegen) -> Codes {
    let estree_derive_id = codegen.get_derive_id_by_name("ESTree");

    let mut codes = Codes { js: PRELUDE.to_string(), ts: PRELUDE.to_string(), both: String::new() };

    for type_def in &schema.types {
        match type_def {
            TypeDef::Struct(struct_def) => {
                generate_struct(struct_def, &mut codes.js, false, estree_derive_id, schema);
                generate_struct(struct_def, &mut codes.ts, true, estree_derive_id, schema);
            }
            TypeDef::Enum(enum_def) => {
                generate_enum(enum_def, &mut codes.both, estree_derive_id, schema);
            }
            TypeDef::Primitive(primitive_def) => {
                generate_primitive(primitive_def, &mut codes.both, schema);
            }
            TypeDef::Option(option_def) => {
                generate_option(option_def, &mut codes.both, schema);
            }
            TypeDef::Box(box_def) => {
                generate_box(box_def, &mut codes.both, schema);
            }
            TypeDef::Vec(vec_def) => {
                generate_vec(vec_def, &mut codes.both, schema);
            }
            TypeDef::Cell(_cell_def) => {
                // No deserializers for `Cell`s - use inner type's deserializer
            }
        }
    }

    codes.js.push_str(&codes.both);
    codes.ts.push_str(&codes.both);
    codes
}

/// Generate deserialize function for a struct.
fn generate_struct(
    struct_def: &StructDef,
    code: &mut String,
    is_ts: bool,
    estree_derive_id: DeriveId,
    schema: &Schema,
) {
    if !struct_def.generates_derive(estree_derive_id) || struct_def.estree.skip {
        return;
    }

    let fn_name = struct_def.deser_name(schema);
    let mut generator = StructDeserializerGenerator::new(is_ts, schema);

    let body = if let Some(converter_name) = &struct_def.estree.via {
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
    } else {
        None
    };

    let body = if let Some(body) = body {
        body
    } else {
        let mut preamble_str = String::new();
        let mut fields_str = String::new();

        generator.generate_struct_fields(struct_def, 0);

        for (field_name, value) in generator.fields {
            if value.starts_with("...") {
                write_it!(fields_str, "{value},");
            } else if generator.dependent_field_names.contains(&field_name) {
                if preamble_str.is_empty() {
                    preamble_str.push_str("const ");
                } else {
                    preamble_str.push_str(",\n");
                }
                write_it!(preamble_str, "{field_name} = {value}");
                write_it!(fields_str, "{field_name},");
            } else if value == field_name {
                write_it!(fields_str, "{field_name},");
            } else {
                write_it!(fields_str, "{field_name}: {value},");
            }
        }

        if !preamble_str.is_empty() {
            preamble_str.push(';');
        }

        for preamble_part in generator.preamble {
            preamble_str.push_str(preamble_part.trim());
        }

        format!(
            "
            {preamble_str}
            return {{
                {fields_str}
            }};
        "
        )
    };

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            {body}
        }}
    ");
}

struct StructDeserializerGenerator<'s> {
    /// `true` if generating deserializer for TypeScript
    is_ts: bool,
    /// Dependencies
    dependent_field_names: FxHashSet<String>,
    /// Preamble
    preamble: Vec<String>,
    /// Fields, keyed by fields name (field name in ESTree AST)
    fields: FxIndexMap<String, String>,
    /// Schema
    schema: &'s Schema,
}

impl<'s> StructDeserializerGenerator<'s> {
    fn new(is_ts: bool, schema: &'s Schema) -> Self {
        Self {
            is_ts,
            dependent_field_names: FxHashSet::default(),
            preamble: vec![],
            fields: FxIndexMap::default(),
            schema,
        }
    }

    fn generate_struct_fields(&mut self, struct_def: &StructDef, struct_offset: u32) {
        for &field_index in &struct_def.estree.field_indices {
            let field_index = field_index as usize;
            if let Some(field) = struct_def.fields.get(field_index) {
                self.generate_struct_field_owned(field, struct_def, struct_offset);
            } else {
                let (field_name, converter_name) =
                    &struct_def.estree.add_fields[field_index - struct_def.fields.len()];
                self.generate_struct_field_added(
                    struct_def,
                    field_name,
                    converter_name,
                    struct_offset,
                );
            }
        }

        // Add `type` field if there isn't one already, and struct isn't marked `#[estree(no_type)]`
        if !struct_def.estree.no_type && !self.fields.contains_key("type") {
            let struct_name =
                struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
            self.fields.insert_before(0, "type".to_string(), format!("'{struct_name}'"));
        }
    }

    fn generate_struct_field_owned(
        &mut self,
        field: &FieldDef,
        struct_def: &StructDef,
        struct_offset: u32,
    ) {
        if !self.is_ts && field.estree.is_ts {
            return;
        }

        if should_skip_field(field, self.schema) {
            return;
        }

        let field_name = get_struct_field_name(field).to_string();
        let field_type = field.type_def(self.schema);
        let field_offset = struct_offset + field.offset_64();

        if should_flatten_field(field, self.schema) {
            match field_type {
                TypeDef::Struct(field_struct_def) => {
                    self.generate_struct_fields(field_struct_def, field_offset);
                }
                TypeDef::Enum(field_enum_def) => {
                    // TODO: Do this better
                    let value_fn = field_enum_def.deser_name(self.schema);
                    let pos = pos_offset(field_offset);
                    self.fields.insert(field_name, format!("...{value_fn}({pos})"));
                }
                _ => panic!(
                    "Cannot flatten a field which is not a struct or enum: `{}::{}`",
                    struct_def.name(),
                    field.name(),
                ),
            }
            return;
        }

        let value_fn = field_type.deser_name(self.schema);
        let pos = pos_offset(field_offset);
        let mut value = format!("{value_fn}({pos})");

        if let Some(appended_field_index) = field.estree.append_field_index {
            self.preamble.push(format!("const {field_name} = {value};"));

            let appended_field = &struct_def.fields[appended_field_index];
            let appended_field_type = appended_field.type_def(self.schema);
            match appended_field_type {
                TypeDef::Vec(vec_def) => {
                    let appended_field_fn = vec_def.deser_name(self.schema);
                    let appended_pos = pos_offset(struct_offset + appended_field.offset_64());
                    self.preamble.push(format!(
                        "{field_name}.push(...{appended_field_fn}({appended_pos}));"
                    ));
                }
                TypeDef::Option(option_def) => {
                    let appended_field_name = get_struct_field_name(appended_field).to_string();
                    let appended_field_fn = option_def.deser_name(self.schema);
                    let appended_pos = pos_offset(struct_offset + appended_field.offset_64());
                    self.preamble.push(format!("
                        const {appended_field_name} = {appended_field_fn}({appended_pos});
                        if ({appended_field_name} !== null) {field_name}.push({appended_field_name});
                    "));
                }
                _ => panic!("Cannot append: `{}::{}`", struct_def.name(), field.name()),
            }

            value.clone_from(&field_name);
        } else if let Some(converter_name) = &field.estree.via {
            let converter = self.schema.meta_by_name(converter_name);
            value = self.apply_converter(converter, struct_def, struct_offset).unwrap();
        }

        self.fields.insert(field_name, value);
    }

    fn generate_struct_field_added(
        &mut self,
        struct_def: &StructDef,
        field_name: &str,
        converter_name: &str,
        struct_offset: u32,
    ) {
        let converter = self.schema.meta_by_name(converter_name);
        if !self.is_ts && converter.estree.is_ts {
            return;
        }

        let value = self.apply_converter(converter, struct_def, struct_offset).unwrap();
        self.fields.insert(field_name.to_string(), value);
    }

    fn apply_converter(
        &mut self,
        converter: &MetaType,
        struct_def: &StructDef,
        struct_offset: u32,
    ) -> Option<String> {
        let raw_deser = converter.estree.raw_deser.as_deref()?;

        let value = IF_TS_REGEX.replace_all(raw_deser, IfTsReplacer::new(self.is_ts));
        let value = IF_JS_REGEX.replace_all(&value, IfJsReplacer::new(self.is_ts));
        let value = THIS_REGEX.replace_all(&value, ThisReplacer::new(self));
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
        // Reuse deserializers for zeroed types
        type_name if type_name.starts_with("NonZero") => return,
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
    if (sourceIsAscii && pos < sourceLen) return sourceText.substr(pos, len);

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
fn generate_option(option_def: &OptionDef, code: &mut String, schema: &Schema) {
    let fn_name = option_def.deser_name(schema);
    let inner_type = option_def.inner_type(schema);
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
fn generate_box(box_def: &BoxDef, code: &mut String, schema: &Schema) {
    let fn_name = box_def.deser_name(schema);
    let inner_fn_name = box_def.inner_type(schema).deser_name(schema);

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            return {inner_fn_name}(uint32[pos >> 2]);
        }}
    ");
}

/// Generate deserialize function for a `Vec`.
fn generate_vec(vec_def: &VecDef, code: &mut String, schema: &Schema) {
    let fn_name = vec_def.deser_name(schema);
    let inner_type = vec_def.inner_type(schema);
    let inner_fn_name = inner_type.deser_name(schema);
    let inner_type_size = inner_type.layout_64().size;

    let ptr_pos32 = pos32_offset(VEC_PTR_FIELD_OFFSET);
    let len_pos32 = pos32_offset(VEC_LEN_FIELD_OFFSET);

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos) {{
            const arr = [],
                pos32 = pos >> 2,
                len = uint32[{len_pos32}];
            pos = uint32[{ptr_pos32}];
            for (let i = 0; i < len; i++) {{
                arr.push({inner_fn_name}(pos));
                pos += {inner_type_size};
            }}
            return arr;
        }}
    ");
}

/// Generate pos offset string.
///
/// * If `offset == 0` -> `pos`.
/// * Otherwise -> `pos + <offset>` (e.g. `pos + 8`).
fn pos_offset<O>(offset: O) -> Cow<'static, str>
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
fn pos_offset_shift<O, S>(offset: O, shift: S) -> Cow<'static, str>
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
fn pos32_offset<O>(offset: O) -> Cow<'static, str>
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
        let field = struct_def.fields.iter().find(|field| field.name() == field_name).unwrap();
        let mut offset = self.struct_offset + field.offset_64();
        let mut type_def = field.type_def(self.schema);
        for field_name in field_names {
            let struct_def = type_def.as_struct().unwrap();
            let field = struct_def.fields.iter().find(|field| field.name() == field_name).unwrap();
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

static IF_TS_REGEX: Lazy<Regex> = lazy_regex!(r"/\* IF_TS \*/\s*([\s\S]*?)/\* END_IF_TS \*/\s*");

struct IfTsReplacer {
    is_ts: bool,
}

impl IfTsReplacer {
    fn new(is_ts: bool) -> Self {
        Self { is_ts }
    }
}

impl Replacer for IfTsReplacer {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        assert_eq!(caps.len(), 2);
        if self.is_ts {
            dst.push_str(caps.get(1).unwrap().as_str());
        }
    }
}

static IF_JS_REGEX: Lazy<Regex> = lazy_regex!(r"/\* IF_JS \*/\s*([\s\S]*?)/\* END_IF_JS \*/\s*");

struct IfJsReplacer {
    is_ts: bool,
}

impl IfJsReplacer {
    fn new(is_ts: bool) -> Self {
        Self { is_ts }
    }
}

impl Replacer for IfJsReplacer {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        assert_eq!(caps.len(), 2);
        if !self.is_ts {
            dst.push_str(caps.get(1).unwrap().as_str());
        }
    }
}

/// Trait to get deserializer function name for a type.
trait DeserializeFunctionName {
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
