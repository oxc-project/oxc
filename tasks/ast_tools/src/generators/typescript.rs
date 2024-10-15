use std::collections::HashMap;

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_prettier::{Prettier, PrettierOptions, TrailingComma};
use oxc_span::SourceType;

use super::define_generator;
use crate::{
    codegen::LateCtx,
    output,
    schema::{
        serialize::{enum_variant_name, get_type_tag},
        EnumDef, GetIdent, StructDef, TypeDef, TypeName,
    },
    Generator, GeneratorOutput,
};

define_generator! {
    pub struct TypescriptGenerator;
}

impl Generator for TypescriptGenerator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let file = file!().replace('\\', "/");
        let mut contents = format!(
            "\
						// To edit this generated file you have to edit `{file}`\n\
						// Auto-generated code, DO NOT EDIT DIRECTLY!\n\n"
        );

        let type_map: HashMap<String, String> = ctx
            .schema()
            .into_iter()
            .filter_map(|def| {
                let TypeDef::Struct(def) = def else {
                    return None;
                };
                let Some(type_tag) = get_type_tag(def) else {
                    return None;
                };
                Some((def.ident().to_string(), type_tag))
            })
            .collect();

        for def in ctx.schema() {
            let type_def = match def {
                TypeDef::Struct(it) => generate_struct(it, &type_map),
                TypeDef::Enum(it) => generate_enum(it, &type_map),
            };

            let ident = def.ident();
            contents.push_str(&format!("export type {ident} = {type_def}\n\n",));
        }

        GeneratorOutput::Raw(output(crate::TYPESCRIPT_PACKAGE, "types.d.ts"), contents)
    }
}

fn generate_enum(def: &EnumDef, type_map: &HashMap<String, String>) -> String {
    if def.markers.estree.untagged {
        def.all_variants()
            .map(|var| {
                let ident = var.ident().to_string();
                type_map.get(&ident).map_or_else(|| ident, |v| v.to_string())
            })
            .join(" | ")
    } else {
        def.all_variants().map(|var| format!("'{}'", enum_variant_name(var, def))).join(" | ")
    }
}

fn generate_struct(def: &StructDef, type_map: &HashMap<String, String>) -> String {
    let mut type_def = "{".to_string();
    let type_tag = type_map.get(&def.ident().to_string());
    if let Some(type_tag) = type_tag {
        type_def.push_str(&format!("type: '{type_tag}';"));
    }
    for field in &def.fields {
        if field.markers.derive_attributes.estree.skip {
            continue;
        }
        let name = field.ident().expect("expected named field!").to_string();
        let name = name.strip_prefix("r#").map(ToString::to_string).unwrap_or(name);
        let ty = type_to_string(field.typ.name(), type_map);
        type_def.push_str(&format!("{name}: {ty};"));
    }
    type_def.push('}');
    type_def
}

fn type_to_string(ty: &TypeName, type_map: &HashMap<String, String>) -> String {
    match ty {
        TypeName::Ident(ident) => match ident.as_str() {
            "f64" | "f32" | "usize" | "u64" | "u32" | "u16" | "u8" | "i64" | "i32" | "i16"
            | "i8" => "number",
            "bool" => "boolean",
            "str" | "String" | "Atom" => "string",
            ty => type_map.get(ty).map_or(ty, |v| v.as_str()),
        }
        .to_string(),
        TypeName::Vec(type_name) => format!("Array<{}>", type_to_string(type_name, type_map)),
        TypeName::Box(type_name) | TypeName::Ref(type_name) | TypeName::Complex(type_name) => {
            type_to_string(type_name, type_map)
        }
        TypeName::Opt(type_name) => format!("({}) | null", type_to_string(type_name, type_map)),
    }
}

/// Unusable until oxc_prettier supports comments
#[allow(dead_code)]
fn format_typescript(source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions { preserve_parens: false, ..ParseOptions::default() })
        .parse();
    Prettier::new(
        &allocator,
        source_text,
        ret.trivias,
        PrettierOptions {
            semi: true,
            trailing_comma: TrailingComma::All,
            single_quote: true,
            ..PrettierOptions::default()
        },
    )
    .build(&ret.program)
}
