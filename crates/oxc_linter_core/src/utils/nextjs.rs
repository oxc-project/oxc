use crate::LintContext;

// Keep in sync with next.js polyfills file : https://github.com/vercel/next.js/blob/v15.0.2/packages/next-polyfill-nomodule/src/index.js
pub const NEXT_POLYFILLED_FEATURES: phf::Set<&'static str> = phf::phf_set![
    "Array.from",
    "Array.of",
    "Array.prototype.@@iterator",
    "Array.prototype.at",
    "Array.prototype.copyWithin",
    "Array.prototype.fill",
    "Array.prototype.find",
    "Array.prototype.findIndex",
    "Array.prototype.flat",
    "Array.prototype.flatMap",
    "Array.prototype.includes",
    "Function.prototype.name",
    "Map",
    "Number.EPSILON",
    "Number.Epsilon",
    "Number.MAX_SAFE_INTEGER",
    "Number.MIN_SAFE_INTEGER",
    "Number.isFinite",
    "Number.isInteger",
    "Number.isNaN",
    "Number.isSafeInteger",
    "Number.parseFloat",
    "Number.parseInt",
    "Object.assign",
    "Object.entries",
    "Object.fromEntries",
    "Object.getOwnPropertyDescriptor",
    "Object.getOwnPropertyDescriptors",
    "Object.is",
    "Object.keys",
    "Object.values",
    "Promise",
    "Promise.prototype.finally",
    "Reflect",
    "Set",
    "String.fromCodePoint",
    "String.prototype.@@iterator",
    "String.prototype.codePointAt",
    "String.prototype.endsWith",
    "String.prototype.includes",
    "String.prototype.padEnd",
    "String.prototype.padStart",
    "String.prototype.repeat",
    "String.prototype.startsWith",
    "String.prototype.trimEnd",
    "String.prototype.trimStart",
    "String.raw",
    "Symbol",
    "Symbol.asyncIterator",
    "URL",
    "URL.prototype.toJSON",
    "URLSearchParams",
    "WeakMap",
    "WeakSet",
    "es2015", // Should be covered by babel-preset-env instead.
    "es2016", // contains polyfilled 'Array.prototype.includes', 'String.prototype.padEnd' and 'String.prototype.padStart'
    "es2017", // contains polyfilled 'Object.entries', 'Object.getOwnPropertyDescriptors', 'Object.values', 'String.prototype.padEnd' and 'String.prototype.padStart'
    "es2018", // contains polyfilled 'Promise.prototype.finally' and ''Symbol.asyncIterator'
    "es2019", // Contains polyfilled 'Object.fromEntries' and polyfilled 'Array.prototype.flat', 'Array.prototype.flatMap', 'String.prototype.trimEnd' and 'String.prototype.trimStart'
    "es5",    // Should be covered by babel-preset-env instead.
    "es6",    // Should be covered by babel-preset-env instead.
    "es7", // contains polyfilled 'Array.prototype.includes', 'String.prototype.padEnd' and 'String.prototype.padStart'
    "fetch",
];

pub fn is_in_app_dir(file_path: &str) -> bool {
    file_path.contains("app/") || file_path.contains("app\\")
}

pub fn is_document_page(file_path: &str) -> bool {
    let Some(page) = file_path.split("pages").last() else {
        return false;
    };
    page.starts_with("/_document") || page.starts_with("\\_document")
}

pub fn get_next_script_import_local_name<'a>(ctx: &'a LintContext) -> Option<&'a str> {
    ctx.module_record().import_entries.iter().find_map(|entry| {
        if entry.module_request.name() == "next/script" {
            Some(entry.local_name.name())
        } else {
            None
        }
    })
}
