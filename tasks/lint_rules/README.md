# tasks/lint_rules

Task to update implementation progress for each linter plugin.

```sh
Usage:
  $ cmd [--target=<pluginName>]... [--update] [--help]

Options:
  --target, -t: Which plugin to target, multiple allowed
  --update: Update the issue instead of printing to stdout
  --help, -h: Print this help message
```

Environment variables `GITHUB_TOKEN` is required when `--update` is specified.

## Design

- Always install `eslint-plugin-XXX@latest` from npm
- Load them through ESLint Node.js API
  - https://eslint.org/docs/latest/integrate/nodejs-api#linter
- List all their plugin rules(name, deprecated, recommended, docs, etc...)
- List all our implemented rules(name)
- Combine these lists and render as markdown
- Update GitHub issue body

## FAQ

- Why is this task written in Node.js? Why not Rust?
  - Some plugins do not provide static rules list
    - https://github.com/jest-community/eslint-plugin-jest/
  - Easiest way to collect the list is just evaluating config file in JavaScript
