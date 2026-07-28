pub const IGNORE_TESTS: &[&str] = &[
    // Parser error message snapshots (added in Prettier v3.9.1); not a formatter concern
    "/_errors_/",
    // ----------------------------------------------------------------------------------------------------------------------------------
    // Copied from Biome, then modified
    // https://github.com/biomejs/biome/blob/cd1c8ec4249e8df8d221393586d664537c9fddb2/crates/biome_formatter_test/src/diff_report.rs#L105
    // ----------------------------------------------------------------------------------------------------------------------------------
    // Bogus nodes
    "typescript/conformance/classes/constructorDeclarations/constructorParameters/readonlyReadonly.ts",
    "typescript/conformance/parser/ecmascript5/Statements/parserES5ForOfStatement21.ts",
    // Expression syntax: `a?.b = c`
    "js/optional-chaining-assignment/",
    // Experimental syntax: `do {}`
    "js/async-do-expressions/",
    "js/do/",
    "jsx/do/",
    // Facebook Translation (fbt) is not supported
    "jsx/fbt/",
    // Experimental syntax: `export X from "mod"`
    "js/export-default/export-default-from/",
    "js/export-default/escaped/default-escaped.js",
    // Experimental syntax: `module <id> {}`
    "js/module-blocks",
    // Experimental syntax: `#[]` and `#{}`
    "js/tuple",
    "js/record",
    "js/arrays/tuple-and-record.js",
    "js/arrows/tuple-and-record.js",
    "js/binary-expressions/tuple-and-record.js",
    "js/class-extends/tuple-and-record.js",
    "js/comments-closure-typecast/tuple-and-record.js",
    "js/comments/tuple-and-record.js",
    "js/function-single-destructuring/tuple-and-record.js",
    "js/method-chain/tuple-and-record.js",
    "jsx/tuple/",
    // Experimental syntax: pipeline operator `|>`
    "js/comments-pipeline-own-line",
    "js/partial-application",
    "js/pipeline-operator",
    // Experimental syntax: `::`
    "js/arrows-bind/",
    "js/bind-expressions/",
    "js/objects/expression.js",
    "js/no-semi-babylon-extensions/no-semi.js",
    // Experimental syntax: `let { #x: x } = ...`
    "js/destructuring-private-fields",
    // Experimental syntax: `import module`
    "js/import-reflection/",
    // Experimental syntax: `throw` expressions
    "js/throw_expressions/",
    // Embedded languages in template literals
    "js/comments-closure-typecast/styled-components.js",
    "js/multiparser-comments/",
    "js/multiparser-css/",
    "js/multiparser-graphql/",
    "js/multiparser-html/",
    "js/multiparser-invalid/",
    "js/multiparser-markdown/",
    "js/multiparser-text/",
    "js/strings/template-literals.js",
    "js/template-literals/css-prop.js",
    "js/template-literals/styled-components-with-expressions.js",
    "js/template-literals/styled-jsx-with-expressions.js",
    "js/template-literals/styled-jsx.js",
    "js/range/issue-7082.js",
    "js/last-argument-expansion/embed.js",
    "jsx/template/styled-components.js",
    "typescript/as/as-const-embedded.ts",
    "js/last-argument-expansion/embed.js",
    "typescript/as/as-const-embedded.ts",
    // Syntax recovery
    "typescript/error-recovery/",
    // ----------------------------------------------------------------------------------------------------------------------------------
    // + Not yet supported by OXC
    // Some items may be already declared in the above
    // ----------------------------------------------------------------------------------------------------------------------------------
    // non-standard syntax
    "js/deferred-import-evaluation",
    "js/bind-expressions",
    // Babel plugins (mostly experimental syntaxes)
    "js/babel-plugins",
    "js/destructuring-private-fields",
    "js/do", // do expression
    "js/export-default/escaped",
    "js/export-default/export-default-from",
    "js/import-reflection",
    "js/module-blocks",
    // embedded
    "js/multiparser",
    "typescript/multiparser",
    "typescript/angular-component-examples",
    "js/partial-application",
    "js/pipeline-operator",
    "js/record",
    "js/source-phase-imports",
    "js/throw_expressions",
    "js/tuple",
    "js/arrows-bind",
    // prettier-ignore
    "js/ignore",
    "typescript/prettier-ignore",
    // range formatting
    "range",
    // IDE cursor
    "cursor",
    // Invalid (some of them are recoverable, though)
    "js/call/invalid",
    // Trailing comma after rest element
    "typescript/trailing-comma/invalid.ts",
    // Invalid modifier combos (`readonly accessor`, optional accessor, ambient initializer)
    "typescript/decorator-auto-accessors/decorator-auto-accessors-abstract-class.ts",
    "typescript/decorator-auto-accessors/decorator-auto-accessors-declare-class.ts",
    "typescript/decorator-auto-accessors/decorator-auto-accessors-mixed-modifiers.ts",
    // Ambiguous await
    "js/top-level-await",
    "jsx/top-level-await",
    "typescript/top-level-await",
    "js/ternaries/parenthesis/await-expression.js",
    // Top-level `await (1)` with no import/export: Prettier always parses `.js` as ESM (await expression),
    // while our unambiguous detection leans to script (call expression), whose output is valid under both
    "js/await/like-call.js",
    // ES5 vs ES6+ identifier: Prettier uses ES5 validation, OXC uses ES6+
    // Characters outside BMP (like U+102A7) are valid ES6+ identifiers but not ES5
    "js/quotes/objects.js",
    // Embedded Angular template
    "typescript/decorators-ts/angular.ts",
    // postcss-conditionals (archived: https://github.com/andyjansson/postcss-conditionals).
    "css/atrule/if-else.css",
    // Prettier's yaml parser rejects these (https://github.com/eemeli/yaml/issues/646),
    // so no snapshot exists (`3-style.yml` is even marked `errors` in its format.test.js).
    // oxc-yaml-parser parses them fine, but there is nothing to compare against.
    "yaml/mapping/3-style.yml",
    "yaml/spec/spec-example-2-11-mapping-between-sequences.yml",
    // Pragma support (`@format` insertion / require)
    "yaml/insert-pragma/",
    "yaml/require-pragma/",
];
