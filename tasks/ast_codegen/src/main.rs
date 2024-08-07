use std::{cell::RefCell, io::Read, path::PathBuf, rc::Rc};

use bpaf::{Bpaf, Parser};
use codegen::{AstCodegen, AstCodegenResult};
use itertools::Itertools;
use syn::parse_file;

mod codegen;
mod fmt;
mod generators;
mod layout;
mod markers;
mod passes;
mod rust_ast;
mod schema;
mod util;

use fmt::{cargo_fmt, pprint};
use generators::{
    AssertLayouts, AstBuilderGenerator, AstKindGenerator, DeriveCloneIn, DeriveGetSpan,
    DeriveGetSpanMut, GeneratedDataStream, GeneratedTokenStream, Generator, GeneratorOutput,
    VisitGenerator, VisitMutGenerator,
};
use passes::{CalcLayout, Linker, Pass};
use util::{write_all_to, NormalizeError};

static SOURCE_PATHS: &[&str] = &[
    "crates/oxc_ast/src/ast/literal.rs",
    "crates/oxc_ast/src/ast/js.rs",
    "crates/oxc_ast/src/ast/ts.rs",
    "crates/oxc_ast/src/ast/jsx.rs",
    "crates/oxc_syntax/src/number.rs",
    "crates/oxc_syntax/src/operator.rs",
    "crates/oxc_span/src/span/types.rs",
    "crates/oxc_span/src/source_type/types.rs",
];

const AST_CRATE: &str = "crates/oxc_ast";

type Result<R> = std::result::Result<R, String>;
type TypeId = usize;

#[derive(Debug, Bpaf)]
pub struct CliOptions {
    /// Runs all generators but won't write anything down.
    #[bpaf(switch)]
    dry_run: bool,
    /// Don't run cargo fmt at the end
    #[bpaf(switch)]
    no_fmt: bool,
    /// Path of output `schema.json`.
    schema: Option<std::path::PathBuf>,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli_options = cli_options().run();

    let AstCodegenResult { outputs, schema } = SOURCE_PATHS
        .iter()
        .fold(AstCodegen::default(), AstCodegen::add_file)
        .pass(Linker)
        .pass(CalcLayout)
        .gen(AssertLayouts)
        .gen(AstKindGenerator)
        .gen(AstBuilderGenerator)
        .gen(DeriveCloneIn)
        .gen(DeriveGetSpan)
        .gen(DeriveGetSpanMut)
        .gen(VisitGenerator)
        .gen(VisitMutGenerator)
        .generate()?;

    let (streams, outputs): (Vec<_>, Vec<_>) =
        outputs.into_iter().partition(|it| matches!(it.1, GeneratorOutput::Stream(_)));

    let (binaries, _): (Vec<_>, Vec<_>) =
        outputs.into_iter().partition(|it| matches!(it.1, GeneratorOutput::Data(_)));

    if !cli_options.dry_run {
        let side_effects =
            write_generated_streams(streams.into_iter().map(|it| it.1.into_stream()))?
                .into_iter()
                .chain(write_data_streams(binaries.into_iter().map(|it| it.1.into_data()))?)
                .collect();
        write_ci_filter(SOURCE_PATHS, side_effects, ".github/.generated_ast_watch_list.yml")?;
    }

    if !cli_options.no_fmt {
        cargo_fmt();
    }

    if let CliOptions { schema: Some(schema_path), dry_run: false, .. } = cli_options {
        let path = schema_path.to_str().expect("invalid path for schema output.");
        let schema = serde_json::to_string_pretty(&schema).normalize()?;
        write_all_to(schema.as_bytes(), path)?;
    }

    Ok(())
}

fn output(krate: &str, path: &str) -> PathBuf {
    std::path::PathBuf::from_iter(vec![krate, "src", "generated", path])
}

/// Writes all streams and returns a vector pointing to side-effects written on the disk
fn write_generated_streams(
    streams: impl IntoIterator<Item = GeneratedTokenStream>,
) -> std::io::Result<Vec<String>> {
    streams
        .into_iter()
        .map(|(path, stream)| {
            let path = path.into_os_string();
            let path = path.to_str().unwrap();
            let content = pprint(&stream);
            write_all_to(content.as_bytes(), path)?;
            Ok(path.to_string().replace('\\', "/"))
        })
        .collect()
}

/// Writes all streams and returns a vector pointing to side-effects written on the disk
fn write_data_streams(
    streams: impl IntoIterator<Item = GeneratedDataStream>,
) -> std::io::Result<Vec<String>> {
    streams
        .into_iter()
        .map(|(path, stream)| {
            let path = path.into_os_string();
            let path = path.to_str().unwrap();
            write_all_to(&stream, path)?;
            Ok(path.to_string().replace('\\', "/"))
        })
        .collect()
}

fn write_ci_filter(
    inputs: &[&str],
    side_effects: Vec<String>,
    output_path: &str,
) -> std::io::Result<()> {
    let file = file!().replace('\\', "/");
    let mut output = format!(
        "\
        # To edit this generated file you have to edit `{file}`\n\
        # Auto-generated code, DO NOT EDIT DIRECTLY!\n\n\
        src:\n"
    );
    let mut push_item = |path: &str| output.push_str(format!("  - '{path}'\n").as_str());

    for input in inputs {
        push_item(input);
    }

    for side_effect in side_effects {
        push_item(side_effect.as_str());
    }

    push_item("tasks/ast_codegen/src/**");

    write_all_to(output.as_bytes(), output_path)
}
