---
name: migrate-oxlint
description: Guide for migrating a project from ESLint to Oxlint. Use when asked to migrate, convert, or switch a JavaScript/TypeScript project's linter from ESLint to Oxlint.
---

This skill guides you through migrating a JavaScript/TypeScript project from ESLint to [Oxlint](https://oxc.rs/docs/guide/usage/linter/).

## Overview

Oxlint is a high-performance linter that implements many popular ESLint rules natively in Rust. It can be used alongside ESLint or as a full replacement.

An official migration tool is available: [`@oxlint/migrate`](https://github.com/oxc-project/oxlint-migrate)

## Step 1: Run Automated Migration

Run the migration tool in the project root:

```bash
npx @oxlint/migrate
```

This reads your ESLint flat config and generates a `.oxlintrc.json` file.

### Key Options

| Option                      | Description                                                               |
| --------------------------- | ------------------------------------------------------------------------- |
| `--merge`                   | Merge with an existing `.oxlintrc.json` instead of overwriting            |
| `--type-aware`              | Include type-aware rules (requires running oxlint with `--type-aware`)    |
| `--with-nursery`            | Include experimental rules still under development                        |
| `--js-plugins [bool]`       | Enable/disable ESLint plugin migration via `jsPlugins` (default: enabled) |
| `--details`                 | List rules that could not be migrated                                     |
| `--replace-eslint-comments` | Convert `// eslint-disable` comments to `// oxlint-disable`               |
| `--output-file <file>`      | Specify output path (default: `.oxlintrc.json`)                           |

If your ESLint config is not at the default location, pass the path explicitly:

```bash
npx @oxlint/migrate ./path/to/eslint.config.js
```

## Step 2: Review Generated Config

After migration, review the generated `.oxlintrc.json`.

### Plugin Mapping

The migration tool automatically maps ESLint plugins to oxlint's built-in equivalents. The following table is for reference when reviewing the generated config:

| ESLint Plugin                                       | Oxlint Plugin Name |
| --------------------------------------------------- | ------------------ |
| `@typescript-eslint/eslint-plugin`                  | `typescript`       |
| `eslint-plugin-react` / `eslint-plugin-react-hooks` | `react`            |
| `eslint-plugin-import` / `eslint-plugin-import-x`   | `import`           |
| `eslint-plugin-unicorn`                             | `unicorn`          |
| `eslint-plugin-jsx-a11y`                            | `jsx-a11y`         |
| `eslint-plugin-react-perf`                          | `react-perf`       |
| `eslint-plugin-promise`                             | `promise`          |
| `eslint-plugin-jest`                                | `jest`             |
| `@vitest/eslint-plugin`                             | `vitest`           |
| `eslint-plugin-jsdoc`                               | `jsdoc`            |
| `eslint-plugin-next`                                | `nextjs`           |
| `eslint-plugin-node`                                | `node`             |
| `eslint-plugin-vue`                                 | `vue`              |

Default plugins (enabled when `plugins` field is omitted): `unicorn`, `typescript`, `oxc`.
Setting the `plugins` array explicitly overrides these defaults.

### Rule Categories

Oxlint groups rules into categories for bulk configuration:

```json
{
  "categories": {
    "correctness": "warn",
    "suspicious": "warn"
  }
}
```

Available categories: `correctness` (default: enabled), `suspicious`, `pedantic`, `perf`, `style`, `restriction`, `nursery`.

Individual rule settings in `rules` override category settings.

### Check Unmigrated Rules

Run with `--details` to see which ESLint rules could not be migrated:

```bash
npx @oxlint/migrate --details
```

Review the output and decide whether to keep ESLint for those rules or find oxlint alternatives.

## Step 3: Handle Unsupported Features

Some features require manual attention:

- Local plugins (relative path imports): Must be migrated manually to `jsPlugins`
- `eslint-plugin-prettier`: Not supported. Use [oxfmt](https://oxc.rs/docs/guide/usage/formatter) instead
- `settings` in override configs: Oxlint does not support `settings` inside `overrides` blocks
- ESLint v9+ plugins: Not all work with oxlint's JS Plugins API. Test with `--js-plugins`

### External ESLint Plugins

For ESLint plugins without a built-in oxlint equivalent, use the `jsPlugins` field to load them:

```json
{
  "jsPlugins": ["eslint-plugin-custom"],
  "rules": {
    "custom/my-rule": "warn"
  }
}
```

## Step 4: Update CI and Scripts

Replace ESLint commands with oxlint. Path arguments are optional; oxlint defaults to the current working directory.

```bash
# Before
npx eslint src/
npx eslint --fix src/

# After
npx oxlint@latest
npx oxlint@latest --fix
```

### Common CLI Options

| ESLint                    | oxlint                                         |
| ------------------------- | ---------------------------------------------- |
| `eslint .`                | `oxlint` (default: cwd)                        |
| `eslint src/`             | `oxlint src/`                                  |
| `eslint --fix`            | `oxlint --fix`                                 |
| `eslint --max-warnings 0` | `oxlint --deny-warnings` or `--max-warnings 0` |
| `eslint --format json`    | `oxlint --format json`                         |
| `eslint -c config.json`   | `oxlint --config config.json`                  |

Additional oxlint options:

- `--type-aware`: Enable rules requiring TypeScript type information
- `--tsconfig <path>`: Specify tsconfig.json path for type-aware linting

## Tips

- Start gradually: Enable `correctness` rules first (the default), then progressively add `suspicious`, `pedantic`, etc.
- Run alongside ESLint: Oxlint is designed to complement ESLint during migration. You can run both until all rules are covered.
- Disable comments work: `// eslint-disable` and `// eslint-disable-next-line` comments are supported by oxlint. Use `--replace-eslint-comments` to convert them to `// oxlint-disable` if desired.
- List available rules: Run `npx oxlint@latest --rules` to see all supported rules.
- Schema support: Add `"$schema": "./node_modules/oxlint/configuration_schema.json"` to `.oxlintrc.json` for editor autocompletion.
- Output formats: `default`, `stylish`, `json`, `github`, `gitlab`, `junit`, `checkstyle`, `unix`

## References

- [CLI Reference](https://oxc.rs/docs/guide/usage/linter/cli.html)
- [Config File Reference](https://oxc.rs/docs/guide/usage/linter/config-file-reference.html)
