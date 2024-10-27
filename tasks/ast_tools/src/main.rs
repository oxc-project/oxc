#![allow(clippy::disallowed_methods)]

use std::{cell::RefCell, io::Read, path::PathBuf, rc::Rc};

use bpaf::{Bpaf, Parser};
use codegen::{AstCodegen, AstCodegenResult};
use itertools::Itertools;
use syn::parse_file;

mod codegen;
mod derives;
mod generators;
mod layout;
mod markers;
mod output;
mod passes;
mod rust_ast;
mod schema;
mod util;

use derives::{
    DeriveCloneIn, DeriveContentEq, DeriveContentHash, DeriveESTree, DeriveGetSpan,
    DeriveGetSpanMut,
};
use generators::{
    AssertLayouts, AstBuilderGenerator, AstKindGenerator, Generator, TypescriptGenerator,
    VisitGenerator, VisitMutGenerator,
};
use output::{write_all_to, RawOutput};
use passes::{CalcLayout, Linker};
use util::NormalizeError;

static SOURCE_PATHS: &[&str] = &[
    "crates/oxc_ast/src/ast/literal.rs",
    "crates/oxc_ast/src/ast/js.rs",
    "crates/oxc_ast/src/ast/ts.rs",
    "crates/oxc_ast/src/ast/jsx.rs",
    "crates/oxc_ast/src/ast/comment.rs",
    "crates/oxc_syntax/src/number.rs",
    "crates/oxc_syntax/src/operator.rs",
    "crates/oxc_span/src/span/types.rs",
    "crates/oxc_span/src/source_type/mod.rs",
    "crates/oxc_regular_expression/src/ast.rs",
];

const AST_CRATE: &str = "crates/oxc_ast";
const TYPESCRIPT_PACKAGE: &str = "npm/oxc-types";
const GITHUB_WATCH_LIST_PATH: &str = ".github/.generated_ast_watch_list.yml";

type Result<R> = std::result::Result<R, String>;
type TypeId = usize;

#[derive(Debug, Bpaf)]
pub struct CliOptions {
    /// Runs all generators but won't write anything down.
    #[bpaf(switch)]
    dry_run: bool,
    /// Prints no logs.
    quiet: bool,
    /// Path of output `schema.json`.
    schema: Option<std::path::PathBuf>,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli_options = cli_options().run();

    if cli_options.quiet {
        logger::quiet().normalize_with("Failed to set logger to `quiet` mode.")?;
    }

    let AstCodegenResult { mut outputs, schema } = SOURCE_PATHS
        .iter()
        .fold(AstCodegen::default(), AstCodegen::add_file)
        .pass(Linker)
        .pass(CalcLayout)
        .generate(DeriveCloneIn)
        .generate(DeriveGetSpan)
        .generate(DeriveGetSpanMut)
        .generate(DeriveContentEq)
        .generate(DeriveContentHash)
        .generate(DeriveESTree)
        .generate(AssertLayouts)
        .generate(AstKindGenerator)
        .generate(AstBuilderGenerator)
        .generate(VisitGenerator)
        .generate(VisitMutGenerator)
        .generate(TypescriptGenerator)
        .run()?;

    outputs.push(generate_ci_filter(&outputs));

    if !cli_options.dry_run {
        for output in outputs {
            output.write_to_file()?;
        }
    }

    if let CliOptions { schema: Some(schema_path), dry_run: false, .. } = cli_options {
        let schema = serde_json::to_string_pretty(&schema.defs).normalize()?;
        write_all_to(schema.as_bytes(), schema_path)?;
    }

    Ok(())
}

fn generate_ci_filter(outputs: &[RawOutput]) -> RawOutput {
    let file = file!().replace('\\', "/");
    let mut output = format!(
        "\
        # To edit this generated file you have to edit `{file}`\n\
        # Auto-generated code, DO NOT EDIT DIRECTLY!\n\n\
        src:\n"
    );
    let mut push_item = |path: &str| output.push_str(format!("  - '{path}'\n").as_str());

    for input in SOURCE_PATHS {
        push_item(input);
    }

    for output in outputs {
        push_item(output.path.as_str());
    }

    push_item("tasks/ast_tools/src/**");
    push_item(GITHUB_WATCH_LIST_PATH);

    RawOutput { path: GITHUB_WATCH_LIST_PATH.to_string(), content: output.into_bytes() }
}

#[macro_use]
mod logger {
    use std::sync::OnceLock;

    static LOG: OnceLock<bool> = OnceLock::new();

    pub(super) fn quiet() -> Result<(), bool> {
        LOG.set(false)
    }

    pub(super) fn __internal_log_enable() -> bool {
        *LOG.get_or_init(|| true)
    }

    macro_rules! log {
        ($fmt:literal $(, $args:expr)*) => {
            if $crate::logger::__internal_log_enable() {
                print!("{}", format!($fmt$(, $args)*));
            }
        }
    }

    macro_rules! log_success {
        () => {
            $crate::log!("Done!\n");
        };
    }

    macro_rules! log_failed {
        () => {
            $crate::log!("FAILED\n");
        };
    }

    macro_rules! log_result {
        ($result:expr) => {
            match &($result) {
                Ok(_) => {
                    $crate::log_success!();
                }
                Err(_) => {
                    $crate::log_failed!();
                }
            }
        };
    }

    pub(super) use {log, log_failed, log_result, log_success};
}

pub(crate) use logger::{log, log_failed, log_result, log_success};
