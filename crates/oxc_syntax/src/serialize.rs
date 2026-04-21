use oxc_ast_macros::ast_meta;
use oxc_estree::{ESTree, Serializer};
use oxc_span::Span;

use crate::module_record::NameSpan;

/// Macro to create a dummy `ESTree` impl for a type.
///
/// The meta types in this module are only used for raw transfer,
/// so no need for real `ESTree` impls.
macro_rules! dummy_estree_impl {
    ($ty:ty) => {
        impl ESTree for $ty {
            fn serialize<S: Serializer>(&self, _serializer: S) {
                unimplemented!();
            }
        }
    };
}

/// Serializer for `Name` variant of `ExportLocalName`, `ExportExportName`, `ExportImportName`,
/// and `ImportImportName`.
#[ast_meta]
#[estree(
    ts_type = "Dummy",
    raw_deser = "
        var nameSpan = DESER[NameSpan](POS);
        { kind: 'Name', name: nameSpan.value, start: nameSpan.start, end: nameSpan.end, ...(RANGE && { range: nameSpan.range }) }
    "
)]
pub struct ImportOrExportNameName<'a, 'b>(#[expect(dead_code)] pub &'b NameSpan<'a>);

dummy_estree_impl!(ImportOrExportNameName<'_, '_>);

/// Serializer for `Default` variant of `ExportLocalName`.
#[ast_meta]
#[estree(
    ts_type = "Dummy",
    raw_deser = "
        var nameSpan = DESER[NameSpan](POS);
        { kind: 'Default', name: nameSpan.value, start: nameSpan.start, end: nameSpan.end, ...(RANGE && { range: nameSpan.range }) }
    "
)]
pub struct ExportLocalNameDefault<'a, 'b>(#[expect(dead_code)] pub &'b NameSpan<'a>);

dummy_estree_impl!(ExportLocalNameDefault<'_, '_>);

/// Serializer for `Null` variant of `ExportLocalName`, `ExportExportName`, and `ExportImportName`.
#[ast_meta]
#[estree(
    ts_type = "Dummy",
    raw_deser = "{ kind: 'None', name: null, start: null, end: null, ...(RANGE && { range: [null, null] }) }"
)]
pub struct ExportNameNull(pub ());

dummy_estree_impl!(ExportNameNull);

/// Serializer for `Default` variant of `ExportExportName` and `ImportImportName`.
#[ast_meta]
#[estree(
    ts_type = "Dummy",
    raw_deser = "
        var { start, end } = DESER[Span](POS);
        { kind: 'Default', name: null, start, end, ...(RANGE && { range: [start, end] }) }
    "
)]
pub struct ImportOrExportNameDefault<'b>(#[expect(dead_code)] pub &'b Span);

dummy_estree_impl!(ImportOrExportNameDefault<'_>);

/// Serializer for `All` variant of `ExportImportName`.
#[ast_meta]
#[estree(
    ts_type = "Dummy",
    raw_deser = "{ kind: 'All', name: null, start: null, end: null, ...(RANGE && { range: [null, null] }) }"
)]
pub struct ExportImportNameAll(pub ());

dummy_estree_impl!(ExportImportNameAll);

/// Serializer for `AllButDefault` variant of `ExportImportName`.
#[ast_meta]
#[estree(
    ts_type = "Dummy",
    raw_deser = "{ kind: 'AllButDefault', name: null, start: null, end: null, ...(RANGE && { range: [null, null] }) }"
)]
pub struct ExportImportNameAllButDefault(pub ());

dummy_estree_impl!(ExportImportNameAllButDefault);

/// Serializer for `NamespaceObject` variant of `ImportImportName`.
#[ast_meta]
#[estree(
    ts_type = "Dummy",
    raw_deser = "{ kind: 'NamespaceObject', name: null, start: null, end: null, ...(RANGE && { range: [null, null] }) }"
)]
pub struct ImportImportNameNamespaceObject(pub ());

dummy_estree_impl!(ImportImportNameNamespaceObject);
