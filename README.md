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
[![Sponsors][sponsors-badge]][sponsors-url]

[![Discord chat][discord-badge]][discord-url]
[![Playground][playground-badge]][playground-url]
</div>

## ⚓ Oxc

The Oxidation Compiler is creating a suite of high-performance tools for JavaScript and TypeScript.

Oxc is a parser, linter, formatter, transpiler, minifier, resolver ... all written in Rust.

## 💡 Philosophy

This project adheres to philosophies from [Rome][rome] and [Ruff][ruff].

1. JavaScript tooling could be rewritten in a more performant language
2. An integrated toolchain can tap into efficiencies that are not available to a disparate set of tools

## ⚡️ Quick Start

The linter is ready to use. It comes with over 60 default rules and no configuration is required.

To start using, install [oxlint][npm-oxlint] or via `npx`:

```bash
npx oxlint@latest .
```

To give you an idea of its capabilities, here’s an example from the [vscode] repository, which finishes linting 4000+ files in 0.5 seconds.

<p float="left" align="left">
  <img src="https://raw.githubusercontent.com/Boshen/oxc-assets/main/linter-screenshot.png" width="60%">
</p>

## ⚡️ Performance

* The parser is currently the fastest ready-for-production Rust-based parser.
* The linter is more than 50 times faster than [ESLint], and scales with the number of CPU cores.

<p float="left" align="middle">
  <img src="https://raw.githubusercontent.com/Boshen/bench-javascript-parser-written-in-rust/main/bar-graph.svg" width="49%">
  <img src="https://raw.githubusercontent.com/Boshen/bench-javascript-linter/main/bar-graph.svg" width="49%">
</p>

## ⌨️  Programming Usage

### Rust

Individual crates are published, you may use them to build your own JavaScript tools.

* The umbrella crate [oxc][docs-oxc-url] exports all public crates from this repository.
* The AST and parser crates [oxc_ast][docs-ast-url] and [oxc_parser][docs-parser-url] are production ready.
* See `crates/*/examples` for example usage

While Rust has gained a reputation for its comparatively slower compilation speed,
we have dedicated significant effort to fine-tune the Rust compilation speed.
Our aim is to minimize any impact on your development workflow,
ensuring that developing your own Oxc based tools remains a smooth and efficient experience.

