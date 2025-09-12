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
//! Each type with an `#[ast]` attribute is assigned a [`TypeId`], and a mapping of type name
//! to [`TypeId`] is built.
//!
//! Any "meta types" with an `#[ast_meta]` attribute are also identified.
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
//! the [`Generator`]s and [`Derive`]s which define those attributes. Meta types ([`MetaType`]s)
//! and their custom attributes are also parsed.
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
//! * Add those types to [`StructDef`], [`EnumDef`], [`FieldDef`], [`VariantDef`] and/or [`MetaType`].
//! * Implement [`Generator::attrs`] / [`Derive::attrs`] to declare the generator's custom attributes.
//! * Implement [`Generator::parse_attr`] / [`Derive::parse_attr`] to parse those attributes
//!   and mutate the "extension" types in [`Schema`] as required.
//!
//! #### Attributes
//!
//! `oxc_ast_tools` provides abstractions [`AttrLocation`] and [`AttrPart`] which assist with parsing
//! custom attributes, and are much simpler than `syn`'s types.
//!
//! #### Meta types
//!
//! Meta types ([`MetaType`]) are types which are not part of the AST, but are used by `oxc_ast_tools`
//! in some way, and may also be used in generated output. Tagging a type with `#[ast_meta]` attribute
//! and then adding further custom attributes to that type is a way to pass ancillary information to a
//! `Derive` / `Generator`.
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
//! [`MetaType`]: schema::MetaType
//! [`AssertLayouts`]: generators::AssertLayouts
//! [`TokenStream`]: proc_macro2::TokenStream
//! [`AttrLocation`]: parse::attr::AttrLocation
//! [`AttrPart`]: parse::attr::AttrPart

use std::fs;

use bpaf::{Bpaf, Parser};
use quote::quote;
use rayon::prelude::*;

mod codegen;
mod derives;
mod generators;
mod logger;
mod output;
mod parse;
mod schema;
mod utils;

use codegen::{Codegen, Runner, get_runners};
use derives::Derive;
use generators::Generator;
use logger::{log, log_failed, log_result, log_success, logln};
use output::{Output, RawOutput, output_path};
use parse::parse_files;
use schema::Schema;
use utils::create_ident;

/// Paths to source files containing AST types
static SOURCE_PATHS: &[&str] = &[
    "crates/oxc_allocator/src/pool/fixed_size.rs",
    "crates/oxc_ast/src/ast/js.rs",
    "crates/oxc_ast/src/ast/literal.rs",
    "crates/oxc_ast/src/ast/jsx.rs",
    "crates/oxc_ast/src/ast/ts.rs",
    "crates/oxc_ast/src/ast/comment.rs",
    "crates/oxc_ast/src/serialize/mod.rs",
    "crates/oxc_ast/src/serialize/basic.rs",
    "crates/oxc_ast/src/serialize/literal.rs",
    "crates/oxc_ast/src/serialize/js.rs",
    "crates/oxc_ast/src/serialize/jsx.rs",
    "crates/oxc_ast/src/serialize/ts.rs",
    "crates/oxc_linter/src/lib.rs",
    "crates/oxc_syntax/src/lib.rs",
    "crates/oxc_syntax/src/comment_node.rs",
    "crates/oxc_syntax/src/module_record.rs",
    "crates/oxc_syntax/src/number.rs",
    "crates/oxc_syntax/src/operator.rs",
    "crates/oxc_syntax/src/scope.rs",
    "crates/oxc_syntax/src/serialize.rs",
    "crates/oxc_syntax/src/symbol.rs",
    "crates/oxc_syntax/src/reference.rs",
    "crates/oxc_span/src/span.rs",
    "crates/oxc_span/src/source_type.rs",
    "crates/oxc_regular_expression/src/ast.rs",
    "napi/parser/src/raw_transfer_types.rs",
];

/// Path to `oxc_allocator` crate
const ALLOCATOR_CRATE_PATH: &str = "crates/oxc_allocator";

/// Path to `oxc_ast` crate
const AST_CRATE_PATH: &str = "crates/oxc_ast";

/// Path to `oxc_ast_visit` crate
const AST_VISIT_CRATE_PATH: &str = "crates/oxc_ast_visit";

/// Path to `oxc_ast_macros` crate
const AST_MACROS_CRATE_PATH: &str = "crates/oxc_ast_macros";

