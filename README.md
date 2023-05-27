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

</div>

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=discord&color=brightgreen
[discord-url]: https://discord.gg/9uXCAwqQZW
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg?color=brightgreen
[mit-url]: LICENSE
[ci-badge]: https://github.com/Boshen/oxc/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/Boshen/oxc/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
[npm-badge]: https://img.shields.io/npm/v/oxidation-compiler/latest?color=brightgreen
[npm-url]: https://www.npmjs.com/package/oxidation-compiler/v/latest
[crates-badge]: https://img.shields.io/crates/v/oxc_parser.svg
[crates-url]: https://crates.io/crates/oxc_parser
[docs-badge]: https://docs.rs/oxc_parser/badge.svg
[docs-url]: https://docs.rs/oxc_parser
[code-size-badge]: https://img.shields.io/github/languages/code-size/Boshen/oxc
[code-size-url]: https://github.com/Boshen/oxc
[code-coverage-badge]: https://codecov.io/gh/Boshen/oxc/branch/main/graph/badge.svg
[code-coverage-url]: https://codecov.io/gh/Boshen/oxc

The Oxidation Compiler is creating a suite of tools for the JavaScript / TypeScript language:

* [AST](./crates/oxc_ast) - See [docs.rs/oxc_ast][docs-ast]
* [Parser](./crates/oxc_parser) - See [docs.rs/oxc_parser][docs-parser], [@oxidation-compiler/napi][npm-napi]
* [Linter](./crates/oxc_linter) - Work in progress. Try it out! `npx oxidation-compiler@latest lint path`
* Formatter
* Transpiler
* [Minifier](./crates/oxc_minifier) - Prototype

[docs-ast]: https://docs.rs/oxc_ast
[docs-parser]: https://docs.rs/oxc_parser
[npm-napi]: https://www.npmjs.com/package/@oxidation-compiler/napi

## Goals

The primary objectives for this project include:

* Create a *really* fast native program by using the Rust programming language
* Provide the basic building blocks for creating your own tools by having good API designs
* Provide good documentation on learning Rust and compiler techniques

> Performance issues are considered as bugs in this project.

## Milestone

As of now, Oxc has a fully working parser, a prototype for the linter and the minifier.

The current objectives are:

* A MVP (Minimal Viable Product) for the minifier.
* A MVP for the linter.

## Contributing

This project is an invitation for you to come and learn Rust with us,
any contributions to this project are appreciated.

To get started, check out some of the [Good First Issues](https://github.com/Boshen/oxc/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc+label%3A%22E-Good+First+Issue%22) or just directly ask us on Discord.

If you are unable to contribute by code, you can still participate by:

* star and watch this project
* join us on [Discord](https://discord.gg/9uXCAwqQZW)
* [follow me on twitter](https://twitter.com/boshen_c)
* provide your wisdom in [discussions](https://github.com/Boshen/oxc/discussions)

## Linter

The linter is fast to the extent that it feels broken.

With 20 rules implemented, testing in the [VSCode](https://github.com/microsoft/vscode) repo:

```
vscode  main ❯ npx oxidation-compiler@latest lint src
Checked 3479 files in 335ms using 12 cores.
Found 17 errors.
```

And also in a huge monorepo:

```
Checked 73660 files in 7415ms using 12 cores.
Found 470 errors.
```

On my Intel i7 6-core, the linter is around 84 times faster than ESLint.
But we'll get slightly slower as we add more features.

See [benchmark](./benchmark/) for details.

### Try it out yourself!

The linter is currently usable and it can potentially catch a few mistakes for you:

```
npx oxidation-compiler@latest lint path
```

All feedbacks are welcome.

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
TypeScript Summary:
AST Parsed     : 2337/2337 (100.00%)
Positive Passed: 2331/2337 (99.74%)
Negative Passed: 673/2535 (26.55%)
```

Test262 conformance is complete. TypeScript parsing is complete.

Only unstable stage 3 `json-modules` and stage 3 `decorators` tests are skipped.

## Learning Resources

* My [small tutorial on how to write a JavaScript Parser in Rust](https://boshen.github.io/javascript-parser-in-rust/)
* [Crafting Interpreters](https://craftinginterpreters.com)
* [Create an issue and insert your inspirational learning resources here]

## Rust [cloc](https://github.com/boyter/scc)

`scc . --include-ext=rs --no-complexity`

```
───────────────────────────────────────────────────────────────────────────────
Language                     Files       Lines     Blanks    Comments      Code
───────────────────────────────────────────────────────────────────────────────
Rust                           194       54278       5933        4636     43709
───────────────────────────────────────────────────────────────────────────────
Estimated Cost to Develop (organic) $1,426,246
Estimated Schedule Effort (organic) 15.74 months
Estimated People Required (organic) 8.05
```

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
