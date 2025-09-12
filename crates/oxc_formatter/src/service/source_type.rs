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
    let extension = path.extension()?.to_string_lossy();

    // Standard extensions, also supported by `oxc_span::VALID_EXTENSIONS`
    if let Ok(source_type) = SourceType::from_extension(&extension) {
        return Some(source_type);
    }
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
