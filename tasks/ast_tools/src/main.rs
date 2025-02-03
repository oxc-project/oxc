//! Generator of code related to AST.
//!
//! # Overview
//!
//! `oxc_ast_tools` is a framework for generating code related to the AST.
//!
//! There are 3 main elements to this crate:
//!
//! 1. [`Codegen`] - Contains data for running the code generation process.
//! 2. [`Schema`] - Schema of all AST types, and their inter-relations.
//! 3. [`Generator`]s and [`Derive`]s - Code generators which generate code, based on the [`Schema`].
//!
//! AST types are annotated with custom attributes (e.g. `#[visit]`, `#[scope]`).
//! These attributes guide [`Generator`]s and [`Derive`]s to generate code appropriately.
//!
//! [`Derive`]s are executed for a type if the type definition is tagged `#[generate_derive(TraitName)]`.
//!
//! The rest of this documentation explains how the code generation works, and how to add a new
//! code generator to this crate.
//!
//! # When code generation happens, and where it goes
//!
//! Code generation can be triggered by running this crate:
//!
//! ```sh
//! cargo run -p oxc_ast_tools
//! ```
//!
//! The generated code is checked into git.
//!
//! Code generation is *not* run automatically during compilation. This has 2 advantages:
//! 1. Code generation does not slow down compile times, unlike e.g. proc macros.
//! 2. Generated code can be viewed and navigated easily in an IDE, in the usual way.
//!
//! # Phases
//!
//! The codegen process proceeds in 5 phases:
//!
//! ### Phase 1: Load
//!
//! All the source files listed in [`SOURCE_PATHS`] are read, and parsed with [`syn`].
//!
//! At this stage, only type names and other basic information about types is obtained.
//! Each type is assigned a [`TypeId`], and a mapping of type name to [`TypeId`] is built.
//!
//! This is the bare minimum required to link types up to each other in the next phase.
//!
//! ### Phase 2: Parse
//!
//! In this phase, [`syn`]'s ASTs for each type definition are parsed in full to generate
//! a [`TypeDef`] for each type.
//!
//! These [`TypeDef`]s contain all the info about each type:
//!
//! * [`StructDef`] contains info about the name and type of every field in the struct.
//! * [`EnumDef`] contains info about all the enum's variants.
//!
//! Additional "defs" are created for other known types which are encountered in the AST:
//!
//! * `Option`: [`OptionDef`]
//! * `Box`: [`BoxDef`]
//! * `Vec`: [`VecDef`]
//! * `Cell`: [`CellDef`]
//! * Primitive types: [`PrimitiveDef`] - e.g. `u32`, `&str`
//! * Special types: [`PrimitiveDef`] - e.g. `Atom`
//!
//! The types are linked up to each other, so that all struct fields ([`FieldDef`]s) contain
//! the [`TypeId`] of the type that field contains. Ditto enum variants ([`VariantDef`]s).
//! Container types e.g. [`VecDef`] contain the `TypeId` of the inner type (e.g. `T` in `Vec<T>`).
//!
//! Custom attributes on types (e.g. `#[visit]`) are also parsed at this stage, in conjunction with
//! the [`Generator`]s and [`Derive`]s which define those attributes.
//!
//! The end result of this phase is the [`Schema`], which is the single source of truth about the AST.
//!
//! After this point, the types produced by `syn` are not used - all info about the AST is in
//! the [`Schema`], and everything from this point onwards works off the `Schema` only.
//!
//! ### Phase 3: Prepare
//!
//! [`Generator`]s and [`Derive`]s have already had a chance to input into the creation of the [`Schema`],
//! setting properties on [`StructDef`]s and [`EnumDef`] during parsing of custom attributes.
//!
//! However, at that point generators only had access to a single `StructDef` or `EnumDef` at a time.
//!
//! Now, in the prepare phase, generators can perform any modifications to the `Schema` that require
//! access to more than 1 [`TypeDef`] at the same time. They do this by implementing the
//! [`Generator::prepare`] or [`Derive::prepare`] method.
//! A good example of this is the [`AssertLayouts`] generator.
//!
//! At the end of this phase, the [`Schema`] is locked as read-only.
//!
//! ### Phase 4: Generate
//!
//! This is main code-generation phase.
//!
//! Each generator is run in parallel, and provided an immutable reference to [`Schema`] and [`Codegen`].
//!
//! The difference between `Generator`s and `Derive`s is:
//!
//! * [`Generator`]s act on the entire AST in one go. They can generate 1 or more [`Output`]s,
//!   which can be Rust code, JS code, or other types of output.
//!
//! * [`Derive`]s act on a single type at a time (though they also have access to the whole `Schema`).
//!   [`Derive::derive`] should return a [`TokenStream`] containing an implementation of the trait
//!   the `Derive` is for. `oxc_ast_tools` combines these into a single output file for each crate.
//!
//! [`Output`]s are converted to [`RawOutput`]s, which includes formatting the generated code
//! with `rustfmt` or `dprint`.
//!
//! ### Phase 5: Output
//!
//! All [`RawOutput`]s generated in previous phase are written to disk.
//!
//! # Generators and Derives
//!
//! [`Generator`]s and [`Derive`]s should keep "special case" logic written with the generator's code
//! to a minimum (and ideally not do it at all).
//!
//! Any info that the generator needs about how to treat each type should be recorded on the type
//! definition itself, with custom attributes e.g. `#[visit]`, `#[clone_in(default)]` - instead of
//! hard-coding those cases within the generator code itself.
//!
//! A generator defines attributes that it uses by implementing [`Generator::attrs`] / [`Derive::attrs`]
//! method. During parsing phase, [`Generator::parse_attr`] / [`Derive::parse_attr`] will be called
//! with details of where those attributes were found, and the generator can record that info in the
//! `Schema`.
//!
//! # Creating a new Generator or Derive
//!
//! ## [`Generator`]
//! * Add a file to `generators` directory in this crate e.g. `generators/picture.rs`.
//! * Add a reference to it to [`GENERATORS`] in `main.rs`.
//!
//! ## [`Derive`]
//! * Add a file to `derives` directory in this crate e.g. `derives/get_flaps.rs`.
//! * Add a reference to it to [`DERIVES`] in `main.rs`.
//!
//! ## Both
//! * If the generator needs to store extra info in the `Schema`, create a file in `schema/extensions`
//!   directory e.g. `schema/extensions/picture.rs`.
//! * Import that file into `mod extensions` in `schema/mod.rs`.
//! * That file should define types for structs / enums / struct fields / enum variants, depending on
//!   where the data needs to be stored. e.g. `PictureStruct`, `PictureEnumField`.
//! * Those types must implement `Default` and `Debug`.
//! * Add those types to [`StructDef`], [`EnumDef`], [`FieldDef`] and/or [`VariantDef`].
//! * Implement [`Generator::attrs`] / [`Derive::attrs`] to declare the generator's custom attributes.
//! * Implement [`Generator::parse_attr`] / [`Derive::parse_attr`] to parse those attributes
//!   and mutate the "extension" types in [`Schema`] as required.
//! * Add the attributes' names to the list on `ast_derive` in `crates/oxc_ast_macros/src/lib.rs`.
//!
//! #### Attributes
//!
//! `oxc_ast_tools` provides abstractions [`AttrLocation`] and [`AttrPart`] which assist with parsing
//! custom attributes, and are much simpler than `syn`'s types.
//!
//! [`TypeId`]: schema::TypeId
//! [`TypeDef`]: schema::TypeDef
//! [`StructDef`]: schema::StructDef
//! [`EnumDef`]: schema::EnumDef
//! [`OptionDef`]: schema::OptionDef
//! [`BoxDef`]: schema::BoxDef
//! [`VecDef`]: schema::VecDef
//! [`CellDef`]: schema::CellDef
//! [`PrimitiveDef`]: schema::PrimitiveDef
//! [`FieldDef`]: schema::FieldDef
//! [`VariantDef`]: schema::VariantDef
//! [`AssertLayouts`]: generators::AssertLayouts
//! [`TokenStream`]: proc_macro2::TokenStream
//! [`AttrLocation`]: parse::attr::AttrLocation
//! [`AttrPart`]: parse::attr::AttrPart

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
