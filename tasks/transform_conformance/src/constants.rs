pub const PLUGINS: &[&str] = &[
    "babel-preset-env",
    // // ES2024
    // "babel-plugin-transform-unicode-sets-regex",
    // // ES2022
    "babel-plugin-transform-class-properties",
    "babel-plugin-transform-class-static-block",
    "babel-plugin-transform-private-methods",
    "babel-plugin-transform-private-property-in-object",
    // // [Syntax] "babel-plugin-transform-syntax-top-level-await",
    // ES2021
    "babel-plugin-transform-logical-assignment-operators",
    // "babel-plugin-transform-numeric-separator",
    // ES2020
    // "babel-plugin-transform-export-namespace-from",
    // "babel-plugin-transform-dynamic-import",
    "babel-plugin-transform-nullish-coalescing-operator",
    "babel-plugin-transform-optional-chaining",
    // // [Syntax] "babel-plugin-transform-syntax-bigint",
    // // [Syntax] "babel-plugin-transform-syntax-dynamic-import",
    // // [Syntax] "babel-plugin-transform-syntax-import-meta",
    // ES2019
    "babel-plugin-transform-optional-catch-binding",
    // "babel-plugin-transform-json-strings",
    // // ES2018
    "babel-plugin-transform-async-generator-functions",
    "babel-plugin-transform-object-rest-spread",
    // // [Regex] "babel-plugin-transform-unicode-property-regex",
    // "babel-plugin-transform-dotall-regex",
    // // [Regex] "babel-plugin-transform-named-capturing-groups-regex",
    // // ES2017
    "babel-plugin-transform-async-to-generator",
    // ES2016
    "babel-plugin-transform-exponentiation-operator",
    // ES2015
    "babel-plugin-transform-arrow-functions",
    // "babel-plugin-transform-function-name",
    // "babel-plugin-transform-shorthand-properties",
    // "babel-plugin-transform-sticky-regex",
    // "babel-plugin-transform-unicode-regex",
    // "babel-plugin-transform-template-literals",
    // "babel-plugin-transform-duplicate-keys",
    // "babel-plugin-transform-instanceof",
    // "babel-plugin-transform-new-target",
    // // ES3
    // "babel-plugin-transform-property-literals",
    // TypeScript
    "babel-preset-typescript",
    "babel-plugin-transform-typescript",
    // React
    "babel-preset-react",
    "babel-plugin-transform-react-jsx",
    "babel-plugin-transform-react-display-name",
    "babel-plugin-transform-react-jsx-self",
    "babel-plugin-transform-react-jsx-source",
    "babel-plugin-transform-react-jsx-development",
    // // Proposal
    // "babel-plugin-proposal-decorators",

    // RegExp tests ported from esbuild + a few additions
    "regexp",
];

pub const PLUGINS_NOT_SUPPORTED_YET: &[&str] = &[
    "proposal-decorators",
    "transform-classes",
    "transform-destructuring",
    "transform-modules-commonjs",
    "transform-parameters",
    "transform-property-literals",
    "transform-react-constant-elements",
];

pub const SKIP_TESTS: &[&str] = &[
    // Shouldn't report in transformer
    "babel-plugin-transform-typescript/test/fixtures/node-extensions/type-assertion-in-cts",
    "babel-plugin-transform-typescript/test/fixtures/node-extensions/type-assertion-in-mts",
    "babel-plugin-transform-typescript/test/fixtures/node-extensions/type-param-arrow-in-cts",
    "babel-plugin-transform-typescript/test/fixtures/node-extensions/type-param-arrow-in-mts",
    "babel-plugin-transform-typescript/test/fixtures/node-extensions/with-in-mts",
    // Report error for deprecate option or oxc doesn’t follow error message
    "babel-plugin-transform-typescript/test/fixtures/opts/allowDeclareFields",
    "babel-plugin-transform-react-jsx/test/fixtures/react-automatic/should-throw-when-filter-is-specified",
    // Not standard JavaScript or typescript syntax
    "babel-plugin-transform-typescript/test/fixtures/exports/export-type-star-from",
    // The output is valid and semantically correct
    // but does not match Babel's expected output
    "babel-plugin-transform-typescript/test/fixtures/namespace/canonical",
    "babel-plugin-transform-typescript/test/fixtures/namespace/nested-shorthand-export",
    "babel-plugin-transform-react-jsx-development/test/fixtures/cross-platform/self-inside-arrow",
    // Babel outputs is not correct
    "babel-plugin-transform-typescript/test/fixtures/namespace/clobber-import",
    "babel-plugin-transform-typescript/test/fixtures/namespace/namespace-nested-module",
    "babel-plugin-transform-typescript/test/fixtures/namespace/nested-destructuring",
    // Ignore these edge cases
    "babel-preset-env/test/fixtures/bugfixes",
    "babel-preset-env/test/fixtures/corejs2",
    "babel-preset-env/test/fixtures/corejs3",
    "babel-preset-env/test/fixtures/debug",
    "babel-preset-env/test/fixtures/debug-babel-7",
    // Assumptions are not implemented yet.
    "babel-plugin-transform-object-rest-spread/test/fixtures/assumption",
    "babel-plugin-transform-object-rest-spread/test/fixtures/object-spread-loose",
    "babel-plugin-transform-object-rest-spread/test/fixtures/object-rest/remove-unused-excluded-keys-loose",
    "babel-plugin-transform-object-rest-spread/test/fixtures/object-rest/regression/gh-8323",
];
