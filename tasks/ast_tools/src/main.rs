use std::fmt::Write;

use bpaf::{Bpaf, Parser};
use rayon::prelude::*;

mod codegen;
mod derives;
mod generators;
mod logger;
mod output;
mod parse;
mod schema;
mod utils;

use codegen::{get_runners, Codegen, Runner};
use derives::Derive;
use generators::Generator;
use logger::{log, log_failed, log_result, log_success, logln};
use output::{Output, RawOutput};
use parse::parse_files;
use schema::Schema;

/// Paths to source files containing AST types
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
    "crates/oxc_syntax/src/scope.rs",
    "crates/oxc_syntax/src/symbol.rs",
    "crates/oxc_syntax/src/reference.rs",
];

/// Path to `oxc_ast` crate
const AST_CRATE: &str = "crates/oxc_ast";

/// Path to write TS type definitions to
const TYPESCRIPT_DEFINITIONS_PATH: &str = "npm/oxc-types/types.d.ts";

/// Path to write CI filter list to
const GITHUB_WATCH_LIST_PATH: &str = ".github/.generated_ast_watch_list.yml";

/// Derives (for use with `#[generate_derive]`)
const DERIVES: &[&(dyn Derive + Sync)] = &[
    &derives::DeriveCloneIn,
    &derives::DeriveGetAddress,
    &derives::DeriveGetSpan,
    &derives::DeriveGetSpanMut,
    &derives::DeriveContentEq,
    &derives::DeriveESTree,
];

/// Code generators
const GENERATORS: &[&(dyn Generator + Sync)] = &[
    &generators::AssertLayouts,
    &generators::AstKindGenerator,
    &generators::AstBuilderGenerator,
    &generators::GetIdGenerator,
    &generators::VisitGenerator,
    &generators::TypescriptGenerator,
];

type Result<R> = std::result::Result<R, ()>;

/// CLI options.
#[derive(Debug, Bpaf)]
struct CliOptions {
    /// Run all generators but don't write to disk
    dry_run: bool,
    /// Run all generators in series (useful when debugging)
    serial: bool,
    /// Print no logs
    quiet: bool,
}

fn main() {
    // Parse CLI options
    let options = cli_options().run();

    // Init logger
    if options.quiet {
        logger::quiet();
    }

    // Parse inputs and generate `Schema`
    let codegen = Codegen::new();
    let mut schema = parse_files(SOURCE_PATHS, &codegen);

    // Run `prepare` actions
    let runners = get_runners();
    for runner in &runners {
        runner.prepare(&mut schema);
    }

    // Run generators
    let mut outputs = if options.serial {
        // Run in series
        let mut outputs = vec![];
        for runner in &runners {
            outputs.extend(runner.run(&schema, &codegen));
        }
        outputs
    } else {
        // Run in parallel
        runners.par_iter().map(|runner| runner.run(&schema, &codegen)).reduce(
            Vec::new,
            |mut outputs, runner_outputs| {
                outputs.extend(runner_outputs);
                outputs
            },
        )
    };

    logln!("All Derives and Generators... Done!");

    // Add CI filter file to outputs
    outputs.sort_unstable_by(|o1, o2| o1.path.cmp(&o2.path));
    outputs.push(generate_ci_filter(&outputs));

    // Write outputs to disk
    if !options.dry_run {
        for output in outputs {
            output.write_to_file().unwrap();
        }
    }
}

/// Generate CI filter list file.
///
/// This is used in `ast_changes` CI job to skip running `oxc_ast_tools`
/// unless relevant files have changed.
///
/// List includes source files, generated files, and all files in `oxc_ast_tools` itself.
fn generate_ci_filter(outputs: &[RawOutput]) -> RawOutput {
    log!("Generate CI filter... ");

    let mut paths = SOURCE_PATHS
        .iter()
        .copied()
        .chain(outputs.iter().map(|output| output.path.as_str()))
        .chain(["tasks/ast_tools/src/**", GITHUB_WATCH_LIST_PATH])
        .collect::<Vec<_>>();
    paths.sort_unstable();

    let mut code = "src:\n".to_string();
    for path in paths {
        writeln!(&mut code, "  - '{path}'").unwrap();
    }

    log_success!();

    Output::Yaml { path: GITHUB_WATCH_LIST_PATH.to_string(), code }.into_raw(file!())
}
