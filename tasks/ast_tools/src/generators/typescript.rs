use convert_case::{Case, Casing};
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

// TODO: Generate directly to types.d.ts instead of relying on wasm-bindgen

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

        for def in ctx.schema() {
            if !def.generates_derive("ESTree") {
                continue;
            }
            let type_def = match def {
                TypeDef::Struct(it) => generate_struct(it),
                TypeDef::Enum(it) => generate_enum(it),
            };
            contents.push_str(&type_def);
            contents.push_str("\n\n");
        }

        GeneratorOutput::Raw(output(crate::TYPESCRIPT_PACKAGE, "types.d.ts"), contents)
    }
}

// Untagged enums: "type Expression = BooleanLiteral | NullLiteral"
// Tagged enums: "type PropertyKind = 'init' | 'get' | 'set'"
fn generate_enum(def: &EnumDef) -> String {
    let union = if def.markers.estree.untagged {
        def.all_variants().map(|var| type_to_string(var.fields[0].typ.name())).join(" | ")
    } else {
        def.all_variants().map(|var| format!("'{}'", enum_variant_name(var, def))).join(" | ")
    };
    let ident = def.ident();
    format!("export type {ident} = {union};")
}

fn generate_struct(def: &StructDef) -> String {
    let ident = def.ident();
    let mut fields = String::new();
    let mut extends = vec![];

    if let Some(type_tag) = get_type_tag(def) {
        fields.push_str(&format!("\n\ttype: '{type_tag}';"));
    }

    for field in &def.fields {
        if field.markers.derive_attributes.estree.skip {
            continue;
        }
        let ty = match &field.markers.derive_attributes.tsify_type {
            Some(ty) => ty.clone(),
            None => type_to_string(field.typ.name()),
        };

        if field.markers.derive_attributes.estree.flatten {
            extends.push(ty);
            continue;
        }

        let name = match &field.markers.derive_attributes.estree.rename {
            Some(rename) => rename.to_string(),
            None => field.name.clone().unwrap().to_case(Case::Camel),
        };

        fields.push_str(&format!("\n\t{name}: {ty};"));
    }
    let extends =
        if extends.is_empty() { String::new() } else { format!(" & {}", extends.join(" & ")) };
    format!("export type {ident} = ({{{fields}\n}}){extends};")
}

fn type_to_string(ty: &TypeName) -> String {
    match ty {
        TypeName::Ident(ident) => match ident.as_str() {
            "f64" | "f32" | "usize" | "u64" | "u32" | "u16" | "u8" | "i64" | "i32" | "i16"
            | "i8" => "number",
            "bool" => "boolean",
            "str" | "String" | "Atom" | "CompactStr" => "string",
            ty => ty,
        }
        .to_string(),
        TypeName::Vec(type_name) => format!("Array<{}>", type_to_string(type_name)),
        TypeName::Box(type_name) | TypeName::Ref(type_name) | TypeName::Complex(type_name) => {
            type_to_string(type_name)
        }
        TypeName::Opt(type_name) => format!("({}) | null", type_to_string(type_name)),
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
