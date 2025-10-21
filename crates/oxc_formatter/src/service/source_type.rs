use std::ffi::OsStr;

use oxc_span::SourceType;

// Additional extensions from linguist-languages, which Prettier also supports
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/JavaScript.js
// No special extensions for TypeScript
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/TypeScript.js
const ADDITIONAL_JS_EXTENSIONS: &[&str] = &[
    "_js",
    "bones",
    "es",
    "es6",
    "frag",
    "gs",
    "jake",
    "javascript",
    "jsb",
    "jscad",
    "jsfl",
    "jslib",
    "jsm",
    "jspre",
    "jss",
    "njs",
    "pac",
    "sjs",
    "ssjs",
    "xsjs",
    "xsjslib",
];

pub fn get_supported_source_type(path: &std::path::Path) -> Option<SourceType> {
    // Standard extensions, also supported by `oxc_span::VALID_EXTENSIONS`
    // NOTE: Use `path` directly for `.d.ts` detection
    if let Ok(source_type) = SourceType::from_path(path) {
        return Some(source_type);
    }

    let extension = path.extension()?.to_string_lossy();
    // Additional extensions from linguist-languages, which Prettier also supports
    if ADDITIONAL_JS_EXTENSIONS.contains(&extension.as_ref()) {
        return Some(SourceType::default());
    }
    // `Jakefile` has no extension but is a valid JS file defined by linguist-languages
    if path.file_name() == Some(OsStr::new("Jakefile")) {
        return Some(SourceType::default());
    }

    None
}

#[must_use]
pub fn enable_jsx_source_type(source_type: SourceType) -> SourceType {
    if source_type.is_jsx() {
        return source_type;
    }

    // Always enable JSX for JavaScript files, no syntax conflict
    if source_type.is_javascript() {
        return source_type.with_jsx(true);
    }

    // Prettier uses `regexp.test(source_text)` to detect JSX in TypeScript files.
    // But we don't follow it for now, since it hurts the performance.
    // if source_type.is_typescript() {
    //   // See https://github.com/prettier/prettier/blob/0d1e7abd5037a1fe8fbcf88a4d8cd13ec4d13a78/src/language-js/parse/utils/jsx-regexp.evaluate.js
    // }

    source_type
}
