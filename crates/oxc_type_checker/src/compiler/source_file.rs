//! A parsed source file — its arena, AST, and module record — referenced by [`FileId`](super::FileId).
//!
//! Corresponds to typescript-go's `ast.SourceFile` (`internal/ast/ast.go`) + `SourceFileParseOptions`
//! (`internal/ast/parseoptions.go`). A loaded file keeps its parsed AST so the checker can run over
//! it without re-parsing, plus the collected external module references (tsgo `Imports` /
//! `ModuleAugmentations`, see [`references`](super::references)). No `Semantic` is built yet — only
//! the parse output (`Program` + `ModuleRecord`), which is what import discovery needs.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_ast::ast::Program as AstProgram;
use oxc_diagnostics::Diagnostics;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_str::CompactStr;
use oxc_syntax::module_record::ModuleRecord;
use self_cell::self_cell;

use super::references::{ExternalModuleReferences, collect_external_module_references};

/// Inputs to parse a source file, mirroring tsgo's `ast.SourceFileParseOptions`.
#[derive(Debug, Clone)]
pub struct SourceFileParseOptions {
    /// The file's resolved name (absolute, normalized).
    pub file_name: PathBuf,
    /// The normalized path used as the file's identity key (tsgo `tspath.Path`).
    pub path: PathBuf,
}

/// Backing storage the AST + module record borrow from: the arena and the owned source text.
struct SourceFileOwner {
    allocator: Allocator,
    source_text: String,
}

// A `SourceFile` is self-referential: the parsed `Program`/`ModuleRecord` borrow from the arena and
// source text. `self_cell` stores the owner (arena + text) alongside the parse output that borrows
// from it. Mirrors `oxc_linter`'s `ModuleContent`.
self_cell! {
    struct SourceFileCell {
        owner: SourceFileOwner,
        #[covariant]
        dependent: SourceFileData,
    }
}

struct SourceFileData<'a> {
    program: AstProgram<'a>,
    module_record: ModuleRecord<'a>,
}

// SAFETY: `SourceFileCell` owns the arena (inside `SourceFileOwner`) together with the `Program` and
// `ModuleRecord` that borrow from it, with no outside borrows. Moving the cell moves the arena with
// its dependents, so the arena references stay valid across threads. This lets a parsed file be sent
// between rayon workers and the graph thread (mirrors `oxc_linter`'s `ModuleContent`).
unsafe impl Send for SourceFileCell {}

/// A single parsed source file.
///
/// Corresponds to tsgo's `*ast.SourceFile`. It keeps its arena-backed AST (`program`), import data
/// (`module_record`), and the module specifiers collected for resolution (tsgo `Imports` /
/// `ModuleAugmentations`). The resolved module-graph edges live on the program's
/// `ProcessedFiles`, not the file (as in tsgo).
pub struct SourceFile {
    parse_options: SourceFileParseOptions,
    source_type: SourceType,
    cell: SourceFileCell,
    /// Parse diagnostics. Owned (they do not borrow the arena). Not yet rendered.
    diagnostics: Diagnostics,
    /// The file's external module references (tsgo `SourceFile.Imports` + `ModuleAugmentations`).
    references: ExternalModuleReferences,
}

impl SourceFile {
    /// Parse `source_text`, mirroring tsgo's `parser.ParseSourceFile`: parse, then collect the
    /// file's external module references. `source_type` selects the JS/TS dialect (derived from
    /// the file extension).
    pub(crate) fn parse(
        parse_options: SourceFileParseOptions,
        source_text: String,
        source_type: SourceType,
    ) -> Self {
        let mut diagnostics = Diagnostics::new();
        let owner = SourceFileOwner { allocator: Allocator::default(), source_text };
        let cell = SourceFileCell::new(owner, |owner| {
            let ret = Parser::new(&owner.allocator, &owner.source_text, source_type).parse();
            diagnostics.extend(ret.diagnostics.into_vec());
            SourceFileData { program: ret.program, module_record: ret.module_record }
        });
        let references = {
            let data = cell.borrow_dependent();
            collect_external_module_references(
                &data.program,
                &data.module_record,
                source_type.is_typescript_definition(),
            )
        };
        Self { parse_options, source_type, cell, diagnostics, references }
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

    /// Parse diagnostics collected for this file.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// The parsed AST.
    pub fn program(&self) -> &AstProgram<'_> {
        &self.cell.borrow_dependent().program
    }

    /// The file's module record (imports/exports).
    pub fn module_record(&self) -> &ModuleRecord<'_> {
        &self.cell.borrow_dependent().module_record
    }

    /// The module specifiers this file imports, in tsgo's `SourceFile.Imports` order: static
    /// imports/re-exports in source order, then dynamic `import()`s and `import("...")` type
    /// queries in source order.
    pub fn imports(&self) -> &[CompactStr] {
        &self.references.imports
    }

    /// String-literal `declare module "..."` names that augment an existing external module
    /// (tsgo `SourceFile.ModuleAugmentations`).
    pub fn module_augmentations(&self) -> &[CompactStr] {
        &self.references.module_augmentations
    }

    /// `/// <reference path="..." />` pragmas (tsgo `SourceFile.ReferencedFiles`).
    pub fn referenced_files(&self) -> &[CompactStr] {
        &self.references.referenced_files
    }

    /// `/// <reference types="..." />` pragmas (tsgo `SourceFile.TypeReferenceDirectives`).
    pub fn type_reference_directives(&self) -> &[CompactStr] {
        &self.references.type_reference_directives
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceFile")
            .field("file_name", &self.parse_options.file_name)
            .field("source_type", &self.source_type)
            .field("diagnostics", &self.diagnostics.len())
            .field("imports", &self.references.imports.len())
            .finish_non_exhaustive()
    }
}
