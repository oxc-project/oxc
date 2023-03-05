# The JavaScript Oxidation Compiler (oxc)

[![MIT licensed][mit-badge]][mit-url]
[![Discord chat][discord-badge]][discord-url]

## Why this project?

The goal of this project is to:

* Create a blazingly fast JavaScript Compiler written in Rust.
* Provide good documentation on learning Rust and compiler techniques.
* Create a linter.

And mostly importantly, an invitation for you to come and learn Rust with me.
We will learn a lot from each other!

You can watch this project and also [follow me on twitter](https://twitter.com/boshen_c) if you don't have the time to
Rust but would like to learn things.

## Contributing

Contributions are welcome and highly appreciated. To get started, check out [CONTRIBUTING.md](./CONTRIBUTING.md).

You can also join us on [Discord](https://discord.gg/9uXCAwqQZW).

## Call for action


You can take a look at some of the [good first issues](
https://github.com/Boshen/oxc/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) if you want to practice some Rust.

I welcome all nitpickings and bikesheddings if you think any of the code can be improved, just make an issue.

I have also created some [discussions](https://github.com/Boshen/oxc/discussions) for documenting my thought processes.

## Milestone

Oxc has a fully working parser and a prototype for the linter right now.

The current objectives are:

* A MVP (Most Viable Product) for the linter.
* Improve the parser for real usage. Areas include:
  * API
  * Diagnostics reporting
  * Performance
  * Pass more conformance tests
  * You may start with https://github.com/Boshen/oxc/issues/36

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

See [benchmark](./benchmark/) for details. Hyperfine results are:

```
Benchmark 1: oxc
  Time (mean ± σ):      30.9 ms ±   1.4 ms    [User: 138.2 ms, System: 54.7 ms]
  Range (min … max):    28.6 ms …  35.7 ms    83 runs

Benchmark 2: rome
  Time (mean ± σ):     145.0 ms ±   2.8 ms    [User: 674.9 ms, System: 69.9 ms]
  Range (min … max):   141.5 ms … 151.8 ms    19 runs

  Warning: Ignoring non-zero exit code.

Benchmark 3: eslint
  Time (mean ± σ):      2.661 s ±  0.057 s    [User: 4.076 s, System: 0.223 s]
  Range (min … max):    2.593 s …  2.790 s    10 runs

  Warning: Ignoring non-zero exit code.

Summary
  'oxc' ran
    4.70 ± 0.23 times faster than 'rome'
   86.20 ± 4.35 times faster than 'eslint'
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

Licenses are list in [LICENSE-THIRD-PARTY](./LICENSE-THIRD-PARTY)

This project partially copies code from the following projects:

| Project       | License       |
| ------------- | ------------- |
| [eslint/eslint](https://github.com/eslint/eslint) | [MIT](https://github.com/eslint/eslint/blob/main/LICENSE)  |
| [typescript-eslint/typescript-eslint](https://github.com/typescript-eslint/typescript-eslint) | [MIT](https://github.com/typescript-eslint/typescript-eslint/blob/main/LICENSE)  |
| [microsoft/TypeScript](https://github.com/microsoft/TypeScript) | [Apache 2.0](https://github.com/microsoft/TypeScript/blob/main/LICENSE.txt)  |
| [rome/tools](https://github.com/rome/tools) | [MIT](https://github.com/rome/tools/blob/main/LICENSE)  |
| [mozilla-spidermonkey/jsparagus](https://github.com/mozilla-spidermonkey/jsparagus) | [MIT](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-MIT) [Apache 2.0](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-APACHE-2.0)  |


[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg?color=brightgreen
[mit-url]: LICENSE
[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=discord&color=brightgreen
[discord-url]: https://discord.gg/9uXCAwqQZW
