pub const JS_IGNORE_TESTS: &[&str] = &[
    // non-standard syntax
    "js/deferred-import-evaluation",
    "js/bind-expressions",
    // Unsupported stage3 features
    "tuple-and-record.js",
    "js/async-do-expressions",
    "js/babel-plugins",
    "js/decorator",
    "js/destructuring-private-fields",
    "js/do", // do expression
    "js/explicit-resource-management",
    "js/export-default/escaped",
    "js/export-default/export-default-from",
    "js/import-assertions",
    "js/import-attributes",
    "js/import-reflection",
    "js/module-blocks",
    "js/multiparser",
    "js/partial-application",
    "js/pipeline-operator",
    "js/record",
    "js/source-phase-imports",
    "js/throw_expressions",
    "js/tuple",
    "js/arrows-bind",
    "js/v8_intrinsic",
    // prettier-ignore
    "js/ignore",
    // range formatting
    "range",
    // IDE cursor
    "cursor",
    // Invalid
    "js/call/invalid",
    "optional-chaining-assignment/invalid-",
];

pub const TS_IGNORE_TESTS: &[&str] = &[];
