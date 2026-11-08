# template_literal_parser

Implements ECMAScript® 2025 Language Specification

- https://tc39.es/ecma262/2025/multipage/ecmascript-language-lexical-grammar.html#sec-template-literal-lexical-components

It only supports `NoSubstitutionTemplate` and returns a diagnostic when it encounters a `${...}` substitution.
