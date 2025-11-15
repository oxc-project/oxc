<p align="center">
  <img alt="OXC Logo" src="https://cdn.jsdelivr.net/gh/oxc-project/oxc-assets/preview-universal.png" width="700">
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

_/oʊ ɛks siː/_

The Oxidation Compiler is a collection of high-performance tools for JavaScript and TypeScript written in Rust.

Oxc is part of [VoidZero](https://voidzero.dev/)'s vision for a unified, high-performance toolchain for JavaScript. It powers [Rolldown](https://rolldown.rs) ([Vite]'s future bundler) and enables the next generation of ultra-fast development tools that work seamlessly together.

For more information, check out our website at [oxc.rs](https://oxc.rs).

<sub>* Oxidation is the chemical process that creates rust</sub>

## 🏗️ Design Principles

- **Performance**: Through rigorous performance engineering.
- **Correctness**: Through conformance testing to standards and similar projects.
- **Developer Experience**: Clear APIs, comprehensive documentation, and sensible configuration.
- **Modular composability**: Use individual components independently or compose them into complete toolchains.

Read more about our [architecture](https://oxc.rs/docs/learn/architecture/parser.html) and [performance philosophy](https://oxc.rs/docs/learn/performance).

## 📦 Tools & Packages

| Tool        | npm                                                          | crates.io                                                   |
| ----------- | ------------------------------------------------------------ | ----------------------------------------------------------- |
| Linter      | [oxlint](https://www.npmjs.com/package/oxlint)               | -                                                           |
| Formatter   | [oxfmt](https://www.npmjs.com/package/oxfmt)                 | -                                                           |
| Parser      | [oxc-parser](https://www.npmjs.com/package/oxc-parser)       | [oxc_parser](https://crates.io/crates/oxc_parser)           |
| Transformer | [oxc-transform](https://www.npmjs.com/package/oxc-transform) | [oxc_transformer](https://crates.io/crates/oxc_transformer) |
| Minifier    | [oxc-minify](https://www.npmjs.com/package/oxc-minify)       | [oxc_minifier](https://crates.io/crates/oxc_minifier)       |
| Resolver    | [oxc-resolver](https://www.npmjs.com/package/oxc-resolver)   | [oxc_resolver](https://crates.io/crates/oxc_resolver)       |

See [documentation](https://oxc.rs/) for detailed usage guides for each tool.

## ⚡️ Quick Start

### Linter

The production-ready linter catches mistakes for you with sensible defaults and optional configuration:

```bash
npx oxlint@latest
```

To give you an idea of its capabilities, here is an example from the [vscode] repository, which finishes linting 4800+ files in 0.7 seconds:

<p float="left" align="left">
  <img src="https://cdn.jsdelivr.net/gh/oxc-project/oxc-assets/linter-screenshot.png" width="60%">
</p>

→ [oxlint documentation](https://oxc.rs/docs/guide/usage/linter/cli.html)

### Formatter

Fast, opinionated code formatter compatible with [Prettier]:

```bash
npx oxfmt@latest
```

→ [Formatter documentation](https://oxc.rs/docs/guide/usage/formatter)

### Parser (Node.js)

The fastest JavaScript/TypeScript parser written in Rust:

```bash
npm install oxc-parser
```

```js
import { parseSync } from 'oxc-parser';
const result = parseSync('const x = 1;');
```

→ [Parser documentation](https://oxc.rs/docs/guide/usage/parser)

### Transformer (Node.js)

TypeScript, React, and modern JavaScript transformation:

```bash
npm install oxc-transform
```

```js
import { transform } from 'oxc-transform';
const result = transform('source.tsx', code, { typescript: true });
```

→ [Transformer documentation](https://oxc.rs/docs/guide/usage/transformer)

### Minifier (Node.js)

High-performance JavaScript minifier:

```bash
npm install oxc-minify
```

```js
import { minify } from 'oxc-minify';
const result = minify(code, { mangle: true });
```

→ [Minifier documentation](https://oxc.rs/docs/guide/usage/minifier)

### Rust

Individual crates are published for building your own JavaScript tools:

```toml
[dependencies]
oxc = "0.x"
```

→ [Rust documentation](https://docs.rs/oxc)

## VoidZero Inc.

Oxc is a project of [VoidZero](https://voidzero.dev/), see our announcement [Announcing VoidZero - Next Generation Toolchain for JavaScript](https://voidzero.dev/blog).

If you have requirements for JavaScript tools at scale, please [get in touch](https://forms.gle/WQgjyzYJpwurpxWKA)!

## 🙋 Who's using Oxc?

[Rolldown] and [Nuxt] use Oxc for parsing. [Rolldown] also uses Oxc for transformation and minification. [Nova], [swc-node], and [knip] use [oxc_resolver][docs-resolver-url] for module resolution. [Preact], [Shopify], [ByteDance], and [Shopee] use oxlint for linting.

[See more projects using Oxc →](https://oxc.rs/docs/guide/projects.html)

## ✍️ Contribute

Check out some of the [good first issues](https://github.com/oxc-project/oxc/contribute) or ask us on [Discord][discord-url].

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidance, or read the complete [contributing guide on our website →](https://oxc.rs/docs/contribute/introduction.html)

If you are unable to contribute by code, you can still participate by:

- Add a [GitHub Star](https://github.com/oxc-project/oxc/stargazers) to the project
- Join us on [Discord][discord-url]
- [Follow me on X](https://x.com/boshen_c) and post about this project

## 🤝 Credits

This project was incubated with the assistance of these exceptional mentors and their projects:

- [Biome][biome] - [@ematipico](https://github.com/ematipico)
- [Ruff][ruff] - [@charliermarsh](https://github.com/charliermarsh), [@MichaReiser](https://github.com/MichaReiser)
- [quick-lint-js](https://github.com/quick-lint/quick-lint-js) - [@strager](https://github.com/strager)
- [elm-review](https://package.elm-lang.org/packages/jfmengels/elm-review/latest) - [@jfmengels](https://github.com/jfmengels)

Special thanks go to:

- [@domonji](https://github.com/domonji) for bootstrapping this project together and also completing the TypeScript parser
- [@tongtong-lu](https://github.com/tongtong-lu) and [@guan-wy](https://github.com/guan-wy) for designing the [project logo](https://github.com/oxc-project/oxc-assets)

## ❤ Who's [Sponsoring Oxc](https://github.com/sponsors/Boshen)?

<p align="center">
  <a href="https://github.com/sponsors/Boshen">
    <img src="https://raw.githubusercontent.com/Boshen/sponsors/main/sponsors.svg" alt="My sponsors" />
  </a>
</p>

## 📖 License

Oxc is free and open-source software licensed under the [MIT License](./LICENSE).

Oxc ports or copies code from other open source projects, their licenses are listed in [**Third-party library licenses**](./THIRD-PARTY-LICENSE).

[discord-badge]: https://img.shields.io/discord/1079625926024900739?logo=discord&label=Discord
[discord-url]: https://discord.gg/9uXCAwqQZW
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: https://github.com/oxc-project/oxc/blob/main/LICENSE
[ci-badge]: https://github.com/oxc-project/oxc/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/oxc-project/oxc/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
[code-coverage-badge]: https://codecov.io/gh/oxc-project/oxc/graph/badge.svg?token=FVHEH0BQLJ
[code-coverage-url]: https://codecov.io/gh/oxc-project/oxc
[sponsors-badge]: https://img.shields.io/github/sponsors/Boshen
[sponsors-url]: https://github.com/sponsors/Boshen
[playground-badge]: https://img.shields.io/badge/Playground-blue?color=9BE4E0
[playground-url]: https://playground.oxc.rs/
[website-badge]: https://img.shields.io/badge/Website-blue
[website-url]: https://oxc.rs
[docs-resolver-url]: https://docs.rs/oxc_resolver
[biome]: https://biomejs.dev/
[ruff]: https://beta.ruff.rs
[vscode]: https://github.com/microsoft/vscode
[rolldown]: https://rolldown.rs
[vite]: https://vitejs.dev/
[nuxt]: https://nuxt.com/
[nova]: https://trynova.dev/
[swc-node]: https://github.com/swc-project/swc-node
[knip]: https://github.com/webpro/knip
[preact]: https://preactjs.com/
[shopify]: https://shopify.com/
[bytedance]: https://www.bytedance.com/
[shopee]: https://shopee.com/
[prettier]: https://prettier.io/
