// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use nonmax::NonMaxU32;

use crate::{
    comment_node::*, module_record::*, number::*, operator::*, reference::*, scope::*, symbol::*,
};

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 0 bytes
    assert!(size_of::<NonMaxU32>() == 4);
    assert!(align_of::<NonMaxU32>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<CommentNodeId>() == 4);
    assert!(align_of::<CommentNodeId>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<NameSpan>() == 40);
    assert!(align_of::<NameSpan>() == 8);
    assert!(offset_of!(NameSpan, name) == 24);
    assert!(offset_of!(NameSpan, span) == 0);

    // Padding: 7 bytes
    assert!(size_of::<ImportEntry>() == 160);
    assert!(align_of::<ImportEntry>() == 8);
    assert!(offset_of!(ImportEntry, statement_span) == 0);
    assert!(offset_of!(ImportEntry, module_request) == 24);
    assert!(offset_of!(ImportEntry, import_name) == 64);
    assert!(offset_of!(ImportEntry, local_name) == 112);
    assert!(offset_of!(ImportEntry, is_type) == 152);

    assert!(size_of::<ImportImportName>() == 48);
    assert!(align_of::<ImportImportName>() == 8);

    // Padding: 7 bytes
    assert!(size_of::<ExportEntry>() == 240);
    assert!(align_of::<ExportEntry>() == 8);
    assert!(offset_of!(ExportEntry, statement_span) == 24);
    assert!(offset_of!(ExportEntry, span) == 0);
    assert!(offset_of!(ExportEntry, module_request) == 48);
    assert!(offset_of!(ExportEntry, import_name) == 88);
    assert!(offset_of!(ExportEntry, export_name) == 136);
    assert!(offset_of!(ExportEntry, local_name) == 184);
    assert!(offset_of!(ExportEntry, is_type) == 232);

    assert!(size_of::<ExportImportName>() == 48);
    assert!(align_of::<ExportImportName>() == 8);

    assert!(size_of::<ExportExportName>() == 48);
    assert!(align_of::<ExportExportName>() == 8);

    assert!(size_of::<ExportLocalName>() == 48);
    assert!(align_of::<ExportLocalName>() == 8);

    // Padding: 0 bytes
    assert!(size_of::<DynamicImport>() == 48);
    assert!(align_of::<DynamicImport>() == 8);
    assert!(offset_of!(DynamicImport, span) == 0);
    assert!(offset_of!(DynamicImport, module_request) == 24);

    assert!(size_of::<NumberBase>() == 1);
    assert!(align_of::<NumberBase>() == 1);

    assert!(size_of::<BigintBase>() == 1);
    assert!(align_of::<BigintBase>() == 1);

    assert!(size_of::<AssignmentOperator>() == 1);
    assert!(align_of::<AssignmentOperator>() == 1);

    assert!(size_of::<BinaryOperator>() == 1);
    assert!(align_of::<BinaryOperator>() == 1);

    assert!(size_of::<LogicalOperator>() == 1);
    assert!(align_of::<LogicalOperator>() == 1);

    assert!(size_of::<UnaryOperator>() == 1);
    assert!(align_of::<UnaryOperator>() == 1);

    assert!(size_of::<UpdateOperator>() == 1);
    assert!(align_of::<UpdateOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ScopeId>() == 4);
    assert!(align_of::<ScopeId>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<SymbolId>() == 4);
    assert!(align_of::<SymbolId>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ReferenceId>() == 4);
    assert!(align_of::<ReferenceId>() == 4);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    // Padding: 0 bytes
    assert!(size_of::<NonMaxU32>() == 4);
    assert!(align_of::<NonMaxU32>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<CommentNodeId>() == 4);
    assert!(align_of::<CommentNodeId>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<NameSpan>() == 32);
    assert!(align_of::<NameSpan>() == 4);
    assert!(offset_of!(NameSpan, name) == 24);
    assert!(offset_of!(NameSpan, span) == 0);

    // Padding: 3 bytes
    assert!(size_of::<ImportEntry>() == 128);
    assert!(align_of::<ImportEntry>() == 4);
    assert!(offset_of!(ImportEntry, statement_span) == 0);
    assert!(offset_of!(ImportEntry, module_request) == 24);
    assert!(offset_of!(ImportEntry, import_name) == 56);
    assert!(offset_of!(ImportEntry, local_name) == 92);
    assert!(offset_of!(ImportEntry, is_type) == 124);

    assert!(size_of::<ImportImportName>() == 36);
    assert!(align_of::<ImportImportName>() == 4);

    // Padding: 3 bytes
    assert!(size_of::<ExportEntry>() == 192);
    assert!(align_of::<ExportEntry>() == 4);
    assert!(offset_of!(ExportEntry, statement_span) == 24);
    assert!(offset_of!(ExportEntry, span) == 0);
    assert!(offset_of!(ExportEntry, module_request) == 48);
    assert!(offset_of!(ExportEntry, import_name) == 80);
    assert!(offset_of!(ExportEntry, export_name) == 116);
    assert!(offset_of!(ExportEntry, local_name) == 152);
    assert!(offset_of!(ExportEntry, is_type) == 188);

    assert!(size_of::<ExportImportName>() == 36);
    assert!(align_of::<ExportImportName>() == 4);

    assert!(size_of::<ExportExportName>() == 36);
    assert!(align_of::<ExportExportName>() == 4);

    assert!(size_of::<ExportLocalName>() == 36);
    assert!(align_of::<ExportLocalName>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<DynamicImport>() == 48);
    assert!(align_of::<DynamicImport>() == 4);
    assert!(offset_of!(DynamicImport, span) == 0);
    assert!(offset_of!(DynamicImport, module_request) == 24);

    assert!(size_of::<NumberBase>() == 1);
    assert!(align_of::<NumberBase>() == 1);

    assert!(size_of::<BigintBase>() == 1);
    assert!(align_of::<BigintBase>() == 1);

    assert!(size_of::<AssignmentOperator>() == 1);
    assert!(align_of::<AssignmentOperator>() == 1);

    assert!(size_of::<BinaryOperator>() == 1);
    assert!(align_of::<BinaryOperator>() == 1);

    assert!(size_of::<LogicalOperator>() == 1);
    assert!(align_of::<LogicalOperator>() == 1);

    assert!(size_of::<UnaryOperator>() == 1);
    assert!(align_of::<UnaryOperator>() == 1);

    assert!(size_of::<UpdateOperator>() == 1);
    assert!(align_of::<UpdateOperator>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ScopeId>() == 4);
    assert!(align_of::<ScopeId>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<SymbolId>() == 4);
    assert!(align_of::<SymbolId>() == 4);

    // Padding: 0 bytes
    assert!(size_of::<ReferenceId>() == 4);
    assert!(align_of::<ReferenceId>() == 4);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
