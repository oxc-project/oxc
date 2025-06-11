//! Generator for raw transfer lazy deserializer.

#![expect(unused_imports)]

use std::{borrow::Cow, fmt::Debug, str};

use crate::{
    Generator, NAPI_PARSER_PACKAGE_PATH,
    codegen::{Codegen, DeriveId},
    derives::estree::{
        get_fieldless_variant_value, get_struct_field_name, should_skip_enum_variant,
        should_skip_field,
    },
    output::Output,
    schema::{
        BoxDef, CellDef, Def, EnumDef, FieldDef, MetaType, OptionDef, PrimitiveDef, Schema,
        StructDef, TypeDef, VecDef,
        extensions::layout::{GetLayout, GetOffset},
    },
    utils::{format_cow, upper_case_first, write_it},
};

use super::define_generator;
use super::raw_transfer::{
    VEC_LEN_FIELD_OFFSET, VEC_PTR_FIELD_OFFSET, pos_offset, pos_offset_shift, pos32_offset,
    should_skip_innermost_type,
};

/// Generator for raw transfer lazy deserializer.
pub struct RawTransferLazyGenerator;

define_generator!(RawTransferLazyGenerator);

impl Generator for RawTransferLazyGenerator {
    fn generate(&self, schema: &Schema, codegen: &Codegen) -> Output {
        let code = generate_constructors(schema, codegen);
        Output::Javascript {
            path: format!("{NAPI_PARSER_PACKAGE_PATH}/generated/deserialize/lazy.js"),
            code,
        }
    }
}

/// Prelude to generated deserializer.
/// Defines the main `construct` function.
static PRELUDE: &str = "
    'use strict';

    // Unique token which is not exposed publicly.
    // Used to prevent user calling class constructors.
    const TOKEN = {};

    module.exports = { construct, TOKEN };

    function construct(ast) {
        // (2 * 1024 * 1024 * 1024 - 16) >> 2
        const metadataPos32 = 536870908;

        return new RawTransferData(ast.buffer.uint32[metadataPos32], ast, TOKEN);
    }

    const textDecoder = new TextDecoder('utf-8', { ignoreBOM: true }),
        decodeStr = textDecoder.decode.bind(textDecoder),
        { fromCodePoint } = String;

";

/// Generate constructor functions for all types.
fn generate_constructors(schema: &Schema, codegen: &Codegen) -> String {
    let estree_derive_id = codegen.get_derive_id_by_name("ESTree");

    let mut code = PRELUDE.to_string();

    for type_def in &schema.types {
        match type_def {
            TypeDef::Struct(struct_def) => {
                generate_struct(struct_def, &mut code, estree_derive_id, schema);
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
                // No constructor for `Cell`s - use inner type's constructor
            }
        }
    }

    code
}

