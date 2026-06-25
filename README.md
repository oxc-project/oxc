<p align="center">
  <br>
  <br>
  <a href="https://oxc.rs" target="_blank" rel="noopener noreferrer">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://oxc.rs/oxc-light.svg">
      <source media="(prefers-color-scheme: light)" srcset="https://oxc.rs/oxc-dark.svg">
      <img alt="Oxc logo" src="https://oxc.rs/oxc-dark.svg" height="60">
    </picture>
  </a>
  <br>
  <br>
  <br>
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

Oxc is part of [VoidZero](https://voidzero.dev/)'s vision for a unified, high-performance toolchain for JavaScript. It powers [Rolldown](https://rolldown.rs) ([Vite]'s bundler) and enables the next generation of ultra-fast development tools that work seamlessly together.

For more information, check out our website at [oxc.rs](https://oxc.rs).

<sub>\* Oxidation is the chemical process that creates rust</sub>

## 🙋 Who's using Oxc?

[Rolldown] and [Nuxt] use Oxc for parsing. [Rolldown] also uses Oxc for transformation and minification. [Nova], [swc-node], and [knip] use [oxc_resolver][docs-resolver-url] for module resolution. [Preact], [Shopify], [ByteDance], and [Shopee] use oxlint for linting.

[See more projects using Oxc →](https://oxc.rs/docs/guide/projects.html)

## 🔧 Lint or Format a Codebase

- **Lint**: [Oxlint](https://oxc.rs/docs/guide/usage/linter) — `npx oxlint@latest`
- **Format**: [Oxfmt](https://oxc.rs/docs/guide/usage/formatter) — `npx oxfmt@latest`

## 🧰 Build Tooling on Top of Oxc

- Parse JavaScript and TypeScript: [Parser](https://oxc.rs/docs/guide/usage/parser)
- Transform TypeScript, JSX, and modern JavaScript: [Transformer](https://oxc.rs/docs/guide/usage/transformer)
- Minify JavaScript for production builds: [Minifier](https://oxc.rs/docs/guide/usage/minifier)
- Resolve modules for JavaScript and TypeScript: [Resolver](https://oxc.rs/docs/guide/usage/resolver)

## ✍️ Contribute

Check out some of the [good first issues](https://github.com/oxc-project/oxc/contribute) or ask us on [Discord][discord-url].

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidance, or read the complete [contributing guide on our website →](https://oxc.rs/docs/contribute/introduction.html)

If you are unable to contribute by code, you can still participate by:

- Add a [GitHub Star](https://github.com/oxc-project/oxc/stargazers) to the project
- Join us on [Discord][discord-url]
- [Follow me on X](https://x.com/boshen_c) and post about this project

## 📚 Other Resources

- [Troubleshooting](https://oxc.rs/docs/guide/troubleshooting)
- [Benchmarks](https://oxc.rs/docs/guide/benchmarks)
- [Talks and media](https://oxc.rs/docs/guide/media)
- [Team](https://oxc.rs/team)
- [Endorsements](https://oxc.rs/endorsements)
- [Releases](https://github.com/oxc-project/oxc/releases)

## ❤ Who's [Sponsoring Oxc](https://github.com/sponsors/Boshen)?

<p align="center">
  <a href="https://github.com/sponsors/Boshen">
    <img src="https://raw.githubusercontent.com/Boshen/sponsors/main/sponsors.svg" alt="My sponsors" />
  </a>
</p>

## 📖 License

Oxc is free and open-source software licensed under the [MIT License](./LICENSE).

Thank you to [namespace.so](https://namespace.so) for powering our CI/CD pipelines with fast, free macOS and Linux runners.

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
[rolldown]: https://rolldown.rs
[vite]: https://vite.dev/
[nuxt]: https://nuxt.com/
[nova]: https://trynova.dev/
[swc-node]: https://github.com/swc-project/swc-node
[knip]: https://github.com/webpro/knip
[preact]: https://preactjs.com/
[shopify]: https://shopify.com/
[bytedance]: https://www.bytedance.com/
[shopee]: https://shopee.com/
