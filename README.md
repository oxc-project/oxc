<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/Boshen/oxc-assets/main/preview-dark-transparent.png" width="600">
    <img alt="OXC Logo" src="https://raw.githubusercontent.com/Boshen/oxc-assets/main/preview-white.png" width="600">
  </picture>
</p>

<div align="center">

[![MIT licensed][license-badge]][license-url]
[![Build Status][ci-badge]][ci-url]
[![Code Coverage][code-coverage-badge]][code-coverage-url]
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/oxc-project/oxc)
[![Sponsors][sponsors-badge]][sponsors-url]

[![Discord chat][discord-badge]][discord-url]
[![Playground][playground-badge]][playground-url]
[![Website][website-badge]][website-url]

</div>

## ⚓ Oxc

The Oxidation Compiler is creating a collection of high-performance tools for JavaScript and TypeScript.

Oxc is building a parser, linter, formatter, transpiler, minifier, resolver ... all written in Rust.

## 💡 Philosophy

This project shares the same philosophies as [Biome][biome] and [Ruff][ruff].

1. JavaScript tooling could be rewritten in a more performant language.
2. An integrated toolchain can tap into efficiencies that are not available to a disparate set of tools.

## ⚡️ Quick Start

The linter is ready to catch mistakes for you. It comes with over 60 default rules and no configuration is required.

To start using, install [oxlint][npm-oxlint] or via `npx`:

```bash
npx oxlint@latest
```

To give you an idea of its capabilities, here is an example from the [vscode] repository, which finishes linting 4000+ files in 0.5 seconds.

<p float="left" align="left">
  <img src="https://raw.githubusercontent.com/Boshen/oxc-assets/main/linter-screenshot.png" width="60%">
</p>

## ⚡️ Performance

- The parser aim to be the fastest Rust-based ready-for-production parser.
- The linter is more than 50 times faster than [ESLint], and scales with the number of CPU cores.

<p float="left" align="middle">
  <img src="https://raw.githubusercontent.com/Boshen/bench-javascript-parser-written-in-rust/main/bar-graph.svg" width="49%">
  <img src="https://raw.githubusercontent.com/Boshen/bench-javascript-linter/main/bar-graph.svg" width="49%">
</p>

## ⌨️ Programming Usage

### Rust

Individual crates are published, you may use them to build your own JavaScript tools.

- The umbrella crate [oxc][docs-oxc-url] exports all public crates from this repository.
- The AST and parser crates [oxc_ast][docs-ast-url] and [oxc_parser][docs-parser-url] are production ready.
- See `crates/*/examples` for example usage.

While Rust has gained a reputation for its comparatively slower compilation speed,
we have dedicated significant effort to fine-tune the Rust compilation speed.
Our aim is to minimize any impact on your development workflow,
ensuring that developing your own Oxc based tools remains a smooth and efficient experience.

