# The JavaScript Oxidation Compiler (oxc)

[![Discord chat][discord-badge]][discord-url]
[![MIT licensed][mit-badge]][mit-url]
[![npm version][npm-badge]][npm-url]

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=discord&color=brightgreen
[discord-url]: https://discord.gg/9uXCAwqQZW
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg?color=brightgreen
[mit-url]: LICENSE
[npm-badge]: https://img.shields.io/npm/v/oxidation-compiler/latest?color=brightgreen
[npm-url]: https://www.npmjs.com/package/oxidation-compiler/v/latest

The Oxidation Compiler is building a set of tools for the JavaScript / TypeScript language.

These tools include:

* Parser - Done
* Linter - Work in progress
* Formatter
* Transpiler
* Minifier

## Goals

The goal of this project is to:

* Create a *really* fast native program by using the Rust programming language.
* Provide the basic building blocks for creating your own tools by having good API designs
* Provide good documentation on learning Rust and compiler techniques.

> Performance issues are considered as bugs in this project.

Rust but would like to learn things.

## Contributing

This project is an invitation for you to come and learn Rust with us.

Contributions are welcome and highly appreciated. To get started, check out [CONTRIBUTING.md](./CONTRIBUTING.md).

If you do not have the time to contribute code, you can still participate by:

* star and watch this project
* join us on [Discord](https://discord.gg/9uXCAwqQZW)
* [follow me on twitter](https://twitter.com/boshen_c)
* provide your wisdom in [discussions](https://github.com/Boshen/oxc/discussions)

## Milestone

Oxc has a fully working parser and a prototype for the linter right now.

The current objectives are:

* A MVP (Most Viable Product) for the linter.
* Improve the parser for real usage. Areas include:
  * Performance
  * API
  * Pass more conformance tests

## Parser Conformance

The `cargo coverage` command currently reports the following summary

```
Test262 Summary:
AST Parsed     : 43934/43934 (100.00%)

Babel Summary:
AST Parsed     : 2045/2057 (99.42%)

TypeScript Summary:
AST Parsed     : 4291/4861 (88.27%)
```

(The parser is failing some of the TypeScript recoverable parser tests.)

## Linter Performance

See [benchmark](./benchmark/) for details. Hyperfine results on an Intel i7 6-core are:

```
Benchmark 1: oxc
  Time (mean ± σ):      34.6 ms ±   1.3 ms    [User: 160.1 ms, System: 67.2 ms]
  Range (min … max):    31.8 ms …  40.9 ms    75 runs

Benchmark 2: rome
  Time (mean ± σ):     147.4 ms ±   3.7 ms    [User: 695.4 ms, System: 72.4 ms]
  Range (min … max):   141.9 ms … 153.8 ms    20 runs

Benchmark 3: eslint
  Time (mean ± σ):      2.905 s ±  0.185 s    [User: 4.387 s, System: 0.254 s]
  Range (min … max):    2.710 s …  3.287 s    10 runs

Summary
  'oxc' ran
    4.26 ± 0.20 times faster than 'rome'
   83.94 ± 6.25 times faster than 'eslint'
```

## Learning Resources

* My [small tutorial on how to write a JavaScript Parser in Rust](https://boshen.github.io/javascript-parser-in-rust/)
* [Crafting Interpreters](https://craftinginterpreters.com)
* [Create an issue and nsert your inspirational learning resources here]

## Credits

This project is incubated from the help of these great mentors and their projects:

* [Rome Tools](https://rome.tools) - [@MichaReiser](https://github.com/MichaReiser), [@ematipico](https://github.com/ematipico)
* [Ruff](https://beta.ruff.rs) - [@charliermarsh](https://github.com/charliermarsh)
* [quick-lint-js](https://quick-lint-js.com) - [@strager](https://github.com/strager)
* [elm-review](https://package.elm-lang.org/packages/jfmengels/elm-review/latest) - [@jfmengels](https://github.com/jfmengels)
* [@domonji](https://github.com/domonji) for contribution to the TypeScript parser

## License

[MIT](./LICENSE)

## Third Party Licenses

Licenses are list in [THIRD-PARTY-LICENSE](./LICENSE-THIRD-PARTY-LICENSE)

This project partially copies code from the following projects:

| Project       | License       |
| ------------- | ------------- |
| [eslint/eslint](https://github.com/eslint/eslint) | [MIT](https://github.com/eslint/eslint/blob/main/LICENSE)  |
| [typescript-eslint/typescript-eslint](https://github.com/typescript-eslint/typescript-eslint) | [MIT](https://github.com/typescript-eslint/typescript-eslint/blob/main/LICENSE)  |
| [microsoft/TypeScript](https://github.com/microsoft/TypeScript) | [Apache 2.0](https://github.com/microsoft/TypeScript/blob/main/LICENSE.txt)  |
| [rome/tools](https://github.com/rome/tools) | [MIT](https://github.com/rome/tools/blob/main/LICENSE)  |
| [mozilla-spidermonkey/jsparagus](https://github.com/mozilla-spidermonkey/jsparagus) | [MIT](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-MIT) [Apache 2.0](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-APACHE-2.0)  |
| [acorn](https://github.com/acornjs/acorn) | [MIT](https://github.com/acornjs/acorn/blob/master/acorn/LICENSE) |
| [sindresorhus/globals](https://github.com/sindresorhus/globals) | [MIT](https://github.com/sindresorhus/globals/blob/main/license) |
