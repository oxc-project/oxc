use oxc_parser::ParseOptions;
use oxc_span::SourceType;

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
