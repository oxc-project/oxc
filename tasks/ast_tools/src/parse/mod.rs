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

use indexmap::{IndexMap, IndexSet};
use oxc_index::IndexVec;
use rustc_hash::FxBuildHasher;
use syn::Ident;

use crate::{
    log, log_success,
    schema::Derives,
    schema::{File, FileId, Schema},
    Codegen,
};

pub mod attr;
mod load;
#[expect(clippy::module_inception)]
mod parse;
mod skeleton;
use load::load_file;
pub use parse::convert_expr_to_string;
use parse::parse;
use skeleton::Skeleton;

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
type FxIndexSet<K> = IndexSet<K, FxBuildHasher>;

/// Analyse the files with provided paths, and generate a [`Schema`].
pub fn parse_files(file_paths: &[&str], codegen: &Codegen) -> Schema {
    // Load files and populate `Vec` of skeletons + mapping from type name to `TypeId`.
    // `TypeId` is index into `skeletons`.
    let mut skeletons = FxIndexMap::default();

    let files = file_paths
        .iter()
        .enumerate()
        .map(|(file_id, &file_path)| {
            let file_id = FileId::from_usize(file_id);
            analyse_file(file_id, file_path, &mut skeletons)
        })
        .collect::<IndexVec<_, _>>();

    // Convert skeletons into schema
    parse(skeletons, files, codegen)
}

/// Analyse file with provided path and add types to `skeletons`.
///
/// Returns a [`File`].
fn analyse_file(
    file_id: FileId,
    file_path: &str,
    skeletons: &mut FxIndexMap<String, Skeleton>,
) -> File {
    log!("Load {file_path}... ");
    load_file(file_id, file_path, skeletons);
    log_success!();

    File::new(file_path)
}

/// Convert [`Ident`] to `String`, removing `r#` from start.
///
/// [`Ident`]: struct@Ident
fn ident_name(ident: &Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_string()
}
