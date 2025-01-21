//! Create schema of types from Rust source files.
//!
//! Analysis proceeds in 2 phases:
//!
//! 1. Parse Rust source files and build list of all type definitions.
//! 2. Parse all the types into `TypeDef`s.
//!
//! ## Phase 1
//!
//! 1st phase involves minimal parsing, just enough to identify structs/enums with `#[ast]` attr,
//! and to get names of types.
//!
//! Each type gets assigned a `TypeId`, and is represented by a `Skeleton`.
//! An indexed hash map is built, mapping type names to their `TypeId`s and `Skeleton`s.
//!
//! ## Phase 2
//!
//! 2nd phase involves full parsing of each type, and linking types to each other.
//!
//! A `TypeDef` is generated for each type. The `Vec<TypeDef>` that is created is indexed by `TypeId`
//! - same order of entries as the `FxIndexMap<Skeleton>` from phase 1.
//!
//! `TypeDef`s are also created for other types which are found within the type definitions:
//!
//! * Primitives (e.g. `f64`, `&str`).
//! * Known types (`Vec`, `Box`, `Option`, `Cell`).
//! * Special cases (`Atom`, `RegExpFlags`, `ScopeId`, `SymbolId`, `ReferenceId`).
//!
//! Each `TypeDef` contains a `FileId`, indicating which file the type was defined in.
//!
//! Note: Individual `TypeDef`s are created for every different `Vec`, `Box`, `Option` and `Cell`.
//! i.e. There are separate `TypeDef`s for `Vec<Statement>` and `Vec<Expression>`,
//! not a single `TypeDef` for `Vec`.
//!
//! ## Schema
//!
//! `Schema` contains all the `TypeDef`s and `File`s.
//!
//! * `defs: Vec<TypeDef>` is indexed by `TypeId`.
//! * `files: Vec<File>` is indexed by `FileId`.

use std::hash::BuildHasherDefault;

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rustc_hash::FxHasher;

use crate::{log, log_success, Codegen};

mod defs;
mod derives;
mod load;
mod parse;
mod schema;
mod skeleton;
use derives::Derives;
use load::load_file;
use parse::parse;
use schema::{File, FileId, Schema};
use skeleton::Skeleton;

pub type DeriveId = usize;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;

/// Analyse the files with provided paths, and generate a `Schema`.
pub fn analyse(file_paths: &[&str], codegen: &Codegen) -> Schema {
    // Load files and populate `Vec` of skeletons + mapping from type name to `TypeId`.
    // `TypeId` is index into `skeletons`.
    let mut skeletons = FxIndexMap::default();

    let files = file_paths
        .iter()
        .enumerate()
        .map(|(file_id, &file_path)| analyse_file(file_id, file_path, &mut skeletons))
        .collect::<Vec<_>>();

    // Convert skeletons into schema
    parse(skeletons, files, codegen)
}

/// Analyse file with provided path and add types to `skeletons`.
///
/// Returns a `File`.
fn analyse_file(
    file_id: FileId,
    file_path: &str,
    skeletons: &mut FxIndexMap<String, Skeleton>,
) -> File {
    log!("Load {file_path}... ");
    let import_path = get_import_path(file_path);
    load_file(file_id, file_path, skeletons);
    log_success!();

    File { file_path: file_path.to_string(), import_path }
}

/// Convert file path to import path.
/// `crates/oxc_ast/src/ast/js.rs` -> `oxc_ast::ast::js`.
/// `crates/oxc_span/src/source_type/mod.rs` -> `oxc_span::source_type`.
fn get_import_path(file_path: &str) -> String {
    let path = file_path.trim_end_matches(".rs").trim_end_matches("/mod");

    let mut parts = path.split('/');
    assert_eq!(parts.next(), Some("crates"));
    let krate = parts.next().unwrap();
    assert_eq!(parts.next(), Some("src"));

    [krate].into_iter().chain(parts).join("::")
}
