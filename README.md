<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/Boshen/oxc-assets/main/preview-dark-transparent.png" width="700">
    <img alt="OXC Logo" src="https://raw.githubusercontent.com/Boshen/oxc-assets/main/preview-white.png" width="700">
  </picture>
</p>

<div align="center">

[![Discord chat][discord-badge]][discord-url]
[![Build Status][ci-badge]][ci-url]
[![npm version][npm-badge]][npm-url]
[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]

[![Code Coverage][code-coverage-badge]][code-coverage-url]
[![Code Size][code-size-badge]][code-size-url]
[![MIT licensed][mit-badge]][mit-url]

### [Playground](https://web-infra-dev.github.io/oxc/playground)
</div>

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=discord&color=brightgreen
[discord-url]: https://discord.gg/9uXCAwqQZW
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg?color=brightgreen
[mit-url]: LICENSE
[ci-badge]: https://github.com/web-infra-dev/oxc/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/web-infra-dev/oxc/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
[npm-badge]: https://img.shields.io/npm/v/oxlint/latest?color=brightgreen
[npm-url]: https://www.npmjs.com/package/oxlint/v/latest
[crates-badge]: https://img.shields.io/crates/v/oxc_parser.svg
[crates-url]: https://crates.io/crates/oxc_parser
[docs-badge]: https://docs.rs/oxc/badge.svg
[docs-url]: https://docs.rs/oxc
[code-size-badge]: https://img.shields.io/github/languages/code-size/web-infra-dev/oxc
[code-size-url]: https://github.com/web-infra-dev/oxc
[code-coverage-badge]: https://codecov.io/github/web-infra-dev/oxc/branch/main/graph/badge.svg
[code-coverage-url]: https://codecov.io/gh/web-infra-dev/oxc

The Oxidation Compiler is creating a suite of high-performance tools for the JavaScript / TypeScript language re-written in Rust:

