#![cfg_attr(not(all(target_pointer_width = "64", target_endian = "little")), expect(dead_code))]

use std::sync::Arc;

use rustc_hash::FxHashMap;

use oxc::{
    allocator::{Allocator, FromIn, Vec},
    ast::ast::{Comment, Program},
    diagnostics::{LabeledSpan, NamedSource, OxcDiagnostic, Severity},
    span::{Atom, Span, format_atom},
    syntax::module_record::{DynamicImport, ExportEntry, ImportEntry, ModuleRecord, NameSpan},
};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;

/// The main struct containing all deserializable data in raw transfer.
#[ast]
#[generate_derive(ESTree)]
#[estree(no_type, no_ts_def)]
pub struct RawTransferData<'a> {
    pub program: Program<'a>,
    pub comments: Vec<'a, Comment>,
    pub module: EcmaScriptModule<'a>,
    pub errors: Vec<'a, Error<'a>>,
}

/// Metadata written to end of buffer.
///
/// Duplicated as `RawTransferMetadata2` in `crates/oxc_linter/src/lib.rs`.
/// Any changes made here also need to be made there.
/// `oxc_ast_tools` checks that the 2 copies are identical.
#[ast]
pub struct RawTransferMetadata {
    /// Offset of `RawTransferData` within buffer.
    pub data_offset: u32,
    /// `true` if AST is TypeScript.
    pub is_ts: bool,
    /// Padding to pad struct to size 16.
    pub(crate) _padding: u64,
}

impl RawTransferMetadata {
    pub fn new(data_offset: u32, is_ts: bool) -> Self {
        Self { data_offset, is_ts, _padding: 0 }
    }
}

// Errors.
//
// These types and the `From` / `FromIn` impls mirror the implementation in `types.rs`
// and `crates/oxc_napi/src/lib.rs`.
// Only difference is that these versions of the types are arena-allocated.

#[ast]
#[generate_derive(ESTree)]
#[estree(no_type, no_ts_def)]
pub struct Error<'a> {
    pub severity: ErrorSeverity,
    pub message: Atom<'a>,
    pub labels: Vec<'a, ErrorLabel<'a>>,
    pub help_message: Option<Atom<'a>>,
    pub codeframe: Atom<'a>,
}

impl<'a> Error<'a> {
    pub(crate) fn from_diagnostics_in(
        diagnostics: impl IntoIterator<Item = OxcDiagnostic>,
        source_text: &str,
        filename: &str,
        allocator: &'a Allocator,
    ) -> Vec<'a, Self> {
        let named_source = Arc::new(NamedSource::new(filename, source_text.to_string()));

        Vec::from_iter_in(
            diagnostics
                .into_iter()
                .map(|diagnostic| Self::from_diagnostic_in(diagnostic, &named_source, allocator)),
            allocator,
        )
    }

    fn from_diagnostic_in(
        diagnostic: OxcDiagnostic,
        named_source: &Arc<NamedSource<String>>,
        allocator: &'a Allocator,
    ) -> Self {
        let labels = diagnostic.labels.as_ref().map_or_else(
            || Vec::new_in(allocator),
            |labels| {
                Vec::from_iter_in(
                    labels.iter().map(|label| ErrorLabel::from_in(label, allocator)),
                    allocator,
                )
            },
        );

        let severity = ErrorSeverity::from(diagnostic.severity);
        let message = Atom::from_in(diagnostic.message.as_ref(), allocator);
        let help_message =
            diagnostic.help.as_ref().map(|help| Atom::from_in(help.as_ref(), allocator));
        let report = diagnostic.with_source_code(Arc::clone(named_source));
        let codeframe = format_atom!(allocator, "{report:?}");

        #[expect(clippy::inconsistent_struct_constructor)] // `#[ast]` macro re-orders struct fields
        Self { severity, message, labels, help_message, codeframe }
    }
}

#[ast]
#[derive(Clone, Copy)]
#[generate_derive(ESTree)]
#[estree(no_rename_variants, no_ts_def)]
pub enum ErrorSeverity {
    Error = 0,
    Warning = 1,
    Advice = 2,
}

impl From<Severity> for ErrorSeverity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Advice => Self::Advice,
        }
    }
}

