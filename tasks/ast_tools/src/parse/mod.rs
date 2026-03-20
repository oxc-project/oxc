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
//! by [`TypeId`] - same order of entries as the `FxIndexSet` of type names from phase 1.
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
//! [`Skeleton`]: skeleton::Skeleton
//! [`Derive`]: crate::Derive
//! [`Generator`]: crate::Generator

use rayon::prelude::*;

use oxc_index::IndexVec;

use crate::{
    Codegen, log, log_success,
    schema::{Derives, File, FileId, Schema},
    utils::FxIndexSet,
};

pub mod attr;
mod load;
#[expect(clippy::module_inception)]
mod parse;
mod skeleton;
use load::load_file;
use parse::parse;

/// Analyse the files with provided paths, and generate a [`Schema`].
pub fn parse_files(file_paths: &[String], codegen: &Codegen) -> Schema {
    // Load files and populate `skeletons` and `meta_skeletons` + mapping from type name to `TypeId`.
    // `skeletons` contains details of types marked with `#[ast]` attribute.
    // `meta_skeletons` contains details of types marked with `#[ast_meta]` attribute.
    // Meta types are not part of the AST, but associated with it.
    // `TypeId` is index into `skeletons`.
    // `MetaId` is index into `meta_skeletons`.
    log!("Loading files... ");
    let results = file_paths
        .par_iter()
        .enumerate()
        .map(|(file_id, file_path)| {
            let file_id = FileId::from_usize(file_id);
            let file_skeletons = load_file(file_id, file_path, codegen.root_path());
            // `Skeleton` contains `syn` types which are `!Send` (see `AssertSend` below)
            AssertSend((file_path, file_skeletons))
        })
        .collect::<Vec<_>>();
    log_success!();

    // Sequential phase: merge into name sets + skeleton vecs (preserving deterministic order).
    let mut type_names = FxIndexSet::default();
    let mut type_skeletons = Vec::new();
    let mut meta_names = FxIndexSet::default();
    let mut meta_skeletons = Vec::new();
    let mut files = IndexVec::new();

    for AssertSend((file_path, file_skeletons)) in results {
        for (name, skeleton, is_meta) in file_skeletons {
            let (names, skeletons) = if is_meta {
                (&mut meta_names, &mut meta_skeletons)
            } else {
                (&mut type_names, &mut type_skeletons)
            };

            let (index, is_new) = names.insert_full(name);
            assert!(is_new, "2 types with same name: {}", names.get_index(index).unwrap());
            skeletons.push(skeleton);
        }
        files.push(File::new(file_path));
    }

    let type_skeletons = IndexVec::from_vec(type_skeletons);
    let meta_skeletons = IndexVec::from_vec(meta_skeletons);

    // Convert skeletons into schema
    parse(type_names, type_skeletons, meta_names, meta_skeletons, files, codegen)
}

/// Wrapper to assert a type is safe to send across threads.
///
/// `syn` types are `!Send` because `proc_macro2::Span` contains an `Rc` internally
/// (when the `proc-macro` feature is enabled on `proc-macro2`).
///
/// This crate does not enable the `proc-macro` feature on `syn` crate, which would usually make `syn` types `Send`.
/// But unfortunately it gets enabled by transitive dependencies (`serde_derive`, `bpaf_derive`, etc),
/// due to feature unification.
///
/// `Span` is embedded throughout the syn AST - in every `Ident`, token, `Type`, `Expr`, and `Attribute` -
/// so there's no way to extract the data we need without it.
///
/// # Why this is sound
///
/// `Rc` is `!Send` because two `Rc`s pointing to the same allocation could be used concurrently from different threads,
/// violating the non-atomic reference count. But that's only a problem if an `Rc` has been *cloned*. A sole owner of
/// an `Rc` is safe to send - there's no other `Rc` to race with.
///
/// `syn::parse_file` parses a `&str` and returns a self-contained AST. The `Span`s in this tree are created fresh
/// by `proc_macro2` and are not clones of any external `Rc`. So each parsed tree is the sole owner of all its `Rc`s,
/// and sending it to another thread cannot cause a data race.
struct AssertSend<T>(T);

// SAFETY: See above
unsafe impl<T> Send for AssertSend<T> {}
