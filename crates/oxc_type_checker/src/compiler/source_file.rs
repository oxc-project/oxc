//! A single parsed + semantically-analyzed source file, and the [`FileId`] used to reference
//! it within a [`Program`](super::program::Program).
//!
//! Corresponds to typescript-go's `ast.SourceFile` (`internal/ast/ast.go`) together with its
//! `SourceFileParseOptions` (`internal/ast/parseoptions.go`). tsgo stores the parsed AST
//! directly and keys files by their normalized `tspath.Path`; here a [`SourceFile`] bundles
//! the arena, the parsed program, and the semantic model, and is referenced by a typed
//! [`FileId`] index.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_diagnostics::Diagnostics;
use oxc_index::define_nonmax_u32_index_type;
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;
use self_cell::self_cell;

define_nonmax_u32_index_type! {
    /// Index of a [`SourceFile`] within a [`Program`](super::program::Program)'s file list.
    ///
    /// typescript-go has no integer file id — it keys files by their normalized
    /// `tspath.Path`. This typed index is an oxc-side addition so files can be referenced by a
    /// cheap `u32` (and, later, declarations by `(FileId, SymbolId)`).
    pub struct FileId;
}

/// Inputs needed to parse a source file, mirroring tsgo's `ast.SourceFileParseOptions`.
#[derive(Debug, Clone)]
pub struct SourceFileParseOptions {
    /// The file's name as it was resolved (absolute, normalized).
    pub file_name: PathBuf,
    /// The normalized path used as the file's identity key (tsgo `tspath.Path`).
    pub path: PathBuf,
}

// A `SourceFile` is self-referential: the parsed `Program` and its `Semantic` model borrow from
// the arena and source text that back them. `self_cell` stores the owner (the arena plus the owned
// source text) alongside the semantic model that borrows from both (dependent). The source text is
// read once and parsed in place — the AST and `Semantic` borrow it directly, with no second copy.
//
// `#[not_covariant]` because `Semantic<'a>` is invariant over `'a` (it becomes invariant once the
// `oxc_semantic` `jsdoc`/`cfg` features are enabled), so the model is reached through a closure
// (`with_semantic`) rather than a borrow that escapes.
self_cell! {
    struct SourceFileCell {
        owner: SourceFileOwner,
        #[not_covariant]
        dependent: Semantic,
    }
}

/// Backing storage a [`SourceFile`]'s AST and [`Semantic`] borrow from: the arena, plus the owned
/// source text they point into.
struct SourceFileOwner {
    allocator: Allocator,
    source_text: String,
}

/// A single parsed and semantically-analyzed source file.
///
/// Corresponds to tsgo's `*ast.SourceFile`. Unlike tsgo — whose `host.GetSourceFile` only
/// parses and defers binding to the checker — a `SourceFile` is eagerly bound here (its
/// [`Semantic`] is built at load time). This is a deliberate, documented divergence.
pub struct SourceFile {
    parse_options: SourceFileParseOptions,
    source_type: SourceType,
    cell: SourceFileCell,
    /// Parse and semantic diagnostics. Owned (they do not borrow the arena), so they can be
    /// read without touching the cell. Not yet rendered — see the crate's diagnostics TODO.
    diagnostics: Diagnostics,
}

impl SourceFile {
    /// Parse and bind `source_text`, mirroring tsgo's `parser.ParseSourceFile` followed by an
    /// (eager) bind. `source_type` selects the JS/TS dialect (derived from the file extension).
    pub(crate) fn parse(
        parse_options: SourceFileParseOptions,
        source_text: String,
        source_type: SourceType,
    ) -> Self {
        let mut diagnostics = Diagnostics::new();
        let owner = SourceFileOwner { allocator: Allocator::default(), source_text };
        let cell = SourceFileCell::new(owner, |owner| {
            let parser_ret = Parser::new(&owner.allocator, &owner.source_text, source_type).parse();
            diagnostics.extend(parser_ret.diagnostics.into_vec());
            // Move the program into the arena so `Semantic` can borrow it for the arena's
            // lifetime (mirrors `oxc_linter`'s `allocator.alloc(ret.program)`).
            let program = owner.allocator.alloc(parser_ret.program);
            let semantic_ret = SemanticBuilder::new().with_check_syntax_error(true).build(program);
            diagnostics.extend(semantic_ret.diagnostics.into_vec());
            semantic_ret.semantic
        });
        Self { parse_options, source_type, cell, diagnostics }
    }

    /// The file's resolved name (absolute, normalized).
    pub fn file_name(&self) -> &Path {
        &self.parse_options.file_name
    }

    /// The file's normalized identity key (tsgo `tspath.Path`).
    pub fn path(&self) -> &Path {
        &self.parse_options.path
    }

    /// The JS/TS dialect the file was parsed as.
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    /// Parse and semantic diagnostics collected for this file.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Run `f` with the file's semantic model (scopes, symbols, references).
    ///
    /// Access is closure-based: `Semantic` borrows the file's arena and is invariant over its
    /// lifetime, so the borrow cannot escape.
    pub fn with_semantic<R>(&self, f: impl FnOnce(&Semantic) -> R) -> R {
        self.cell.with_dependent(|_owner, semantic| f(semantic))
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceFile")
            .field("file_name", &self.parse_options.file_name)
            .field("source_type", &self.source_type)
            .field("diagnostics", &self.diagnostics.len())
            .finish_non_exhaustive()
    }
}