* [AST](./crates/oxc_ast) - See [docs.rs][docs-ast]
* [Parser](./crates/oxc_parser) ([acorn][acorn]) - See [docs.rs][docs-parser], [@oxidation-compiler/napi][npm-napi]
* [Linter](./crates/oxc_linter) ([ESLint][ESLint]) - Prototype - Try it out! `npx oxlint@latest path`
* Formatter ([prettier][prettier])
* Transpiler ([babel][babel])
* [Minifier](./crates/oxc_minifier) ([terser][terser]) - [Prototype](https://github.com/web-infra-dev/oxc/tree/main/crates/oxc_minifier)
* [Resolver](./crates/oxc_resolver) ([enhanced-resolve](enhanced-resolve))
* Type Checker - See [ezno][ezno], available via `npx oxidation-compiler@latest check path`

[docs-ast]: https://docs.rs/oxc/latest/oxc/ast/index.html
[docs-parser]: https://docs.rs/oxc/latest/oxc/parser/index.html
[npm-napi]: https://www.npmjs.com/package/@oxidation-compiler/napi
[acorn]: https://github.com/acornjs/acorn
[babel]: https://babel.dev
[prettier]: https://prettier.io
[ESLint]: https://eslint.org/
[prettier]: https://github.com/prettier/prettier
[ezno]: https://github.com/kaleidawave/ezno
[terser]: https://terser.org
[enhanced-resolve]: https://github.com/webpack/enhanced-resolve

## Philosophy

This project follows philosophies from the [Rome](https://rome.tools) and [Ruff](https://beta.ruff.rs) projects.

1. JavaScript tooling could be rewritten in a more performant language
2. An integrated toolchain can tap into efficiencies that are not available to a disparate set of tools

## Milestone

This project currently has a fully working parser, a prototype for the linter as well as the minifier.

The current objectives are:

* Publish the linter as a product
* Finish the minifier

## Contributing

This project is an invitation for you to come and learn Rust with us,
any contributions to this project are appreciated.

To get started, check out some of the [good first issues](https://github.com/web-infra-dev/oxc/contribute) or ask us on [Discord][discord-url].

If you are unable to contribute by code, you can still participate by:

* star and watch this project
* test the linter by running `npx oxlint@latest .` in your own projects
* join us on [Discord](https://discord.gg/9uXCAwqQZW)
* [follow me on twitter](https://twitter.com/boshen_c)
* provide your wisdom in [discussions](https://github.com/web-infra-dev/oxc/discussions)

## Linter

The linter is fast to the extent that it feels broken.

With 45 rules implemented, testing in the [VSCode](https://github.com/microsoft/vscode) repo on a Mac M2:

```
vscode  main ❯ npx oxlint@latest src
Finished in 388ms on 3477 files with 45 rules using 8 threads.
Found 798 warnings.
```

And also in a huge monorepo using Mac i7:

```
Finished in 5568ms on 51931 files with 45 rules using 12 threads.
```

On my Intel i7 6-core, the linter is around 80 times faster than ESLint.

See [benchmark](./benchmark/) for details.

### Try it out yourself!

The linter is currently usable and it can potentially catch a few mistakes for you:

```
npx oxlint@latest path
```

## Parser Conformance

The `cargo coverage` command reports the following conformance summary

```
Test262 Summary:
AST Parsed     : 44000/44000 (100.00%)
Positive Passed: 44000/44000 (100.00%)
Negative Passed: 3915/3915 (100.00%)

Babel Summary:
AST Parsed     : 2065/2071 (99.71%)
Positive Passed: 2062/2071 (99.57%)
Negative Passed: 1332/1502 (88.68%)

TypeScript Summary:
AST Parsed     : 2337/2337 (100.00%)
Positive Passed: 2331/2337 (99.74%)
Negative Passed: 673/2535 (26.55%)
```

[Test262 conformance](https://github.com/tc39/test262) is complete. TypeScript parsing is complete.

Only unstable stage 3 `json-modules` and stage 3 `decorators` tests are skipped.

## Learning Resources

* My [small tutorial on how to write a JavaScript Parser in Rust](https://boshen.github.io/javascript-parser-in-rust/)
* My small article - [Pursuit of Performance on Building a JavaScript Compiler](https://rustmagazine.org/issue-3/javascript-compiler/)
* [Crafting Interpreters](https://craftinginterpreters.com)
* [Create an issue and insert your inspirational learning resources here]

## Maintainers

* Project Lead: [Boshen](https://github.com/boshen)

## Credits

This project was incubated with the assistance of these exceptional mentors and their projects:

* [Rome Tools](https://rome.tools) - [@MichaReiser](https://github.com/MichaReiser), [@ematipico](https://github.com/ematipico)
* [Ruff](https://beta.ruff.rs) - [@charliermarsh](https://github.com/charliermarsh)
* [quick-lint-js](https://quick-lint-js.com) - [@strager](https://github.com/strager)
* [elm-review](https://package.elm-lang.org/packages/jfmengels/elm-review/latest) - [@jfmengels](https://github.com/jfmengels)
* [@domonji](https://github.com/domonji) for contribution to the TypeScript parser

## License

[MIT](./LICENSE)

## Third Party Licenses

Licenses are listed in [THIRD-PARTY-LICENSE](./THIRD-PARTY-LICENSE)

This project partially copies code from the following projects:

| Project       | License       |
| ------------- | ------------- |
| [eslint/eslint](https://github.com/eslint/eslint) | [MIT](https://github.com/eslint/eslint/blob/main/LICENSE)  |
| [typescript-eslint/typescript-eslint](https://github.com/typescript-eslint/typescript-eslint) | [MIT](https://github.com/typescript-eslint/typescript-eslint/blob/main/LICENSE)  |
| [import-js/eslint-plugin-import](https://github.com/import-js/eslint-plugin-import) | [MIT](https://github.com/import-js/eslint-plugin-import/blob/main/LICENSE)  |
| [jest-community/eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest) | [MIT](https://github.com/jest-community/eslint-plugin-jest/blob/main/LICENSE)  |
| [microsoft/TypeScript](https://github.com/microsoft/TypeScript) | [Apache 2.0](https://github.com/microsoft/TypeScript/blob/main/LICENSE.txt)  |
| [rome/tools](https://github.com/rome/tools) | [MIT](https://github.com/rome/tools/blob/main/LICENSE)  |
| [mozilla-spidermonkey/jsparagus](https://github.com/mozilla-spidermonkey/jsparagus) | [MIT](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-MIT) [Apache 2.0](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-APACHE-2.0)  |
| [acorn](https://github.com/acornjs/acorn) | [MIT](https://github.com/acornjs/acorn/blob/master/acorn/LICENSE) |
| [zkat/miette](https://github.com/zkat/miette) | [Apache 2.0](https://github.com/zkat/miette/blob/main/LICENSE) |
| [sindresorhus/globals](https://github.com/sindresorhus/globals) | [MIT](https://github.com/sindresorhus/globals/blob/main/license) |
| [terser](https://github.com/terser/terser) | [BSD](https://github.com/terser/terser/blob/master/LICENSE) |
| [evanw/esbuild](https://github.com/evanw/esbuild) | [MIT](https://github.com/evanw/esbuild/blob/main/LICENSE.md) |
| [google/closure-compiler](https://github.com/google/closure-compiler) | [Apache 2.0](https://github.com/google/closure-compiler#closure-compiler-license) |
| [tdewolff/minify](https://github.com/tdewolff/minify) | [MIT](https://github.com/tdewolff/minify/blob/master/LICENSE) |
