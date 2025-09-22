//! Create schema of types from Rust source files.
//!
//! Parsing proceeds in 2 phases:
//!
//! 1. Parse Rust source files and build list of all type definitions.
//! 2. Parse all the types into [`TypeDef`]s and link them to each other, to form the [`Schema`].
//!
//! ## Phase 1
//!
//! 1st phase involves minimal parsing, just enough to identify structs/enums with `#[ast]` attr,
//! and to get names of types.
//!
//! Each type gets assigned a [`TypeId`], and is represented by a [`Skeleton`].
//! An indexed hash map is built, mapping type names to their [`TypeId`]s and [`Skeleton`]s.
//!
//! ## Phase 2
//!
//! 2nd phase involves full parsing of each type, and linking types to each other.
//!
//! A [`TypeDef`] is generated for each type. The `IndexVec<TypeId, TypeDef>` that is created is indexed
//! by [`TypeId`] - same order of entries as the `FxIndexMap<Skeleton>` from phase 1.
//!
//! `parse_attr` method is called on [`Derive`]s and [`Generator`]s which handle attributes,
//! for the derive/generator to parse the attribute and update the [`TypeDef`] accordingly.
//!
//! [`TypeDef`]s are also created for other types which are found within the type definitions:
//!
//! * Primitives (e.g. `f64`, `&str`).
//! * Known types (`Vec`, `Box`, `Option`, `Cell`).
//! * Special cases (`Atom`, `RegExpFlags`, `ScopeId`, `SymbolId`, `ReferenceId`).
//!
//! Each [`TypeDef`] contains a [`FileId`], indicating which file the type was defined in.
//!
//! Note: Individual [`TypeDef`]s are created for every different `Vec`, `Box`, `Option` and `Cell`.
//! i.e. There are separate [`TypeDef`]s for `Vec<Statement>` and `Vec<Expression>`,
//! not a single [`TypeDef`] for `Vec`.
//!
//! ## Schema
//!
//! [`Schema`] contains all the [`TypeDef`]s and [`File`]s.
//!
//! * `types: IndexVec<TypeId, TypeDef>` is indexed by [`TypeId`].
//! * `files: IndexVec<FileId, File>` is indexed by [`FileId`].
//!
//! [`TypeId`]: crate::schema::TypeId
//! [`TypeDef`]: crate::schema::TypeDef
//! [`Derive`]: crate::Derive
//! [`Generator`]: crate::Generator

use std::path::Path;

use oxc_index::IndexVec;

use crate::{
    Codegen, log, log_success,
    schema::{Derives, File, FileId, Schema},
    utils::FxIndexMap,
};

pub mod attr;
mod load;
#[expect(clippy::module_inception)]
mod parse;
mod skeleton;
use load::load_file;
use parse::parse;
use skeleton::Skeleton;

/// Analyse the files with provided paths, and generate a [`Schema`].
pub fn parse_files(file_paths: &[&str], codegen: &Codegen) -> Schema {
    // Load files and populate `skeletons` and `meta_skeletons` + mapping from type name to `TypeId`.
    // `skeletons` contains details of types marked with `#[ast]` attribute.
    // `meta_skeletons` contains details of types marked with `#[ast_meta]` attribute.
    // Meta types are not part of the AST, but associated with it.
    // `TypeId` is index into `skeletons`.
    // `MetaId` is index into `meta_skeletons`.
    let mut skeletons = FxIndexMap::default();
    let mut meta_skeletons = FxIndexMap::default();

    let files = file_paths
        .iter()
        .enumerate()
        .map(|(file_id, &file_path)| {
            let file_id = FileId::from_usize(file_id);
            analyse_file(
                file_id,
                file_path,
                &mut skeletons,
                &mut meta_skeletons,
                codegen.root_path(),
            )
        })
        .collect::<IndexVec<_, _>>();

    // Convert skeletons into schema
    parse(skeletons, meta_skeletons, files, codegen)
}

/// Analyse file with provided path and add types to `skeletons` and `meta_skeletons`.
///
/// Returns a [`File`].
fn analyse_file(
    file_id: FileId,
    file_path: &str,
    skeletons: &mut FxIndexMap<String, Skeleton>,
    meta_skeletons: &mut FxIndexMap<String, Skeleton>,
    root_path: &Path,
) -> File {
    log!("Load {file_path}... ");
    load_file(file_id, file_path, skeletons, meta_skeletons, root_path);
    log_success!();

    File::new(file_path)
}