This is demonstrated by our [CI runs](https://github.com/web-infra-dev/oxc/actions/workflows/ci.yml?query=branch%3Amain),
where warm runs complete in 5 minutes.

### Node.js

* You may use the parser via napi: [@oxidation-compiler/napi][npm-napi]

----

## 🎯 Tools

- [AST and Parser](#-ast-and-parser)
- [Linter](#-linter)
- [Resolver](#-resolver)
- [Minifier](#-minifier)
- [Formatter](#-formatter)
- [Transpiler](#-transpiler)
- [Ezno Type Checker](#-ezno-type-checker)

### 🔸 AST and Parser

Oxc maintains its own AST and parser, which is by far the fastest and most conformant  JavaScript and TypeScript (including JSX and TSX) parser developed in Rust.

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

Our [benchmark][parser-benchmark] reveals that the Oxc parser surpasses the speed of the [swc] parser by approximately 2 times and the [Rome] parser by 3 times.

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
npx oxlint@latest .
```

We also plan to port essential plugins such as [eslint-plugin-import] and [eslint-plugin-jest].

#### 🏆 Linter Performance

The linter is 50 - 100 times faster than [ESLint] depending on the number of rules and number of CPU cores used.
It completes in less than a second for most codebases with a few hundred files and completes in a few seconds for
larger monorepos. See [bench-javascript-linter](https://github.com/Boshen/bench-javascript-linter) for details.

As an upside, the binary is approximately 3MB, whereas [ESLint] and its associated plugin dependencies can easily exceed 100.

You may also download the linter binary from the [latest release tag](https://github.com/web-infra-dev/oxc/releases/latest) as a standalone binary,
which means you can run the linter without a Node.js installation in your CI.

<details>
  <summary>How is it so fast?</summary>
  <ul>
    <li>Oxc parser is used.</li>
    <li>AST visit is a fast operation due to linear memory scan from the memory arena.</li>
    <li>Files are linted in a multi-threaded environment, so scales with the total number of CPU cores.</li>
    <li>Every single lint rule is tuned for performance.</li>
  </ul>
</details>

#### Linter Plugin

We are currently developing a DSL-based plugin system.
The plugin system uses [trustfall] as its query engine and a subset of GraphQL as its query language.

You will not need to use JavaScript or Rust to write a plugin, this is useful for QAs and security researchers.

### 🔸 Resolver

Module resolution plays a crucial role in JavaScript tooling, especially for tasks like multi-file analysis or bundling. However, it can often become a performance bottleneck.
To address this, we are actively working on porting [enhanced-resolve].

[eslint-plugin-import] will be our first application for the resolver, since it is currently a performance and complexity blocker for a lot of projects.

### 🔸 Minifier

JavaScript minification plays a crucial role in optimizing website performance as it reduces the amount of data sent to users,
resulting in faster page loads.
This holds tremendous economic value, particularly for e-commerce websites, where every second can equate to millions of dollars.

However, existing minifiers typically require a trade-off between compression quality and speed. You have to choose between the slowest for the best compression or the fastest for less compression.
But what if we could develop a faster minifier without compromising on compression efficiency?

We are actively working on a prototype that aims to achieve this goal,
by porting all test cases from well-known minifiers such as [google-closure-compiler], [terser], [esbuild], and [tdewolff-minify].

Preliminary results indicate that we are on track to achieve our objectives.
With the Oxc minifier, you can expect faster minification times without sacrificing compression quality.

### 🔸 Formatter

While [prettier] has established itself as the de facto code formatter for JavaScript, there is a significant demand in the developer community for a less opinionated alternative. Recognizing this need, our ambition is to undertake research and development to create a new JavaScript formatter that offers increased flexibility and customization options.
Unfortunately we are currently lacking the resources to do so.

### 🔸 Transpiler

Creating a robust transpiler requires significant investment in terms of time, expertise, and resources.
It necessitates deep knowledge of ECMAScript specifications, compatibility analysis, and code transformation techniques.
Unfortunately, we currently lack the necessary resources to embark on this endeavor.

If there is any interest, the project will be limited to an esnext to es6 transpiler.

### 🔸 Ezno Type Checker

Developed by @kaleidawave, [ezno] is a TypeScript checker written in Rust with a focus on static analysis and runtime performance.
You may read the [announcement blog post](https://kaleidawave.github.io/posts/introducing-ezno/) for more information.

The type checker is available via `npx oxidation-compiler@latest check path` and the [playground][playground-url].

----

## ✍️  Contribute

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidance.

Check out some of the [good first issues](https://github.com/web-infra-dev/oxc/contribute) or ask us on [Discord][discord-url].

If you are unable to contribute by code, you can still participate by:

* Add a [GitHub Star](https://github.com/web-infra-dev/oxc/stargazers) to the project.
* Join us on [Discord][discord-url].
* [Follow me on twitter](https://twitter.com/boshen_c) and tweet about this project.

## 📚 Learning Resources

* My small tutorial on [how to write a JavaScript Parser in Rust](https://boshen.github.io/javascript-parser-in-rust/)
* My small article [Pursuit of Performance on Building a JavaScript Compiler](https://rustmagazine.org/issue-3/javascript-compiler/)
* [Crafting Interpreters](https://craftinginterpreters.com)
* [Andrew Kelley - Practical DOD](https://vimeo.com/649009599)

## 🧑‍💻 Maintenance

* Oxc is currently being developed and maintained by project lead [Boshen] with the help of [contributors] from all over the world
* APIs should be simple and well-documented
* All performance issues (runtime and compilation speed) are considered as bugs in this project
* Third-party dependencies should be minimal
* Code coverage should be monitored for unused code. Aim for 99% code coverage
* Embrace data-oriented design

## 🤝 Credits

This project was incubated with the assistance of these exceptional mentors and their projects:

* [Rome Tools](https://rome.tools) - [@MichaReiser](https://github.com/MichaReiser), [@ematipico](https://github.com/ematipico)
* [Ruff](https://beta.ruff.rs) - [@charliermarsh](https://github.com/charliermarsh)
* [quick-lint-js](https://quick-lint-js.com) - [@strager](https://github.com/strager)
* [elm-review](https://package.elm-lang.org/packages/jfmengels/elm-review/latest) - [@jfmengels](https://github.com/jfmengels)

Special thanks go to

* [@domonji](https://github.com/domonji) for contribution to the TypeScript parser
* [@guan-wy](https://github.com/guan-wy) for the [project logo](https://github.com/Boshen/oxc-assets)

## 📖 License

Oxc is free and open-source software licensed under the [MIT License](./LICENSE).

Oxc partially copies code from the following projects, their licenses are listed in [**Third-party library licenses**](./THIRD-PARTY-LICENSE).

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
| [parcel-bundler/parcel](https://github.com/parcel-bundler/parcel) | [MIT](https://github.com/parcel-bundler/parcel/blob/v2/LICENSE) |
| [dividab/tsconfig-paths](https://github.com/dividab/tsconfig-paths) | [MIT](https://github.com/dividab/tsconfig-paths/blob/master/LICENSE) |
| [tmccombs/json-comments-rs](https://github.com/tmccombs/json-comments-rs) | [Apache 2.0](https://github.com/tmccombs/json-comments-rs/blob/main/LICENSE) |

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=Discord
[discord-url]: https://discord.gg/9uXCAwqQZW
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: ./LICENSE
[ci-badge]: https://github.com/web-infra-dev/oxc/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/web-infra-dev/oxc/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
[npm-badge]: https://img.shields.io/npm/v/oxlint/latest?color=brightgreen
[npm-url]: https://www.npmjs.com/package/oxlint/v/latest
[code-size-badge]: https://img.shields.io/github/languages/code-size/web-infra-dev/oxc
[code-size-url]: https://github.com/web-infra-dev/oxc
[code-coverage-badge]: https://codecov.io/github/web-infra-dev/oxc/branch/main/graph/badge.svg
[code-coverage-url]: https://codecov.io/gh/web-infra-dev/oxc
[sponsors-badge]: https://img.shields.io/github/sponsors/Boshen
[sponsors-url]: https://github.com/sponsors/Boshen
[playground-badge]: https://img.shields.io/badge/Playground-blue?color=9BE4E0
[playground-url]: https://web-infra-dev.github.io/oxc/playground

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
[contributors]: https://github.com/web-infra-dev/oxc/graphs/contributors
[docs-ast]: https://docs.rs/oxc/latest/oxc/ast/index.html
[docs-parser]: https://docs.rs/oxc/latest/oxc/parser/index.html
[enhanced-resolve]: https://github.com/webpack/enhanced-resolve
[esbuild]: https://esbuild.github.io/
[eslint-plugin-import]: https://www.npmjs.com/package/eslint-plugin-import
[eslint-plugin-jest]: https://www.npmjs.com/package/eslint-plugin-jest
[estree]: https://github.com/estree/estree
[ezno]: https://github.com/kaleidawave/ezno
[google-closure-compiler]: https://github.com/google/closure-compiler
[minification-benchmarks]: https://github.com/privatenumber/minification-benchmarks
[npm-napi]: https://www.npmjs.com/package/@oxidation-compiler/napi
[npm-oxlint]: https://www.npmjs.com/package/oxlint
[parser-benchmark]: https://github.com/Boshen/bench-javascript-parser-written-in-rust
[prettier]: https://github.com/prettier/prettier
[prettier]: https://prettier.io
[rome]: https://rome.tools
[ruff]: https://rome.tools
[swc]: https://swc.rs
[tdewolff-minify]: https://github.com/tdewolff/minify
[terser]: https://terser.org
[trustfall]: https://github.com/obi1kenobi/trustfall
[vscode]: https://github.com/microsoft/vscode
[@typescript-eslint]: https://typescript-eslint.io