This is demonstrated by our [CI runs](https://github.com/oxc-project/oxc/actions/workflows/ci.yml?query=branch%3Amain),
where warm runs complete in 5 minutes.

### Node.js

- You may use the parser via napi: [oxc-parser][npm-napi]

---

## 🎯 Tools

- [AST and Parser](#-ast-and-parser)
- [Linter](#-linter)
- [Resolver](#-resolver)
- [Minifier](#-minifier)
- [Formatter](#-formatter)
- [Transpiler](#-transpiler)

### 🔸 AST and Parser

Oxc maintains its own AST and parser, which is by far the fastest and most conformant JavaScript and TypeScript (including JSX and TSX) parser written in Rust.

As the parser often represents a key performance bottleneck in JavaScript tooling,
any minor improvements can have a cascading effect on our downstream tools.
By developing our parser, we have the opportunity to explore and implement well-researched performance techniques.

While many existing JavaScript tools rely on [estree] as their AST specification,
a notable drawback is its abundance of ambiguous nodes.
This ambiguity often leads to confusion during development with [estree].

The Oxc AST differs slightly from the [estree] AST by removing ambiguous nodes and introducing distinct types.
For example, instead of using a generic [estree] `Identifier`,
the Oxc AST provides specific types such as `BindingIdentifier`, `IdentifierReference`, and `IdentifierName`.
This clear distinction greatly enhances the development experience by aligning more closely with the ECMAScript specification.

#### 🏆 Parser Performance

Our [benchmark][parser-benchmark] reveals that the Oxc parser surpasses the speed of the [swc] parser by approximately 2 times and the [Biome] parser by 3 times.

<details>
  <summary>How is it so fast?</summary>
  <ul>
    <li>AST is allocated in a memory arena (<a href="https://crates.io/crates/bumpalo">bumpalo</a>) for fast AST memory allocation and deallocation.</li>
    <li>Short strings are inlined by <a href="https://crates.io/crates/compact_str">CompactString</a>.</li>
    <li>No other heap allocations are done except the above two.</li>
    <li>Scope binding, symbol resolution and some syntax errors are not done in the parser, they are delegated to the semantic analyzer.</li>
  </ul>
</details>

### 🔸 Linter

The linter embraces convention over configuration, eliminating the need for extensive configuration and plugin setup.
Unlike other linters like [ESLint], which often require intricate configurations and plugin installations (e.g. [@typescript-eslint]),
our linter only requires a single command that you can immediately run on your codebase:

```bash
npx oxlint@latest
```

#### 🏆 Linter Performance

The linter is 50 - 100 times faster than [ESLint] depending on the number of rules and number of CPU cores used.
It completes in less than a second for most codebases with a few hundred files and completes in a few seconds for
larger monorepos. See [bench-javascript-linter](https://github.com/Boshen/bench-javascript-linter) for details.

As an upside, the binary is approximately 5MB, whereas [ESLint] and its associated plugin dependencies can easily exceed 100.

You may also download the linter binary from the [latest release tag](https://github.com/oxc-project/oxc/releases/latest) as a standalone binary,
this lets you run the linter without a Node.js installation in your CI.

<details>
  <summary>How is it so fast?</summary>
  <ul>
    <li>Oxc parser is used.</li>
    <li>AST visit is a fast operation due to linear memory scan from the memory arena.</li>
    <li>Files are linted in a multi-threaded environment, so scales with the total number of CPU cores.</li>
    <li>Every single lint rule is tuned for performance.</li>
  </ul>
</details>

### 🔸 Resolver

Module resolution plays a crucial role in JavaScript tooling, especially for tasks like multi-file analysis or bundling. However, it can often become a performance bottleneck.
To address this, we are actively working on porting [enhanced-resolve].

The resolver is production-ready and is currently being used in Rspack. Usage and examples can be found in its own [repository](https://github.com/oxc-project/oxc_resolver).

### 🔸 Transpiler

A transpiler is responsible for turning higher versions of ECMAScript to a lower version that can be used in older browsers.
We are currently focusing on an esnext to es2015 transpiler. See the [umbrella issue](https://github.com/oxc-project/oxc/issues/974) for details.

### 🔸 Minifier

JavaScript minification plays a crucial role in optimizing website performance as it reduces the amount of data sent to users,
resulting in faster page loads.
This holds tremendous economic value, particularly for e-commerce websites, where every second can equate to millions of dollars.

However, existing minifiers typically require a trade-off between compression quality and speed.
You have to choose between the slowest for the best compression or the fastest for less compression.
But what if we could develop a faster minifier without compromising on compression?

We are actively working on a prototype that aims to achieve this goal,
by porting all test cases from well-known minifiers such as [google-closure-compiler], [terser], [esbuild], and [tdewolff-minify].

Preliminary results indicate that we are on track to achieve our objectives.
With the Oxc minifier, you can expect faster minification times without sacrificing compression quality.

### 🔸 Formatter

While [prettier] has established itself as the de facto code formatter for JavaScript, there is a significant demand in the developer community for a less opinionated alternative. Recognizing this need, our ambition is to undertake research and development to create a new JavaScript formatter that offers increased flexibility and customization options.

The [prototype](https://github.com/oxc-project/oxc/tree/main/crates/oxc_prettier) is currently work in progress.

---

## ✍️ Contribute

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidance.

Check out some of the [good first issues](https://github.com/oxc-project/oxc/contribute) or ask us on [Discord][discord-url].

If you are unable to contribute by code, you can still participate by:

- Add a [GitHub Star](https://github.com/oxc-project/oxc/stargazers) to the project.
- Join us on [Discord][discord-url].
- [Follow me on twitter](https://twitter.com/boshen_c) and tweet about this project.

## 📚 Learning Resources

- My small tutorial on [how to write a JavaScript Parser in Rust](https://oxc-project.github.io/docs/learn/parser_in_rust/intro.html)
- My small article [Pursuit of Performance on Building a JavaScript Compiler](https://oxc-project.github.io/docs/learn/performance.html)
- [And more](https://oxc-project.github.io/docs/learn/references.html)

## 🤝 Credits

This project was incubated with the assistance of these exceptional mentors and their projects:

- [Biome][biome] - [@ematipico](https://github.com/ematipico)
- [Ruff][ruff] - [@charliermarsh](https://github.com/charliermarsh), [@MichaReiser](https://github.com/MichaReiser)
- [quick-lint-js](https://quick-lint-js.com) - [@strager](https://github.com/strager)
- [elm-review](https://package.elm-lang.org/packages/jfmengels/elm-review/latest) - [@jfmengels](https://github.com/jfmengels)

Special thanks go to

- [@domonji](https://github.com/domonji) for contribution to the TypeScript parser
- [@guan-wy](https://github.com/guan-wy) for the [project logo](https://github.com/Boshen/oxc-assets)

And also

- [@kaleidawave](https://github.com/kaleidawave) for [Ezno Type Checker](https://github.com/kaleidawave/ezno)
- [@zackradisic](https://github.com/zackradisic) for [tyvm](https://github.com/zackradisic/tyvm)

## 📖 License

Oxc is free and open-source software licensed under the [MIT License](./LICENSE).

Oxc partially copies code from the following projects, their licenses are listed in [**Third-party library licenses**](./THIRD-PARTY-LICENSE).

| Project                                                                                       | License                                                                                                                                                                         |
| --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [eslint/eslint](https://github.com/eslint/eslint)                                             | [MIT](https://github.com/eslint/eslint/blob/main/LICENSE)                                                                                                                       |
| [typescript-eslint/typescript-eslint](https://github.com/typescript-eslint/typescript-eslint) | [MIT](https://github.com/typescript-eslint/typescript-eslint/blob/main/LICENSE)                                                                                                 |
| [import-js/eslint-plugin-import](https://github.com/import-js/eslint-plugin-import)           | [MIT](https://github.com/import-js/eslint-plugin-import/blob/main/LICENSE)                                                                                                      |
| [jest-community/eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest)     | [MIT](https://github.com/jest-community/eslint-plugin-jest/blob/main/LICENSE)                                                                                                   |
| [microsoft/TypeScript](https://github.com/microsoft/TypeScript)                               | [Apache 2.0](https://github.com/microsoft/TypeScript/blob/main/LICENSE.txt)                                                                                                     |
| [biomejs/biome](https://github.com/biomejs/biome)                                             | [MIT](https://github.com/biomejs/biome/blob/main/LICENSE-MIT)                                                                                                                   |
| [mozilla-spidermonkey/jsparagus](https://github.com/mozilla-spidermonkey/jsparagus)           | [MIT](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-MIT) [Apache 2.0](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/LICENSE-APACHE-2.0) |
| [prettier/prettier](https://github.com/prettier/prettier)                                     | [MIT](https://github.com/prettier/prettier/blob/main/LICENSE)                                                                                                                   |
| [acorn](https://github.com/acornjs/acorn)                                                     | [MIT](https://github.com/acornjs/acorn/blob/master/acorn/LICENSE)                                                                                                               |
| [zkat/miette](https://github.com/zkat/miette)                                                 | [Apache 2.0](https://github.com/zkat/miette/blob/main/LICENSE)                                                                                                                  |
| [sindresorhus/globals](https://github.com/sindresorhus/globals)                               | [MIT](https://github.com/sindresorhus/globals/blob/main/license)                                                                                                                |
| [terser](https://github.com/terser/terser)                                                    | [BSD](https://github.com/terser/terser/blob/master/LICENSE)                                                                                                                     |
| [evanw/esbuild](https://github.com/evanw/esbuild)                                             | [MIT](https://github.com/evanw/esbuild/blob/main/LICENSE.md)                                                                                                                    |
| [google/closure-compiler](https://github.com/google/closure-compiler)                         | [Apache 2.0](https://github.com/google/closure-compiler#closure-compiler-license)                                                                                               |
| [tdewolff/minify](https://github.com/tdewolff/minify)                                         | [MIT](https://github.com/tdewolff/minify/blob/master/LICENSE)                                                                                                                   |

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=Discord
[discord-url]: https://discord.gg/9uXCAwqQZW
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: https://github.com/oxc-project/oxc/blob/main/LICENSE
[ci-badge]: https://github.com/oxc-project/oxc/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/oxc-project/oxc/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
[npm-badge]: https://img.shields.io/npm/v/oxlint/latest?color=brightgreen
[npm-url]: https://www.npmjs.com/package/oxlint/v/latest
[code-size-badge]: https://img.shields.io/github/languages/code-size/oxc-project/oxc
[code-size-url]: https://github.com/oxc-project/oxc
[code-coverage-badge]: https://codecov.io/github/oxc-project/oxc/branch/main/graph/badge.svg
[code-coverage-url]: https://codecov.io/gh/oxc-project/oxc
[sponsors-badge]: https://img.shields.io/github/sponsors/Boshen
[sponsors-url]: https://github.com/sponsors/Boshen
[playground-badge]: https://img.shields.io/badge/Playground-blue?color=9BE4E0
[playground-url]: https://oxc-project.github.io/oxc/playground
[website-badge]: https://img.shields.io/badge/Website-blue
[website-url]: https://oxc-project.github.io
[crate-oxc-url]: https://crates.io/crates/oxc
[crate-ast-url]: https://crates.io/crates/oxc_ast
[crate-parser-url]: https://crates.io/crates/oxc_parser
[docs-oxc-url]: https://docs.rs/oxc
[docs-ast-url]: https://docs.rs/oxc_ast
[docs-parser-url]: https://docs.rs/oxc_parser
[Boshen]: https://github.com/boshen
[CompactString]: https://github.com/ParkMyCar/compact_str
[ESLint]: https://eslint.org/
[acorn]: https://github.com/acornjs/acorn
[babel]: https://babel.dev
[bumpalo]: https://docs.rs/bumpalo
[contributors]: https://github.com/oxc-project/oxc/graphs/contributors
[docs-ast]: https://docs.rs/oxc/latest/oxc/ast/index.html
[docs-parser]: https://docs.rs/oxc/latest/oxc/parser/index.html
[enhanced-resolve]: https://github.com/webpack/enhanced-resolve
[esbuild]: https://esbuild.github.io/
[eslint-plugin-import]: https://www.npmjs.com/package/eslint-plugin-import
[eslint-plugin-jest]: https://www.npmjs.com/package/eslint-plugin-jest
[estree]: https://github.com/estree/estree
[google-closure-compiler]: https://github.com/google/closure-compiler
[minification-benchmarks]: https://github.com/privatenumber/minification-benchmarks
[npm-napi]: https://www.npmjs.com/package/oxc-parser
[npm-oxlint]: https://www.npmjs.com/package/oxlint
[parser-benchmark]: https://github.com/Boshen/bench-javascript-parser-written-in-rust
[prettier]: https://prettier.io
[biome]: https://biomejs.dev/
[ruff]: https://beta.ruff.rs
[swc]: https://swc.rs
[tdewolff-minify]: https://github.com/tdewolff/minify
[terser]: https://terser.org
[vscode]: https://github.com/microsoft/vscode
[@typescript-eslint]: https://typescript-eslint.io
