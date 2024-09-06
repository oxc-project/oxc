use std::{cell::RefCell, io::Read, path::PathBuf, rc::Rc};

use bpaf::{Bpaf, Parser};
use codegen::{AstCodegen, AstCodegenResult};
use itertools::Itertools;
use syn::parse_file;

mod codegen;
mod derives;
mod fmt;
mod generators;
mod layout;
mod markers;
mod passes;
mod rust_ast;
mod schema;
mod util;

use derives::{DeriveCloneIn, DeriveContentEq, DeriveContentHash, DeriveGetSpan, DeriveGetSpanMut};
use fmt::cargo_fmt;
use generators::{
    AssertLayouts, AstBuilderGenerator, AstKindGenerator, Generator, GeneratorOutput,
    VisitGenerator, VisitMutGenerator,
};
use passes::{CalcLayout, Linker};
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
    "crates/oxc_regular_expression/src/ast.rs",
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
    /// Prints no logs.
    quiet: bool,
    /// Path of output `schema.json`.
    schema: Option<std::path::PathBuf>,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli_options = cli_options().run();

    if cli_options.quiet {
        // SAFETY: we haven't started using logger yet!
        unsafe { logger::quiet() };
    }

    let AstCodegenResult { outputs, schema } = SOURCE_PATHS
        .iter()
        .fold(AstCodegen::default(), AstCodegen::add_file)
        .pass(Linker)
        .pass(CalcLayout)
        .derive(DeriveCloneIn)
        .derive(DeriveGetSpan)
        .derive(DeriveGetSpanMut)
        .derive(DeriveContentEq)
        .derive(DeriveContentHash)
        .generate(AssertLayouts)
        .generate(AstKindGenerator)
        .generate(AstBuilderGenerator)
        .generate(VisitGenerator)
        .generate(VisitMutGenerator)
        .run()?;

    if !cli_options.dry_run {
        let side_effects = outputs
            .into_iter()
            .map(|it| {
                let path = it.path();
                log!("Writing {path}...");
                it.apply().unwrap();
                logln!(" Done!");
                path
            })
            .collect();
        write_ci_filter(SOURCE_PATHS, side_effects, ".github/.generated_ast_watch_list.yml")?;
    }

    if !cli_options.no_fmt {
        cargo_fmt();
    }

    if let CliOptions { schema: Some(schema_path), dry_run: false, .. } = cli_options {
        let path = schema_path.to_str().expect("invalid path for schema output.");
        let schema = serde_json::to_string_pretty(&schema.defs).normalize()?;
        write_all_to(schema.as_bytes(), path)?;
    }

    Ok(())
}

fn output(krate: &str, path: &str) -> PathBuf {
    std::path::PathBuf::from_iter(vec![krate, "src", "generated", path])
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

    push_item("tasks/ast_codegen/src/**/*");
    push_item(output_path);

    log!("Writing {output_path}...");
    write_all_to(output.as_bytes(), output_path)?;
    logln!(" Done!");
    Ok(())
}

#[macro_use]
mod logger {
    static mut LOG: bool = true;

    /// Shouldn't be called after first use of the logger macros.
    pub(super) unsafe fn quiet() {
        LOG = false;
    }

    pub(super) fn __internal_log_enable() -> bool {
        // SAFETY:`LOG` doesn't change from the moment we start using it.
        unsafe { LOG }
    }

    macro_rules! log {
        ($fmt:literal $(, $args:expr)*) => {
            if $crate::logger::__internal_log_enable() {
                print!("{}", format!($fmt$(, $args)*));
            }
        }
    }

    macro_rules! logln {
        ($fmt:literal $(, $args:expr)*) => {
            $crate::log!("{}\n", format!($fmt $(, $args)*));
        }
    }

    pub(super) use {log, logln};
}

pub(crate) use logger::{log, logln};
