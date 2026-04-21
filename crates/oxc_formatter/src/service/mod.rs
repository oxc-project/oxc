use oxc_parser::ParseOptions;
use oxc_span::SourceType;
use phf::phf_set;

pub fn get_parse_options() -> ParseOptions {
    ParseOptions {
        // Do not need to parse regexp
        parse_regular_expression: false,
        // Enable all syntax features
        allow_return_outside_function: true,
        allow_v8_intrinsics: true,
        // `oxc_formatter` expects this to be `false`, otherwise panics
        preserve_parens: false,
    }
}

// Additional extensions from linguist-languages, which Prettier also supports
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/JavaScript.js
// No special extensions for TypeScript
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/TypeScript.js
// And on top of this data, Prettier adds its own checks.
// Ultimately, it can be confirmed with the following command.
// `prettier --support-info | jq '.languages[] | select(.name == "JavaScript")'`
static ADDITIONAL_JS_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "_js",
    "bones",
    "es",
    "es6",
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
};

// Special filenames that are valid JS files
static SPECIAL_JS_FILENAMES: phf::Set<&'static str> = phf_set! {
    "Jakefile",
    "start.frag",
    "end.frag",
};

pub fn get_supported_source_type(path: &std::path::Path) -> Option<SourceType> {
    // Standard extensions, also supported by `oxc_span::VALID_EXTENSIONS`
    // NOTE: Use `path` directly for `.d.ts` detection
    if let Ok(source_type) = SourceType::from_path(path) {
        return Some(source_type);
    }

    // Check special filenames first
    if let Some(file_name) = path.file_name()
        && SPECIAL_JS_FILENAMES.contains(file_name.to_str()?)
    {
        return Some(SourceType::default());
    }

    let extension = path.extension()?.to_string_lossy();
    // Additional extensions Prettier also supports
    if ADDITIONAL_JS_EXTENSIONS.contains(extension.as_ref()) {
        return Some(SourceType::default());
    }
    // Special handling for `.frag` files: only allow `*.start.frag` and `*.end.frag`
    if extension == "frag" {
        let stem = path.file_stem()?.to_str()?;
        #[expect(clippy::case_sensitive_file_extension_comparisons)]
        return (stem.ends_with(".start") || stem.ends_with(".end"))
            .then_some(SourceType::default());
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
