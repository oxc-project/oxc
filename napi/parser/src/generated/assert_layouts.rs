// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/assert_layouts.rs`.

#![allow(unused_imports)]

use std::mem::{align_of, offset_of, size_of};

use crate::raw_transfer_types::*;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<RawTransferData>() == 288);
    assert!(align_of::<RawTransferData>() == 8);
    assert!(offset_of!(RawTransferData, program) == 0);
    assert!(offset_of!(RawTransferData, comments) == 136);
    assert!(offset_of!(RawTransferData, module) == 160);
    assert!(offset_of!(RawTransferData, errors) == 264);

    assert!(size_of::<Error>() == 64);
    assert!(align_of::<Error>() == 8);
    assert!(offset_of!(Error, severity) == 0);
    assert!(offset_of!(Error, message) == 8);
    assert!(offset_of!(Error, labels) == 24);
    assert!(offset_of!(Error, help_message) == 48);

    assert!(size_of::<ErrorSeverity>() == 1);
    assert!(align_of::<ErrorSeverity>() == 1);

    assert!(size_of::<ErrorLabel>() == 24);
    assert!(align_of::<ErrorLabel>() == 8);
    assert!(offset_of!(ErrorLabel, message) == 0);
    assert!(offset_of!(ErrorLabel, span) == 16);

    assert!(size_of::<EcmaScriptModule>() == 104);
    assert!(align_of::<EcmaScriptModule>() == 8);
    assert!(offset_of!(EcmaScriptModule, has_module_syntax) == 0);
    assert!(offset_of!(EcmaScriptModule, static_imports) == 8);
    assert!(offset_of!(EcmaScriptModule, static_exports) == 32);
    assert!(offset_of!(EcmaScriptModule, dynamic_imports) == 56);
    assert!(offset_of!(EcmaScriptModule, import_metas) == 80);

    assert!(size_of::<StaticImport>() == 56);
    assert!(align_of::<StaticImport>() == 8);
    assert!(offset_of!(StaticImport, span) == 0);
    assert!(offset_of!(StaticImport, module_request) == 8);
    assert!(offset_of!(StaticImport, entries) == 32);

    assert!(size_of::<StaticExport>() == 32);
    assert!(align_of::<StaticExport>() == 8);
    assert!(offset_of!(StaticExport, span) == 0);
    assert!(offset_of!(StaticExport, entries) == 8);
};

#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(size_of::<RawTransferData>() == 260);
    assert!(align_of::<RawTransferData>() == 4);
    assert!(offset_of!(RawTransferData, program) == 0);
    assert!(offset_of!(RawTransferData, comments) == 112);
    assert!(offset_of!(RawTransferData, module) == 136);
    assert!(offset_of!(RawTransferData, errors) == 236);

    assert!(size_of::<Error>() == 44);
    assert!(align_of::<Error>() == 4);
    assert!(offset_of!(Error, severity) == 0);
    assert!(offset_of!(Error, message) == 4);
    assert!(offset_of!(Error, labels) == 12);
    assert!(offset_of!(Error, help_message) == 36);

    assert!(size_of::<ErrorSeverity>() == 1);
    assert!(align_of::<ErrorSeverity>() == 1);

    assert!(size_of::<ErrorLabel>() == 16);
    assert!(align_of::<ErrorLabel>() == 4);
    assert!(offset_of!(ErrorLabel, message) == 0);
    assert!(offset_of!(ErrorLabel, span) == 8);

    assert!(size_of::<EcmaScriptModule>() == 100);
    assert!(align_of::<EcmaScriptModule>() == 4);
    assert!(offset_of!(EcmaScriptModule, has_module_syntax) == 0);
    assert!(offset_of!(EcmaScriptModule, static_imports) == 4);
    assert!(offset_of!(EcmaScriptModule, static_exports) == 28);
    assert!(offset_of!(EcmaScriptModule, dynamic_imports) == 52);
    assert!(offset_of!(EcmaScriptModule, import_metas) == 76);

    assert!(size_of::<StaticImport>() == 48);
    assert!(align_of::<StaticImport>() == 4);
    assert!(offset_of!(StaticImport, span) == 0);
    assert!(offset_of!(StaticImport, module_request) == 8);
    assert!(offset_of!(StaticImport, entries) == 24);

    assert!(size_of::<StaticExport>() == 32);
    assert!(align_of::<StaticExport>() == 4);
    assert!(offset_of!(StaticExport, span) == 0);
    assert!(offset_of!(StaticExport, entries) == 8);
};

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
