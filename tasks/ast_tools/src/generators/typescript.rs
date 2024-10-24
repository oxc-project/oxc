use convert_case::{Case, Casing};
use itertools::Itertools;
use std::{
    io::Write,
    process::{Command, Stdio},
};

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

const CUSTOM_TYPESCRIPT: &str = include_str!("../../../../crates/oxc_ast/src/ast/types.d.ts");

define_generator! {
    pub struct TypescriptGenerator;
}

impl Generator for TypescriptGenerator {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let file = file!().replace('\\', "/");
        let mut content = format!(
            "\
        		// To edit this generated file you have to edit `{file}`\n\
        		// Auto-generated code, DO NOT EDIT DIRECTLY!\n\n\
						{CUSTOM_TYPESCRIPT}\n\
						"
        );

        for def in ctx.schema() {
            if !def.generates_derive("ESTree") {
                continue;
            }
            let ts_type_def = match def {
                TypeDef::Struct(it) => Some(typescript_struct(it)),
                TypeDef::Enum(it) => typescript_enum(it),
            };
            let Some(ts_type_def) = ts_type_def else { continue };

            content.push_str(&ts_type_def);
            content.push_str("\n\n");
        }
        GeneratorOutput::Text {
            path: output(crate::TYPESCRIPT_PACKAGE, "types.d.ts"),
            content: format_typescript(&content),
        }
    }
}

// Untagged enums: "type Expression = BooleanLiteral | NullLiteral"
// Tagged enums: "type PropertyKind = 'init' | 'get' | 'set'"
fn typescript_enum(def: &EnumDef) -> Option<String> {
    if def.markers.estree.custom_ts_def {
        return None;
    }

    let union = if def.markers.estree.untagged {
        def.all_variants().map(|var| type_to_string(var.fields[0].typ.name())).join(" | ")
    } else {
        def.all_variants().map(|var| format!("'{}'", enum_variant_name(var, def))).join(" | ")
    };
    let ident = def.ident();
    Some(format!("export type {ident} = {union};"))
}

fn typescript_struct(def: &StructDef) -> String {
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

    let extends_union = extends.iter().any(|it| it.contains('|'));

    if extends_union {
        let extends =
            if extends.is_empty() { String::new() } else { format!(" & {}", extends.join(" & ")) };
        format!("export type {ident} = ({{{fields}\n}}){extends};")
    } else {
        let extends = if extends.is_empty() {
            String::new()
        } else {
            format!(" extends {}", extends.join(", "))
        };
        format!("export interface {ident}{extends} {{{fields}\n}}")
    }
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

fn format_typescript(source_text: &str) -> String {
    let mut dprint = Command::new("dprint")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["fmt", "--stdin", "types.d.ts"])
        .spawn()
        .expect("Failed to run dprint (is it installed?)");

    let stdin = dprint.stdin.as_mut().unwrap();
    stdin.write_all(source_text.as_bytes()).unwrap();
    stdin.flush().unwrap();

    let output = dprint.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

// Unusable until oxc_prettier supports comments
// fn format_typescript(source_text: &str) -> String {
//     let allocator = Allocator::default();
//     let source_type = SourceType::ts();
//     let ret = Parser::new(&allocator, source_text, source_type)
//         .with_options(ParseOptions { preserve_parens: false, ..ParseOptions::default() })
//         .parse();
//     Prettier::new(
//         &allocator,
//         PrettierOptions {
//             semi: true,
//             trailing_comma: TrailingComma::All,
//             single_quote: true,
//             ..PrettierOptions::default()
//         },
//     )
//     .build(&ret.program)
// }