/// Generate class for a struct.
fn generate_struct(
    struct_def: &StructDef,
    code: &mut String,
    estree_derive_id: DeriveId,
    schema: &Schema,
) {
    if !struct_def.generates_derive(estree_derive_id) || struct_def.estree.skip {
        return;
    }

    let struct_name = struct_def.name();

    let mut getters = String::new();
    let mut to_json = String::new();
    let mut extra_props = String::new();

    let mut add_type_field = !struct_def.estree.no_type;
    for field in &struct_def.fields {
        if should_skip_field(field, schema) {
            continue;
        }

        let field_name = get_struct_field_name(field);
        if field_name == "type" {
            add_type_field = false;
        }

        let field_type = field.type_def(schema);
        let needs_cached_prop = match field_type {
            TypeDef::Vec(_) => true,
            TypeDef::Primitive(primitive_def) => {
                matches!(primitive_def.name(), "&str" | "Atom")
            }
            TypeDef::Option(option_def) => {
                if let TypeDef::Primitive(primitive_def) = option_def.inner_type(schema) {
                    matches!(primitive_def.name(), "&str" | "Atom")
                } else {
                    false
                }
            }
            TypeDef::Struct(struct_def) => {
                // TODO: Don't hard-code this
                if field_name == "span" && struct_def.name() == "Span" {
                    for span_field in &struct_def.fields {
                        if span_field.name() == "_align" {
                            continue;
                        }

                        let span_field_name = get_struct_field_name(span_field);
                        let value_fn = span_field.type_def(schema).constructor_name(schema);
                        let pos = internal_pos_offset(field.offset_64() + span_field.offset_64());

                        #[rustfmt::skip]
                        write_it!(getters, "
                            get {span_field_name}() {{
                                const internal = this.#internal;
                                return {value_fn}({pos}, internal.$ast);
                            }}
                        ");

                        write_it!(to_json, "{span_field_name}: this.{span_field_name},\n");
                    }
                    continue;
                }

                false
            }
            _ => false,
        };

        let value_fn = field_type.constructor_name(schema);
        let pos = internal_pos_offset(field.offset_64());

        if needs_cached_prop {
            write_it!(extra_props, ", {field_name}: void 0");

            #[rustfmt::skip]
            write_it!(getters, "
                get {field_name}() {{
                    const internal = this.#internal,
                        node = internal.{field_name};
                    if (node !== void 0) return node;
                    return internal.{field_name} = {value_fn}({pos}, internal.$ast);
                }}
            ");
        } else {
            #[rustfmt::skip]
            write_it!(getters, "
                get {field_name}() {{
                    const internal = this.#internal;
                    return {value_fn}({pos}, internal.$ast);
                }}
            ");
        }

        // Remove this special case for `RegExpFlags`
        if struct_name != "RegExpFlags" {
            write_it!(to_json, "{field_name}: this.{field_name},\n");
        }
    }

    let type_prop_init = if add_type_field {
        to_json = format!("type: '{struct_name}',\n{to_json}");
        format!("type = '{struct_name}';")
    } else {
        String::new()
    };

    // TODO: Nodes cache does not work because some types have same `pos` as their parent
    /*
    constructor(pos, ast) {{
        if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');

        const {{ nodes }} = ast;
        let node = nodes.get(pos);
        if (node !== void 0) return node;

        this.#internal = {{ $pos: pos, $ast: ast {extra_props} }};
        nodes.set(pos, this);
    }}
    */

    #[rustfmt::skip]
    write_it!(code, "
        class {struct_name} {{
            {type_prop_init}
            #internal;

            constructor(pos, ast) {{
                if (ast.token !== TOKEN) throw new Error('Constructor is for internal use only');
                this.#internal = {{ $pos: pos, $ast: ast {extra_props} }};
            }}

            {getters}

            toJSON() {{
                return {{
                    {to_json}
                }};
            }}
        }}
    ");
}

/// Generate constructor function for an enum.
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
    let fn_name = enum_def.constructor_name(schema);
    let payload_offset = enum_def.layout_64().align;

    let mut variants = enum_def
        .all_variants(schema)
        .filter(|variant| !should_skip_enum_variant(variant))
        .collect::<Vec<_>>();
    variants.sort_by_key(|variant| variant.discriminant);

    let mut switch_cases = String::new();
    for variant in variants {
        write_it!(switch_cases, "case {}: ", variant.discriminant);

        if let Some(variant_type) = variant.field_type(schema) {
            let variant_fn_name = variant_type.constructor_name(schema);
            let payload_pos = pos_offset(payload_offset);
            write_it!(switch_cases, "return {variant_fn_name}({payload_pos}, ast);");
        } else {
            write_it!(switch_cases, "return '{}';", get_fieldless_variant_value(enum_def, variant));
        }
    }

    let body = format!(
        "
        switch(ast.buffer[pos]) {{
            {switch_cases}
            default: throw new Error(`Unexpected discriminant ${{ast.buffer[pos]}} for {type_name}`);
        }}
        "
    );

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos, ast) {{
            {body}
        }}
    ");
}

/// Generate constructor function for a primitive.
fn generate_primitive(primitive_def: &PrimitiveDef, code: &mut String, schema: &Schema) {
    #[expect(clippy::match_same_arms)]
    let ret = match primitive_def.name() {
        // Reuse constructor for `&str`
        "Atom" => return,
        // Dummy type
        "PointerAlign" => return,
        "bool" => "return ast.buffer[pos] === 1;",
        "u8" => "return ast.buffer[pos];",
        // "u16" => "return uint16[pos >> 1];",
        "u32" => "return ast.buffer.uint32[pos >> 2];",
        #[rustfmt::skip]
        "u64" => "
            const { uint32 } = ast.buffer,
                pos32 = pos >> 2;
            return uint32[pos32] + uint32[pos32 + 1] * 4294967296;
        ",
        "f64" => "return ast.buffer.float64[pos >> 3];",
        "&str" => STR_DESERIALIZER_BODY,
        // Reuse constructors for zeroed types
        type_name if type_name.starts_with("NonZero") => return,
        type_name => panic!("Cannot generate constructor for primitive `{type_name}`"),
    };

    let fn_name = primitive_def.constructor_name(schema);

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos, ast) {{
            {ret}
        }}
    ");
}

static STR_DESERIALIZER_BODY: &str = "
    const pos32 = pos >> 2,
        { buffer } = ast,
        { uint32 } = buffer,
        len = uint32[pos32 + 2];
    if (len === 0) return '';

    pos = uint32[pos32];
    if (ast.sourceIsAscii && pos < ast.sourceLen) return ast.sourceText.substr(pos, len);

    // Longer strings use `TextDecoder`
    // TODO: Find best switch-over point
    const end = pos + len;
    if (len > 50) return decodeStr(buffer.subarray(pos, end));

    // Shorter strings decode by hand to avoid native call
    let out = '',
        c;
    do {
        c = buffer[pos++];
        if (c < 0x80) {
            out += fromCodePoint(c);
        } else {
            out += decodeStr(buffer.subarray(pos - 1, end));
            break;
        }
    } while (pos < end);

    return out;
";

/// Generate constructor function for an `Option`.
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

    let fn_name = option_def.constructor_name(schema);
    let inner_fn_name = inner_type.constructor_name(schema);
    let inner_layout = inner_type.layout_64();

    let (none_condition, payload_offset) = if option_def.layout_64().size == inner_layout.size {
        let niche = inner_layout.niche.clone().unwrap();
        let none_condition = match niche.size {
            1 => format!("ast.buffer[{}] === {}", pos_offset(niche.offset), niche.value()),
            // 2 => format!("ast.buffer.uint16[{}] === {}", pos_offset_shift(niche.offset, 1), niche.value()),
            4 => format!(
                "ast.buffer.uint32[{}] === {}",
                pos_offset_shift(niche.offset, 2),
                niche.value()
            ),
            8 => {
                // TODO: Use `float64[pos >> 3] === 0` instead of
                // `uint32[pos >> 2] === 0 && uint32[(pos + 4) >> 2] === 0`?
                let value = niche.value();
                format!(
                    "ast.buffer.uint32[{}] === {} && ast.buffer.uint32[{}] === {}",
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
        ("ast.buffer[pos] === 0".to_string(), pos_offset(inner_layout.align))
    };

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos, ast) {{
            if ({none_condition}) return null;
            return {inner_fn_name}({payload_offset}, ast);
        }}
    ");
}

/// Generate constructor function for a `Box`.
fn generate_box(box_def: &BoxDef, code: &mut String, estree_derive_id: DeriveId, schema: &Schema) {
    let inner_type = box_def.inner_type(schema);
    if should_skip_innermost_type(inner_type, estree_derive_id, schema) {
        return;
    }

    let fn_name = box_def.constructor_name(schema);
    let inner_fn_name = inner_type.constructor_name(schema);

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos, ast) {{
            return {inner_fn_name}(ast.buffer.uint32[pos >> 2], ast);
        }}
    ");
}

/// Generate constructor function for a `Vec`.
fn generate_vec(vec_def: &VecDef, code: &mut String, estree_derive_id: DeriveId, schema: &Schema) {
    let inner_type = vec_def.inner_type(schema);
    if should_skip_innermost_type(inner_type, estree_derive_id, schema) {
        return;
    }

    let fn_name = vec_def.constructor_name(schema);
    let inner_fn_name = inner_type.constructor_name(schema);
    let inner_type_size = inner_type.layout_64().size;

    let ptr_pos32 = pos32_offset(VEC_PTR_FIELD_OFFSET);
    let len_pos32 = pos32_offset(VEC_LEN_FIELD_OFFSET);

    // TODO: Wrap array in a proxy, instead of eagerly deserializing all elements

    #[rustfmt::skip]
    write_it!(code, "
        function {fn_name}(pos, ast) {{
            const {{ uint32 }} = ast.buffer,
                arr = [],
                pos32 = pos >> 2,
                len = uint32[{len_pos32}];
            pos = uint32[{ptr_pos32}];
            for (let i = 0; i < len; i++) {{
                arr.push({inner_fn_name}(pos, ast));
                pos += {inner_type_size};
            }}
            return arr;
        }}
    ");
}

/// Generate pos offset string.
///
/// * If `offset == 0` -> `internal.$pos`.
/// * Otherwise -> `internal.$pos + <offset>` (e.g. `internal.$pos + 8`).
fn internal_pos_offset<O>(offset: O) -> Cow<'static, str>
where
    O: TryInto<u64>,
    <O as TryInto<u64>>::Error: Debug,
{
    let offset = offset.try_into().unwrap();
    if offset == 0 {
        Cow::Borrowed("internal.$pos")
    } else {
        format_cow!("internal.$pos + {offset}")
    }
}

/// Trait to get constructor function name for a type.
///
/// `construct<type name>` for all types except structs, for which it's `new <type name>`.
pub(super) trait ConstructorName {
    fn constructor_name(&self, schema: &Schema) -> String {
        format!("construct{}", self.plain_name(schema))
    }

    fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str>;
}

impl ConstructorName for TypeDef {
    fn constructor_name(&self, schema: &Schema) -> String {
        match self {
            TypeDef::Struct(def) => def.constructor_name(schema),
            TypeDef::Enum(def) => def.constructor_name(schema),
            TypeDef::Primitive(def) => def.constructor_name(schema),
            TypeDef::Option(def) => def.constructor_name(schema),
            TypeDef::Box(def) => def.constructor_name(schema),
            TypeDef::Vec(def) => def.constructor_name(schema),
            TypeDef::Cell(def) => def.constructor_name(schema),
        }
    }

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

impl ConstructorName for StructDef {
    fn constructor_name(&self, _schema: &Schema) -> String {
        format!("new {}", self.name())
    }

    fn plain_name(&self, _schema: &Schema) -> Cow<'_, str> {
        Cow::Borrowed(self.name())
    }
}

impl ConstructorName for EnumDef {
    fn plain_name(&self, _schema: &Schema) -> Cow<'_, str> {
        Cow::Borrowed(self.name())
    }
}

macro_rules! impl_deser_name_concat {
    ($ty:ident, $prefix:expr) => {
        impl ConstructorName for $ty {
            fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str> {
                format_cow!("{}{}", $prefix, self.inner_type(schema).plain_name(schema))
            }
        }
    };
}

impl_deser_name_concat!(OptionDef, "Option");
impl_deser_name_concat!(BoxDef, "Box");
impl_deser_name_concat!(VecDef, "Vec");

impl ConstructorName for PrimitiveDef {
    fn plain_name<'s>(&'s self, _schema: &'s Schema) -> Cow<'s, str> {
        let type_name = self.name();
        if matches!(type_name, "&str" | "Atom") {
            // Use 1 constructor for both `&str` and `Atom`
            Cow::Borrowed("Str")
        } else if let Some(type_name) = type_name.strip_prefix("NonZero") {
            // Use zeroed type's constructor for `NonZero*` types
            Cow::Borrowed(type_name)
        } else {
            upper_case_first(type_name)
        }
    }
}

// `Cell`s use same constructor as inner type, as layout is identical
impl ConstructorName for CellDef {
    fn constructor_name<'s>(&'s self, schema: &'s Schema) -> String {
        self.inner_type(schema).constructor_name(schema)
    }

    fn plain_name<'s>(&'s self, schema: &'s Schema) -> Cow<'s, str> {
        self.inner_type(schema).plain_name(schema)
    }
}
