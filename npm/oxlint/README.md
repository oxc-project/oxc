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

</div>

# ⚓ Oxc

The Oxidation Compiler is creating a suite of high-performance tools for JavaScript and TypeScript.

## Oxlint

This is the linter for oxc.

Run

* `npx --yes oxlint@latest` in your JavaScript / TypeScript codebase and see it complete in milliseconds. No configurations are required.
* `npx oxlint@latest --help` for usage instructions.
* `npx oxlint@latest --rules` for the list of rules.


### Usage Instructions

`npx oxlint@latest --help`:

```
Usage: oxlint [-A=NAME | -D=NAME]... [--fix] [PATH]...

Allowing / Denying Multiple Lints
  For example `-D correctness -A no-debugger` or `-A all -D no-debugger`.
  The default category is "-D correctness".
  Use "--rules" for rule names.
  Use "--help --help" for rule categories.
    -A, --allow=NAME          Allow the rule or category (suppress the lint)
    -D, --deny=NAME           Deny the rule or category (emit an error)

Enable Plugins
        --import-plugin       Enable the experimental import plugin and detect ESM problems
        --jest-plugin         Enable the Jest plugin and detect test problems
        --jsx-a11y-plugin     Enable the JSX-a11y plugin and detect accessibility problems

Fix Problems
        --fix                 Fix as many issues as possible. Only unfixed issues are reported in the
                              output

Ignore Files
        --ignore-path=PATH    Specify the file to use as your .eslintignore
        --ignore-pattern=PAT  Specify patterns of files to ignore (in addition to those in .eslintignore)
        --no-ignore           Disables excluding of files from .eslintignore files, --ignore-path flags
                              and --ignore-pattern flags

Handle Warnings
        --quiet               Disable reporting on warnings, only errors are reported
        --max-warnings=INT    Specify a warning threshold, which can be used to force exit with an error
                              status if there are too many warning-level rule violations in your project

Miscellaneous
        --timing              Display the execution time of each lint rule
                              [env:TIMING: not set]
        --rules               list all the rules that are currently registered
        --threads=INT         Number of threads to use. Set to 1 for using only 1 CPU core

Codeowners
        --codeowners-file=PATH  Path to CODEOWNERS file
        --codeowners=NAME     Code owner names, e.g. @Boshen

Available positional items:
    PATH                      Single file, single path or list of paths

Available options:
    -h, --help                Prints help information
```