/// Path to `oxc_traverse` crate
const TRAVERSE_CRATE_PATH: &str = "crates/oxc_traverse";

/// Path to write TS type definitions to
const TYPESCRIPT_DEFINITIONS_PATH: &str = "npm/oxc-types/types.d.ts";

/// Path to NAPI parser package
const NAPI_PARSER_PACKAGE_PATH: &str = "napi/parser";

/// Path to NAPI oxlint package
const NAPI_OXLINT_PACKAGE_PATH: &str = "napi/oxlint";

/// Path to write AST changes filter list to
const AST_CHANGES_WATCH_LIST_PATH: &str = ".github/generated/ast_changes_watch_list.yml";

/// Derives (for use with `#[generate_derive]`)
const DERIVES: &[&(dyn Derive + Sync)] = &[
    &derives::DeriveCloneIn,
    &derives::DeriveDummy,
    &derives::DeriveTakeIn,
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
    &generators::ScopesCollectorGenerator,
    &generators::Utf8ToUtf16ConverterGenerator,
    &generators::RawTransferGenerator,
    &generators::RawTransferLazyGenerator,
    &generators::TypescriptGenerator,
    &generators::FormatterFormatGenerator,
    &generators::FormatterAstNodesGenerator,
];

/// Attributes on structs and enums (not including those defined by derives/generators)
const ATTRIBUTES: [&str; 2] = ["generate_derive", "plural"];

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
        runner.prepare(&mut schema, &codegen);
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

    // Edit `lib.rs` in `oxc_ast_macros` crate
    outputs.push(generate_proc_macro());
    outputs.push(generate_updated_proc_macro(&codegen));

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

    let paths =
        SOURCE_PATHS.iter().copied().chain(outputs.iter().map(|output| output.path.as_str()));
    let output = Output::yaml_watch_list(AST_CHANGES_WATCH_LIST_PATH, paths);

    log_success!();

    output.into_raw(file!())
}

/// Generate function for proc macro in `oxc_ast_macros` crate.
///
/// This function translates trait name to path to the trait and any generic params.
fn generate_proc_macro() -> RawOutput {
    let match_arms = DERIVES.iter().map(|derive| {
        let trait_name = derive.trait_name();
        let trait_ident = create_ident(trait_name);
        let crate_ident = create_ident(derive.crate_name());
        if derive.trait_has_lifetime() {
            quote!( #trait_name => (quote!(::#crate_ident::#trait_ident), quote!( <'static> )) )
        } else {
            quote!( #trait_name => (quote!(::#crate_ident::#trait_ident), TokenStream::new()) )
        }
    });

    let output = quote! {
        use proc_macro2::TokenStream;
        use quote::quote;

        ///@@line_break
        pub fn get_trait_crate_and_generics(trait_name: &str) -> Option<(TokenStream, TokenStream)> {
            let res = match trait_name {
                #(#match_arms,)*
                _ => return None,
            };
            Some(res)
        }
    };

    Output::Rust { path: output_path(AST_MACROS_CRATE_PATH, "derived_traits.rs"), tokens: output }
        .into_raw(file!())
}

/// Update the list of helper attributes for `Ast` derive proc macro in `oxc_ast_macros` crate
/// to include all attrs which generators/derives utilize.
///
/// Unfortunately we can't add a separate generated file for this, as proc macros can only be declared
/// in the main `lib.rs` of a proc macro crate. So we have to edit the existing file.
fn generate_updated_proc_macro(codegen: &Codegen) -> RawOutput {
    // Get all attrs which derives/generators use
    let mut attrs = codegen.attrs();
    attrs.extend(ATTRIBUTES);
    attrs.sort_unstable();
    let attrs = attrs.join(", ");

    // Load `oxc_ast_macros` crate's `lib.rs` file.
    // Substitute list of used attrs into `#[proc_macro_derive(Ast, attributes(...))]`.
    let path = format!("{AST_MACROS_CRATE_PATH}/src/lib.rs");
    let code = fs::read_to_string(&path).unwrap();
    let (start, end) = code.split_once("#[proc_macro_derive(").unwrap();
    let (_, end) = end.split_once(")]").unwrap();
    assert!(end.starts_with("\npub fn ast_derive("));
    let code = format!("{start}#[proc_macro_derive(Ast, attributes({attrs}))]{end}");

    Output::RustString { path, code }.into_raw("")
}