#[ast]
#[generate_derive(ESTree)]
#[estree(no_type, no_ts_def)]
pub struct ErrorLabel<'a> {
    pub message: Option<Atom<'a>>,
    pub span: Span,
}

impl<'a> FromIn<'a, &LabeledSpan> for ErrorLabel<'a> {
    fn from_in(label: &LabeledSpan, allocator: &'a Allocator) -> Self {
        Self {
            message: label.label().map(|message| Atom::from_in(message, allocator)),
            #[expect(clippy::cast_possible_truncation)]
            span: Span::sized(label.offset() as u32, label.len() as u32),
        }
    }
}

// Module record.
//
// These types and the `From` impl mirror the implementation in `types.rs` and `convert.rs`.
// However, a lot of the data can be left as is, because it's already in the arena,
// and that's what raw transfer needs - unlike the other implementation which requires owned types.
// In particular, there's no need to copy or convert the many `Atom`s in `ModuleRecord`.

#[ast]
#[generate_derive(ESTree)]
#[estree(no_type, no_ts_def)]
pub struct EcmaScriptModule<'a> {
    /// Has ESM syntax.
    ///
    /// i.e. `import` and `export` statements, and `import.meta`.
    ///
    /// Dynamic imports `import('foo')` are ignored since they can be used in non-ESM files.
    pub has_module_syntax: bool,
    /// Import statements.
    pub static_imports: Vec<'a, StaticImport<'a>>,
    /// Export statements.
    pub static_exports: Vec<'a, StaticExport<'a>>,
    /// Dynamic import expressions.
    pub dynamic_imports: Vec<'a, DynamicImport>,
    /// Span positions` of `import.meta`
    pub import_metas: Vec<'a, Span>,
}

#[ast]
#[generate_derive(ESTree)]
#[estree(no_type, no_ts_def)]
pub struct StaticImport<'a> {
    /// Span of import statement.
    pub span: Span,
    /// Import source.
    ///
    /// ```js
    /// import { foo } from "mod";
    /// //                   ^^^
    /// ```
    pub module_request: NameSpan<'a>,
    /// Import specifiers.
    ///
    /// Empty for `import "mod"`.
    pub entries: Vec<'a, ImportEntry<'a>>,
}

#[ast]
#[generate_derive(ESTree)]
#[estree(no_type, no_ts_def)]
pub struct StaticExport<'a> {
    pub span: Span,
    pub entries: Vec<'a, ExportEntry<'a>>,
}

impl<'a> FromIn<'a, ModuleRecord<'a>> for EcmaScriptModule<'a> {
    fn from_in(record: ModuleRecord<'a>, allocator: &'a Allocator) -> Self {
        let static_imports =
            record.requested_modules.iter().flat_map(|(name, requested_modules)| {
                requested_modules.iter().filter(|m| m.is_import).map(|m| {
                    let entries = record
                        .import_entries
                        .iter()
                        .filter(|e| e.statement_span == m.statement_span)
                        .cloned();
                    let entries = Vec::from_iter_in(entries, allocator);

                    StaticImport {
                        span: m.statement_span,
                        module_request: NameSpan { name: *name, span: m.span },
                        entries,
                    }
                })
            });
        let mut static_imports = Vec::from_iter_in(static_imports, allocator);
        static_imports.sort_unstable_by_key(|e| e.span.start);

        let static_exports = record
            .local_export_entries
            .iter()
            .chain(&record.indirect_export_entries)
            .chain(&record.star_export_entries)
            .fold(FxHashMap::<Span, Vec<'a, ExportEntry>>::default(), |mut acc, e| {
                acc.entry(e.statement_span)
                    .or_insert_with(|| Vec::new_in(allocator))
                    .push(e.clone());
                acc
            })
            .into_iter()
            .map(|(span, entries)| StaticExport { span, entries });
        let mut static_exports = Vec::from_iter_in(static_exports, allocator);
        static_exports.sort_unstable_by_key(|e| e.span.start);

        Self {
            has_module_syntax: record.has_module_syntax,
            static_imports,
            static_exports,
            dynamic_imports: record.dynamic_imports,
            import_metas: record.import_metas,
        }
    }
}
