// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::raw_transfer_types::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    // Padding: 0 bytes
    assert!(size_of::<RawTransferData>() == 296);
    assert!(align_of::<RawTransferData>() == 8);
    assert!(offset_of!(RawTransferData, program) == 0);
    assert!(offset_of!(RawTransferData, comments) == 144);
    assert!(offset_of!(RawTransferData, module) == 168);
    assert!(offset_of!(RawTransferData, errors) == 272);

    // Padding: 2 bytes
    assert!(size_of::<RawTransferMetadata>() == 16);
    assert!(align_of::<RawTransferMetadata>() == 8);
    assert!(offset_of!(RawTransferMetadata, data_offset) == 8);
    assert!(offset_of!(RawTransferMetadata, is_ts) == 12);
    assert!(offset_of!(RawTransferMetadata, is_jsx) == 13);
    assert!(offset_of!(RawTransferMetadata, _padding) == 0);

    // Padding: 7 bytes
    assert!(size_of::<Error>() == 80);
    assert!(align_of::<Error>() == 8);
    assert!(offset_of!(Error, severity) == 72);
    assert!(offset_of!(Error, message) == 0);
    assert!(offset_of!(Error, labels) == 16);
    assert!(offset_of!(Error, help_message) == 40);
    assert!(offset_of!(Error, codeframe) == 56);

    assert!(size_of::<ErrorSeverity>() == 1);
    assert!(align_of::<ErrorSeverity>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ErrorLabel>() == 24);
    assert!(align_of::<ErrorLabel>() == 8);
    assert!(offset_of!(ErrorLabel, message) == 8);
    assert!(offset_of!(ErrorLabel, span) == 0);

    // Padding: 7 bytes
    assert!(size_of::<EcmaScriptModule>() == 104);
    assert!(align_of::<EcmaScriptModule>() == 8);
    assert!(offset_of!(EcmaScriptModule, has_module_syntax) == 96);
    assert!(offset_of!(EcmaScriptModule, static_imports) == 0);
    assert!(offset_of!(EcmaScriptModule, static_exports) == 24);
    assert!(offset_of!(EcmaScriptModule, dynamic_imports) == 48);
    assert!(offset_of!(EcmaScriptModule, import_metas) == 72);

    // Padding: 0 bytes
    assert!(size_of::<StaticImport>() == 56);
    assert!(align_of::<StaticImport>() == 8);
    assert!(offset_of!(StaticImport, span) == 0);
    assert!(offset_of!(StaticImport, module_request) == 8);
    assert!(offset_of!(StaticImport, entries) == 32);

    // Padding: 0 bytes
    assert!(size_of::<StaticExport>() == 32);
    assert!(align_of::<StaticExport>() == 8);
    assert!(offset_of!(StaticExport, span) == 0);
    assert!(offset_of!(StaticExport, entries) == 8);
};

#[cfg(target_pointer_width = "32")]
const _: () = if cfg!(target_family = "wasm") || align_of::<u64>() == 8 {
    // Padding: 0 bytes
    assert!(size_of::<RawTransferData>() == 196);
    assert!(align_of::<RawTransferData>() == 4);
    assert!(offset_of!(RawTransferData, program) == 0);
    assert!(offset_of!(RawTransferData, comments) == 96);
    assert!(offset_of!(RawTransferData, module) == 112);
    assert!(offset_of!(RawTransferData, errors) == 180);

    // Padding: 2 bytes
    assert!(size_of::<RawTransferMetadata>() == 16);
    assert!(align_of::<RawTransferMetadata>() == 8);
    assert!(offset_of!(RawTransferMetadata, data_offset) == 8);
    assert!(offset_of!(RawTransferMetadata, is_ts) == 12);
    assert!(offset_of!(RawTransferMetadata, is_jsx) == 13);
    assert!(offset_of!(RawTransferMetadata, _padding) == 0);

    // Padding: 3 bytes
    assert!(size_of::<Error>() == 44);
    assert!(align_of::<Error>() == 4);
    assert!(offset_of!(Error, severity) == 40);
    assert!(offset_of!(Error, message) == 0);
    assert!(offset_of!(Error, labels) == 8);
    assert!(offset_of!(Error, help_message) == 24);
    assert!(offset_of!(Error, codeframe) == 32);

    assert!(size_of::<ErrorSeverity>() == 1);
    assert!(align_of::<ErrorSeverity>() == 1);

    // Padding: 0 bytes
    assert!(size_of::<ErrorLabel>() == 16);
    assert!(align_of::<ErrorLabel>() == 4);
    assert!(offset_of!(ErrorLabel, message) == 8);
    assert!(offset_of!(ErrorLabel, span) == 0);

    // Padding: 3 bytes
    assert!(size_of::<EcmaScriptModule>() == 68);
    assert!(align_of::<EcmaScriptModule>() == 4);
    assert!(offset_of!(EcmaScriptModule, has_module_syntax) == 64);
    assert!(offset_of!(EcmaScriptModule, static_imports) == 0);
    assert!(offset_of!(EcmaScriptModule, static_exports) == 16);
    assert!(offset_of!(EcmaScriptModule, dynamic_imports) == 32);
    assert!(offset_of!(EcmaScriptModule, import_metas) == 48);

    // Padding: 0 bytes
    assert!(size_of::<StaticImport>() == 40);
    assert!(align_of::<StaticImport>() == 4);
    assert!(offset_of!(StaticImport, span) == 0);
    assert!(offset_of!(StaticImport, module_request) == 8);
    assert!(offset_of!(StaticImport, entries) == 24);

    // Padding: 0 bytes
    assert!(size_of::<StaticExport>() == 24);
    assert!(align_of::<StaticExport>() == 4);
    assert!(offset_of!(StaticExport, span) == 0);
    assert!(offset_of!(StaticExport, entries) == 8);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
