// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::raw_transfer_types::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<RawTransferData>() == 360);
    assert!(align_of::<RawTransferData>() == 8);
    assert!(offset_of!(RawTransferData, program) == 0);
    assert!(offset_of!(RawTransferData, comments) == 160);
    assert!(offset_of!(RawTransferData, module) == 192);
    assert!(offset_of!(RawTransferData, errors) == 328);

    assert!(size_of::<Error>() == 72);
    assert!(align_of::<Error>() == 8);
    assert!(offset_of!(Error, severity) == 0);
    assert!(offset_of!(Error, message) == 8);
    assert!(offset_of!(Error, labels) == 24);
    assert!(offset_of!(Error, help_message) == 56);

    assert!(size_of::<ErrorSeverity>() == 1);
    assert!(align_of::<ErrorSeverity>() == 1);

    assert!(size_of::<ErrorLabel>() == 24);
    assert!(align_of::<ErrorLabel>() == 8);
    assert!(offset_of!(ErrorLabel, message) == 0);
    assert!(offset_of!(ErrorLabel, span) == 16);

    assert!(size_of::<EcmaScriptModule>() == 136);
    assert!(align_of::<EcmaScriptModule>() == 8);
    assert!(offset_of!(EcmaScriptModule, has_module_syntax) == 0);
    assert!(offset_of!(EcmaScriptModule, static_imports) == 8);
    assert!(offset_of!(EcmaScriptModule, static_exports) == 40);
    assert!(offset_of!(EcmaScriptModule, dynamic_imports) == 72);
    assert!(offset_of!(EcmaScriptModule, import_metas) == 104);

    assert!(size_of::<StaticImport>() == 64);
    assert!(align_of::<StaticImport>() == 8);
    assert!(offset_of!(StaticImport, span) == 0);
    assert!(offset_of!(StaticImport, module_request) == 8);
    assert!(offset_of!(StaticImport, entries) == 32);

    assert!(size_of::<StaticExport>() == 40);
    assert!(align_of::<StaticExport>() == 8);
    assert!(offset_of!(StaticExport, span) == 0);
    assert!(offset_of!(StaticExport, entries) == 8);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(size_of::<RawTransferData>() == 188);
    assert!(align_of::<RawTransferData>() == 4);
    assert!(offset_of!(RawTransferData, program) == 0);
    assert!(offset_of!(RawTransferData, comments) == 88);
    assert!(offset_of!(RawTransferData, module) == 104);
    assert!(offset_of!(RawTransferData, errors) == 172);

    assert!(size_of::<Error>() == 36);
    assert!(align_of::<Error>() == 4);
    assert!(offset_of!(Error, severity) == 0);
    assert!(offset_of!(Error, message) == 4);
    assert!(offset_of!(Error, labels) == 12);
    assert!(offset_of!(Error, help_message) == 28);

    assert!(size_of::<ErrorSeverity>() == 1);
    assert!(align_of::<ErrorSeverity>() == 1);

    assert!(size_of::<ErrorLabel>() == 16);
    assert!(align_of::<ErrorLabel>() == 4);
    assert!(offset_of!(ErrorLabel, message) == 0);
    assert!(offset_of!(ErrorLabel, span) == 8);

    assert!(size_of::<EcmaScriptModule>() == 68);
    assert!(align_of::<EcmaScriptModule>() == 4);
    assert!(offset_of!(EcmaScriptModule, has_module_syntax) == 0);
    assert!(offset_of!(EcmaScriptModule, static_imports) == 4);
    assert!(offset_of!(EcmaScriptModule, static_exports) == 20);
    assert!(offset_of!(EcmaScriptModule, dynamic_imports) == 36);
    assert!(offset_of!(EcmaScriptModule, import_metas) == 52);

    assert!(size_of::<StaticImport>() == 40);
    assert!(align_of::<StaticImport>() == 4);
    assert!(offset_of!(StaticImport, span) == 0);
    assert!(offset_of!(StaticImport, module_request) == 8);
    assert!(offset_of!(StaticImport, entries) == 24);

    assert!(size_of::<StaticExport>() == 24);
    assert!(align_of::<StaticExport>() == 4);
    assert!(offset_of!(StaticExport, span) == 0);
    assert!(offset_of!(StaticExport, entries) == 8);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
