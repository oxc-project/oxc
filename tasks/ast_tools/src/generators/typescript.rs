use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_prettier::{Prettier, PrettierOptions, TrailingComma};
use oxc_span::SourceType;

use super::define_generator;
use crate::{
    codegen::LateCtx,
    output,
    schema::{EnumDef, GetIdent, StructDef, TypeDef, TypeName},
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
						// Auto-generated code, DO NOT EDIT DIRECTLY!\n\n\
						type bool = boolean;type f64 = number;type str = string;type Atom = string;type u32 = number;type u64 = number;\n\n"
        );

        for def in ctx.schema() {
            let type_def = match def {
                TypeDef::Struct(it) => generate_struct(it),
                TypeDef::Enum(it) => generate_enum(it),
            };

            let ident = def.ident();
            contents.push_str(&format!("export type {ident} = {type_def}\n\n",));
        }

        GeneratorOutput::Raw(output(crate::TYPESCRIPT_PACKAGE, "types.d.ts"), contents)
    }
}

fn generate_enum(def: &EnumDef) -> String {
    if def.markers.estree.untagged {
        def.all_variants().map(|v| v.ident().to_string()).join(" | ")
    } else {
        def.all_variants().map(|v| format!("'{}'", v.ident().to_string())).join(" | ")
    }
}

fn generate_struct(def: &StructDef) -> String {
    let mut type_def = "{".to_string();
    for field in &def.fields {
        let name = field.ident().expect("expected named field!").to_string();
        let name = name.strip_prefix("r#").map(ToString::to_string).unwrap_or(name);
        let ty = type_to_string(field.typ.name());
        type_def.push_str(&format!("{name}: {ty};"));
    }
    type_def.push('}');
    type_def
}

fn type_to_string(ty: &TypeName) -> String {
    match ty {
        TypeName::Ident(ident) => ident.to_string(),
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
